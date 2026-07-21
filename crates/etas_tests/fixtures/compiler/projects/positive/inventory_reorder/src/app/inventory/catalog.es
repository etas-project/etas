module app.inventory.catalog;

import std.collections.Array;

public type StockItem = {
    sku: string,
    quantity: i32,
    reorder_at: i32,
};

public flow sample_catalog() -> Array<StockItem> ![] {
    return [
        StockItem { sku = "paper", quantity = 8, reorder_at = 10 },
        StockItem { sku = "ink", quantity = 3, reorder_at = 5 },
    ];
}
