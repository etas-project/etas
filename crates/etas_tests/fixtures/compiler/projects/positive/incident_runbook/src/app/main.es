module app.main;

import std.collections.Array;
import app.incident.model.{Incident};
import app.incident.runbook.{build_runbook};

flow main(args: Array<string>) -> i32 ![]
{
    let incident = Incident { service = "api", severity = 2 };
    let next_action = build_runbook(incident).next_action;
    return 0;
}
