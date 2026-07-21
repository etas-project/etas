module app.main;

import std.collections.Array;
import app.checkout.cart.{sample_cart};
import app.checkout.pricing.{quote};

flow main(args: Array<string>) -> i32 ![]
{
    let result = quote(sample_cart());
    let status = result.status;
    return 0;
}
