module app.main;

import std.collections.Array;
import app.expense.model.{sample_report};
import app.expense.policy.{audit};

flow main(args: Array<string>) -> i32 ![]
{
    let decision = audit(sample_report()).decision;
    return 0;
}
