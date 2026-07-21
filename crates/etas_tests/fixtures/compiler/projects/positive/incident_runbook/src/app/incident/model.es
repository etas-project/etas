module app.incident.model;

public type Incident = {
    service: string,
    severity: i32,
};

public type Runbook = {
    owner: string,
    next_action: string,
};
