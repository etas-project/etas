module app.checkout.pricing;

import app.checkout.cart.{Cart};

public type Quote = {
    status: string,
    total: i32,
};

public flow quote(cart: Cart) -> Quote ![] {
    return Quote { status = "ready", total = 30 };
}
