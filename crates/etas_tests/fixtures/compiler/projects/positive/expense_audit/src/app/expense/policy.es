module app.expense.policy;

import app.expense.model.{ExpenseReport};

public type AuditResult = {
    decision: string,
    risk_score: i32,
};

public flow audit(report: ExpenseReport) -> AuditResult ![] {
    return AuditResult { decision = "approve", risk_score = 1 };
}
