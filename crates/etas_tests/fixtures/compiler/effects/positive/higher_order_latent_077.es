module tests.compiler.effects.positive.higher_order_latent_077;


flow higher_order_latent_cascade_cascade_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var cascade_total = higher_order_latent_cascade_cascade_prepare(seed);
    cascade_total = cascade_total + higher_order_latent_cascade_cascade_route(seed + 6);
    let latent_stage: i32 -> i32 = (value: i32) => value + 3;
    let pipeline_marker = latent_stage(seed);
    let cascade_adjust: i32 -> i32 = (value: i32) => value + 13;
    cascade_total = cascade_adjust(cascade_total);
    cascade_total = cascade_total + higher_order_latent_cascade_cascade_score(4);
    cascade_total = cascade_total + higher_order_latent_cascade_cascade_finish(3);
    if cascade_total > 117 {
        cascade_total = cascade_total - 2;
    } else {
        cascade_total = cascade_total + 13;
    }
    return cascade_total;
}

flow higher_order_latent_cascade_cascade_prepare(seed: i32) -> i32 ![]
{
    var umbra_prepare_total = seed + 4;
    var umbra_prepare_cursor = 0;
    while umbra_prepare_cursor < 10 limit Iterations(10) {
        umbra_prepare_total = umbra_prepare_total + umbra_prepare_cursor + 0;
        umbra_prepare_cursor = umbra_prepare_cursor + 1;
    }
    if umbra_prepare_total % 2 == 0 {
        umbra_prepare_total = umbra_prepare_total + higher_order_latent_cascade_cascade_score(1);
    } else {
        umbra_prepare_total = umbra_prepare_total - 3;
    }
    var umbra_prepare_left = umbra_prepare_total + seed;
    var umbra_prepare_right = umbra_prepare_left * 3;
    var umbra_prepare_merged = umbra_prepare_right - umbra_prepare_left;
    if umbra_prepare_merged > 15 {
        umbra_prepare_total = umbra_prepare_total + umbra_prepare_merged;
    }
    return umbra_prepare_total;
}

flow higher_order_latent_cascade_cascade_route(seed: i32) -> i32 ![]
{
    var umbra_route_total = seed * 4;
    var umbra_route_cursor = 0;
    while umbra_route_cursor < 12 limit Iterations(12) {
        umbra_route_total = umbra_route_total + umbra_route_cursor + 0;
        umbra_route_cursor = umbra_route_cursor + 1;
    }
    if umbra_route_total % 2 == 0 {
        umbra_route_total = umbra_route_total + 13;
    } else {
        umbra_route_total = umbra_route_total - 3;
    }
    var umbra_route_left = umbra_route_total + seed;
    var umbra_route_right = umbra_route_left * 3;
    var umbra_route_merged = umbra_route_right - umbra_route_left;
    if umbra_route_merged > 15 {
        umbra_route_total = umbra_route_total + umbra_route_merged;
    }
    return umbra_route_total;
}

flow higher_order_latent_cascade_cascade_score(seed: i32) -> i32 ![]
{
    var umbra_score_total = seed + 4;
    var umbra_score_cursor = 0;
    while umbra_score_cursor < 6 limit Iterations(6) {
        umbra_score_total = umbra_score_total + umbra_score_cursor + 0;
        umbra_score_cursor = umbra_score_cursor + 1;
    }
    if umbra_score_total % 2 == 0 {
        umbra_score_total = umbra_score_total + 13;
    } else {
        umbra_score_total = umbra_score_total - 3;
    }
    var umbra_score_left = umbra_score_total + seed;
    var umbra_score_right = umbra_score_left * 3;
    var umbra_score_merged = umbra_score_right - umbra_score_left;
    if umbra_score_merged > 15 {
        umbra_score_total = umbra_score_total + umbra_score_merged;
    }
    return umbra_score_total;
}

flow higher_order_latent_cascade_cascade_finish(seed: i32) -> i32 ![]
{
    var umbra_finish_total = seed - 4;
    var umbra_finish_cursor = 0;
    while umbra_finish_cursor < 10 limit Iterations(10) {
        umbra_finish_total = umbra_finish_total + umbra_finish_cursor + 0;
        umbra_finish_cursor = umbra_finish_cursor + 1;
    }
    if umbra_finish_total % 2 == 0 {
        umbra_finish_total = umbra_finish_total + 13;
    } else {
        umbra_finish_total = umbra_finish_total - 3;
    }
    var umbra_finish_left = umbra_finish_total + seed;
    var umbra_finish_right = umbra_finish_left * 3;
    var umbra_finish_merged = umbra_finish_right - umbra_finish_left;
    if umbra_finish_merged > 15 {
        umbra_finish_total = umbra_finish_total + umbra_finish_merged;
    }
    return umbra_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var cascade_seed = 1;
    if args.len() > 0 {
        cascade_seed = cascade_seed + 1;
    } else {
        cascade_seed = cascade_seed + 2;
    }
    let cascade_result = higher_order_latent_cascade_cascade_entry(cascade_seed);
    if cascade_result > 0 {
        return 0;
    }
    return 1;
}
