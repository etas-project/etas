module app.consumer;

import app.shared.{shared_calc_000, shared_calc_025, shared_calc_050};

public flow run_consumer(seed: i32) -> i32 ![] {
    let a: i32 = shared_calc_000(seed);
    let b: i32 = shared_calc_025(a);
    return shared_calc_050(b);
}
