module app.support.case;

public type CaseRequest = {
    customer_id: string,
    body: string,
};

public type CaseSummary = {
    queue: string,
    priority: i32,
    note: string,
};
