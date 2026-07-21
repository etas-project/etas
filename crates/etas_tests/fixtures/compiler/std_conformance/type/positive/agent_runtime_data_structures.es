// support: Schema, Provenance, ModelResponse, ResponseDecode, Message, SessionConfig
// layer: type
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.type.agent_runtime_data_structures;

import std.agent.message.Provenance;
import std.agent.session.SessionConfig;

type Review = { accepted: bool, comments: Array<string> };
type DecodeContext = { schema: Schema<Review>, decoder: ResponseDecode<Review>, provenance: Provenance };

flow accept_decode_context(raw: ModelResponse, ctx: DecodeContext) -> DecodeContext {
    return ctx;
}

flow cast_review(message: Message<Review>) -> Option<Message<Review>> {
    return message.cast<Review>();
}

flow attach_session(message: Message<Review>, session: SessionConfig) -> Message<Review> ![Memory.write<SessionId>] {
    return Message.with_session(message, session);
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
