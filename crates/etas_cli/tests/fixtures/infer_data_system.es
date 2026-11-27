module infer_data_system;
import std.agent.prompt.{PromptEncode, PromptPart, PromptChannel};
import std.io.println;

effect Observe { action mark(label: string) -> unit; }
type Draft = { text: string };
impl Draft ~ PromptEncode {
    public flow encode(self) -> PromptPart {
        perform Observe.mark("encode");
        return PromptPart { text = self.text };
    }
}
flow input() -> Draft {
    perform Observe.mark("data");
    return Draft { text = "review me" };
}
flow instructions() -> Trusted<string> {
    perform Observe.mark("system");
    return Trusted("Review the draft.");
}
type RejectedDraft;
impl RejectedDraft ~ PromptEncode {
    public flow encode(self) -> PromptPart {
        return perform Error<string>.raise("encoder stopped");
    }
}

@model("configured-model")
agent Reviewer = {};
impl Reviewer {
    public flow review(self) -> string {
        return perform infer(input(), system = instructions());
    }
    public flow rejected(self) -> string {
        return perform infer(RejectedDraft {}, system = Trusted("Review the draft."));
    }
    public flow reversed(self) -> string {
        return perform infer<string>(system = instructions(), data = input());
    }
}
flow inspect(prompt: Prompt) -> string {
    if prompt.parts.length != 2 { return "wrong count"; }
    return match prompt.parts[0] {
        PromptChannel.System(_) => match prompt.parts[1] {
            PromptChannel.Data(part) => part.text,
            _ => "wrong data channel",
        },
        _ => "wrong system channel",
    };
}
public flow main(args: Array<string>) -> i32 {
    let reviewer = Reviewer {};
    let mock = handler {
        Agentic.infer<_, string>(prompt, _) => { println("infer"); resume inspect(prompt); }
        Observe.mark(label) => { println(label); resume (); }
    };
    let a = reviewer.review() with mock;
    let b = reviewer.reversed() with mock;
    println(a);
    println(b);
    let stopped = (reviewer.rejected() with {
        Agentic.infer<_, string>(_, _) => std.core.abort("inference ran after encoding failed");
    })?;
    match stopped {
        Err(reason) => println(reason),
        Ok(_) => { return 1; },
    };
    return if a == "review me" && b == "review me" { 0 } else { 1 };
}
