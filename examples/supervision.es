module supervision;

import std.effects.Fs;
import std.fs.{Region, WorkspacePath};
import std.runtime.approval.{approve, ApprovalDecision, Risk};
import std.io.println;

effect AskArchitect {
    action clarify(question: string) -> string;
}

@model("configured-model")
agent Architect = {};
impl Architect {
    public flow answer(self, question: string) -> string {
        return perform infer<string>(Prompt.new().data(question));
    }
}

@model("configured-model")
agent Coder = { revisions: Array<string> };
impl Coder {
    public flow implement<R ~ Region>(var self, task: string, path: WorkspacePath<R>) -> string
        ![AskArchitect.clarify, Fs.write<R>, Error<std.fs.IOError>] {
        let design = perform AskArchitect.clarify(task);
        let code = perform infer<string>(Prompt.new().data(task).data(design));
        self.revisions = self.revisions.push(code);
        perform Fs.write<R>(path, std.codec.text.utf8_encode(code));
        return code;
    }

    public flow count(self) -> usize ![] { return self.revisions.len(); }
}

type DemoRoot;
impl DemoRoot ~ Region {}

flow main(args: Array<string>) -> i32 {
    let coder = Coder { revisions = [] };
    let architect = Architect {};
    let path = match std.fs.path<DemoRoot>("generated.es") {
        Ok(value) => value,
        Err(_) => { return 1; },
    };
    // Only the demo boundary is mocked: no model, file write or real approval.
    let services = handler {
        Agentic.infer<Architect.answer, string>(_, _) => "Write a pure flow.";
        Agentic.infer<Coder.implement, string>(_, _) => "flow generated() -> i32 { return 42; }";
        Fs.write<DemoRoot>(_, _) => { println("Dry run: write accepted."); () }
    };
    return handle {
        let rejected = Build(coder, architect, "Return 42", path) with {
            Approval.request(_) => ApprovalDecision.Rejected;
        };
        match rejected {
            Err(message) => println(message),
            Ok(_) => { return 1; },
        };
        if coder.count() != 0 { return 1; }

        let accepted = Build(coder, architect, "Return 42", path) with {
            Approval.request(_) => ApprovalDecision.Accepted;
        };
        match accepted {
            Ok(code) => println(code),
            Err(_) => { return 1; },
        };
        if coder.count() != 1 { return 1; }
        return 0;
    } with services;
}

flow Build<R ~ Region>(coder: Coder, architect: Architect, task: string, path: WorkspacePath<R>)
    -> Result<string, string> {
    return handle Ok(coder.implement(task, path)) with {
        AskArchitect.clarify(question) => architect.answer(question);
        Fs.write<R>(target, body) => {
            if !approve("Write generated code?", body, Risk.High) {
                finish Err("Write rejected");
            }
            perform Fs.write<R>(target, body)
        }
    };
}
