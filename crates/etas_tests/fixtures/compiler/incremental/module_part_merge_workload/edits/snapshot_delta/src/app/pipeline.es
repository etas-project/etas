module app.pipeline;

import app.pipeline_helpers.{helper_adjust};

public flow run_pipeline(seed: i32) -> i32 ![] {
    let a: i32 = pipeline_part_a_000(seed);
    let b: i32 = pipeline_part_b_000(a);
    let c: i32 = pipeline_part_c_000(b);
    return helper_adjust(c);
}
