// support: Message
// layer: type
// polarity: negative
// status: covered-negative
// expect: Message.cast<T> requires a checked safe payload conversion fact
module tests.compiler.std_conformance.type.agent_runtime_bad_cast_policy;

type Review = { accepted: bool, comments: Array<string> };
type Draft = { body: string };

flow bad_cast(message: Message<Draft>) -> Message<Review> {
    return message.cast<Review>();
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
