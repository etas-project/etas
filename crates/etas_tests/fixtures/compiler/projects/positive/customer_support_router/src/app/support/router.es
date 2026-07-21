module app.support.router;

import app.support.case.{CaseRequest, CaseSummary};

public flow route_case(request: CaseRequest) -> string ![] {
    let summary = CaseSummary {
        queue = "billing",
        priority = 2,
        note = request.body,
    };
    return summary.queue;
}
