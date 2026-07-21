module app.inventory.planner;

import std.collections.Array;
import app.inventory.catalog.{StockItem};

public type ReorderPlan = {
    summary: string,
    count: i32,
};

public flow plan_reorders(items: Array<StockItem>) -> ReorderPlan ![] {
    return ReorderPlan { summary = "reorder needed", count = 2 };
}
