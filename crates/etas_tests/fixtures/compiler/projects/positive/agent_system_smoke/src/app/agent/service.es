module app.agent.service;

import std.io.{println};
import app.agent.model.{smoke_request};

agent SmokeWriter(input: string) -> string {
    return Prompt.new()
        .system(Trusted("Return one short smoke-test sentence."))
        .data(input);
}

public flow run_agent_smoke(topic: string) -> string ![Error<IOError>] {
    let request = smoke_request(topic);
    let summary = SmokeWriter.run(request);
    println(summary);
    return summary;
}
