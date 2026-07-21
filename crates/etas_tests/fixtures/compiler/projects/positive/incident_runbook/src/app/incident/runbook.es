module app.incident.runbook;

import app.incident.model.{Incident, Runbook};

public flow build_runbook(incident: Incident) -> Runbook ![] {
    if incident.severity <= 1 {
        return Runbook { owner = "support", next_action = "watch" };
    }

    return Runbook { owner = "sre", next_action = "page primary" };
}
