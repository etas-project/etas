module app.main;

import std.collections.Array;
import app.agent.service.{run_agent_smoke};

flow main(args: Array<string>) -> i32 ![Error<IOError>] {
    let summary = run_agent_smoke("project-level agent smoke");
    if summary == "" {
        return 1;
    }
    return 0;
}
