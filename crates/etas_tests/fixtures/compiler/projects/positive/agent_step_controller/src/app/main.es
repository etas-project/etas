module app.main;

import std.agent.{StepDecision, ToolCall, ToolResult};
import std.agent.conversation;
import std.runtime.effects.Agentic;
import std.tool;
import std.io.println;

tool uppercase(text: string) -> string ![] {
    return std.text.uppercase(text);
}

flow invoke_native<C>(call: ToolCall<C>) -> ToolResult<C> {
    return tool.invoke<C>(call);
}

@model("configured-model")
@tools([uppercase])
agent Writer(input: string) -> string {
    let prompt = Prompt.new()
        .system(Trusted("Use uppercase when needed, then return the final text."))
        .data(input);
    return perform infer<string>(prompt);
}

flow RunControlled(input: string) -> string {
    return handle Writer.run(input) with {
        Agentic.infer<Writer.run, string>(prompt, schema) => {
            var history = conversation.from_prompt(prompt);
            for _ in [0, 8) limit Iterations(8) {
                let step = std.agent.step(history, schema);
                history = match conversation.append_step(history, step) {
                    Ok(next) => next,
                    Err(error) => abort(error.message),
                };
                match step.decision {
                    StepDecision.Final(answer) => { resume answer; },
                    StepDecision.CallTools(calls) => {
                        var results: Array<ToolResult<Writer.run>> = [];
                        for call in calls limit Iterations(64) {
                            results = results.push(invoke_native(call));
                        }
                        history = match conversation.append_results(history, results) {
                            Ok(next) => next,
                            Err(error) => abort(error.message),
                        };
                    },
                    StepDecision.Stopped(_) => { abort("Model step stopped without a final answer"); },
                };
            }
            abort("Model step limit reached");
        }
    };
}

public flow main(args: Array<string>) -> i32 {
    println(RunControlled("hi"))?;
    return 0;
}
