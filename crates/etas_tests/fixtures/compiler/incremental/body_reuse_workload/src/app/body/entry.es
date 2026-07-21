module app.body.entry;

import app.body.workload.{body_calc_000, body_calc_050, body_calc_100, body_calc_150};
import app.body.chain.{chain_calc_000, chain_calc_050, chain_calc_100};

public flow run_body_workload(seed: i32) -> i32 ![] {
    let a: i32 = body_calc_000(seed);
    let b: i32 = body_calc_050(a);
    let c: i32 = body_calc_100(b);
    let d: i32 = body_calc_150(c);
    let e: i32 = chain_calc_000(d);
    let f: i32 = chain_calc_050(e);
    return chain_calc_100(f);
}
