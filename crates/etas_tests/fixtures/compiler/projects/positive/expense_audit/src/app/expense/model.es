module app.expense.model;

import std.collections.Array;

public type ExpenseLine = {
    category: string,
    amount: i32,
};

public type ExpenseReport = {
    owner: string,
    lines: Array<ExpenseLine>,
};

public flow sample_report() -> ExpenseReport ![] {
    return ExpenseReport {
        owner = "alex",
        lines = [
            ExpenseLine { category = "travel", amount = 120 },
        ],
    };
}
