module tests.compiler.effects.positive.higher_order_latent_084;


flow higher_order_latent_jigsaw_jigsaw_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var jigsaw_total = higher_order_latent_jigsaw_jigsaw_prepare(seed);
    jigsaw_total = jigsaw_total + higher_order_latent_jigsaw_jigsaw_route(seed + 4);
    let latent_stage: i32 -> i32 = (value: i32) => value + 10;
    let pipeline_marker = latent_stage(seed);
    let jigsaw_adjust: i32 -> i32 = (value: i32) => value + 7;
    jigsaw_total = jigsaw_adjust(jigsaw_total);
    jigsaw_total = jigsaw_total + higher_order_latent_jigsaw_jigsaw_score(6);
    jigsaw_total = jigsaw_total + higher_order_latent_jigsaw_jigsaw_finish(3);
    if jigsaw_total > 124 {
        jigsaw_total = jigsaw_total - 9;
    } else {
        jigsaw_total = jigsaw_total + 20;
    }
    return jigsaw_total;
}

flow higher_order_latent_jigsaw_jigsaw_prepare(seed: i32) -> i32 ![]
{
    var uplink_prepare_total = seed + 11;
    var uplink_prepare_cursor = 0;
    while uplink_prepare_cursor < 12 limit Iterations(12) {
        uplink_prepare_total = uplink_prepare_total + uplink_prepare_cursor + 0;
        uplink_prepare_cursor = uplink_prepare_cursor + 1;
    }
    if uplink_prepare_total % 2 == 0 {
        uplink_prepare_total = uplink_prepare_total + higher_order_latent_jigsaw_jigsaw_score(1);
    } else {
        uplink_prepare_total = uplink_prepare_total - 5;
    }
    var uplink_prepare_left = uplink_prepare_total + seed;
    var uplink_prepare_right = uplink_prepare_left * 2;
    var uplink_prepare_merged = uplink_prepare_right - uplink_prepare_left;
    if uplink_prepare_merged > 22 {
        uplink_prepare_total = uplink_prepare_total + uplink_prepare_merged;
    }
    return uplink_prepare_total;
}

flow higher_order_latent_jigsaw_jigsaw_route(seed: i32) -> i32 ![]
{
    var uplink_route_total = seed * 11;
    var uplink_route_cursor = 0;
    while uplink_route_cursor < 7 limit Iterations(7) {
        uplink_route_total = uplink_route_total + uplink_route_cursor + 0;
        uplink_route_cursor = uplink_route_cursor + 1;
    }
    if uplink_route_total % 2 == 0 {
        uplink_route_total = uplink_route_total + 20;
    } else {
        uplink_route_total = uplink_route_total - 5;
    }
    var uplink_route_left = uplink_route_total + seed;
    var uplink_route_right = uplink_route_left * 2;
    var uplink_route_merged = uplink_route_right - uplink_route_left;
    if uplink_route_merged > 22 {
        uplink_route_total = uplink_route_total + uplink_route_merged;
    }
    return uplink_route_total;
}

flow higher_order_latent_jigsaw_jigsaw_score(seed: i32) -> i32 ![]
{
    var uplink_score_total = seed + 11;
    var uplink_score_cursor = 0;
    while uplink_score_cursor < 6 limit Iterations(6) {
        uplink_score_total = uplink_score_total + uplink_score_cursor + 0;
        uplink_score_cursor = uplink_score_cursor + 1;
    }
    if uplink_score_total % 2 == 0 {
        uplink_score_total = uplink_score_total + 20;
    } else {
        uplink_score_total = uplink_score_total - 5;
    }
    var uplink_score_left = uplink_score_total + seed;
    var uplink_score_right = uplink_score_left * 2;
    var uplink_score_merged = uplink_score_right - uplink_score_left;
    if uplink_score_merged > 22 {
        uplink_score_total = uplink_score_total + uplink_score_merged;
    }
    return uplink_score_total;
}

flow higher_order_latent_jigsaw_jigsaw_finish(seed: i32) -> i32 ![]
{
    var uplink_finish_total = seed - 11;
    var uplink_finish_cursor = 0;
    while uplink_finish_cursor < 9 limit Iterations(9) {
        uplink_finish_total = uplink_finish_total + uplink_finish_cursor + 0;
        uplink_finish_cursor = uplink_finish_cursor + 1;
    }
    if uplink_finish_total % 2 == 0 {
        uplink_finish_total = uplink_finish_total + 20;
    } else {
        uplink_finish_total = uplink_finish_total - 5;
    }
    var uplink_finish_left = uplink_finish_total + seed;
    var uplink_finish_right = uplink_finish_left * 2;
    var uplink_finish_merged = uplink_finish_right - uplink_finish_left;
    if uplink_finish_merged > 22 {
        uplink_finish_total = uplink_finish_total + uplink_finish_merged;
    }
    return uplink_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var jigsaw_seed = 8;
    if args.len() > 0 {
        jigsaw_seed = jigsaw_seed + 1;
    } else {
        jigsaw_seed = jigsaw_seed + 2;
    }
    let jigsaw_result = higher_order_latent_jigsaw_jigsaw_entry(jigsaw_seed);
    if jigsaw_result > 0 {
        return 0;
    }
    return 1;
}
