// support: Array, Slice, Map, Range, Prompt, Message, CommandResult
// layer: hir
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.hir.std_method_resolution_surface;

import std.host.command.CommandResult;

type Review = { accepted: bool, comments: Array<string> };

flow collection_methods(xs: Array<i32>, scores: Map<string, i32>) -> Array<i32> {
    let first = xs.get(0);
    let strict = xs.at(1);
    let next: Array<i32> = xs.push(4);
    let window: Slice<i32> = next[0, 2);
    let copy: Array<i32> = window.to_array();
    let score = scores.get("alice");
    let range = Range.open(0, 3);
    return copy;
}

flow message_methods(message: Message<Review>) -> Option<Message<Review>> {
    return message.cast<Review>();
}

flow command_result_identity(result: CommandResult) -> CommandResult {
    return result;
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
