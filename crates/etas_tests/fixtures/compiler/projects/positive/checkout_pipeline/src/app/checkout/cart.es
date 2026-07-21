module app.checkout.cart;

import std.collections.Array;

public type CartLine = {
    sku: string,
    quantity: i32,
    unit_price: i32,
};

public type Cart = {
    lines: Array<CartLine>,
};

public flow sample_cart() -> Cart ![] {
    return Cart {
        lines = [
            CartLine { sku = "book", quantity = 2, unit_price = 15 },
        ],
    };
}
