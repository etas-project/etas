module app.main;

import std.collections.Array;
import app.inventory.catalog.{sample_catalog};
import app.inventory.planner.{plan_reorders};

flow main(args: Array<string>) -> i32 ![]
{
    let plan = plan_reorders(sample_catalog());
    let summary = plan.summary;
    return 0;
}
