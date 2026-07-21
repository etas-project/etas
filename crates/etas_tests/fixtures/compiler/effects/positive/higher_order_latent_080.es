module tests.compiler.effects.positive.higher_order_latent_080;


flow higher_order_latent_flint_flint_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var flint_total = higher_order_latent_flint_flint_prepare(seed);
    flint_total = flint_total + higher_order_latent_flint_flint_route(seed + 9);
    let latent_stage: i32 -> i32 = (value: i32) => value + 6;
    let pipeline_marker = latent_stage(seed);
    let flint_adjust: i32 -> i32 = (value: i32) => value + 3;
    flint_total = flint_adjust(flint_total);
    flint_total = flint_total + higher_order_latent_flint_flint_score(2);
    flint_total = flint_total + higher_order_latent_flint_flint_finish(6);
    if flint_total > 120 {
        flint_total = flint_total - 5;
    } else {
        flint_total = flint_total + 16;
    }
    return flint_total;
}

flow higher_order_latent_flint_flint_prepare(seed: i32) -> i32 ![]
{
    var ridge_prepare_total = seed + 7;
    var ridge_prepare_cursor = 0;
    while ridge_prepare_cursor < 8 limit Iterations(8) {
        ridge_prepare_total = ridge_prepare_total + ridge_prepare_cursor + 3;
        ridge_prepare_cursor = ridge_prepare_cursor + 1;
    }
    if ridge_prepare_total % 2 == 0 {
        ridge_prepare_total = ridge_prepare_total + higher_order_latent_flint_flint_score(1);
    } else {
        ridge_prepare_total = ridge_prepare_total - 1;
    }
    var ridge_prepare_left = ridge_prepare_total + seed;
    var ridge_prepare_right = ridge_prepare_left * 2;
    var ridge_prepare_merged = ridge_prepare_right - ridge_prepare_left;
    if ridge_prepare_merged > 18 {
        ridge_prepare_total = ridge_prepare_total + ridge_prepare_merged;
    }
    return ridge_prepare_total;
}

flow higher_order_latent_flint_flint_route(seed: i32) -> i32 ![]
{
    var ridge_route_total = seed * 7;
    var ridge_route_cursor = 0;
    while ridge_route_cursor < 9 limit Iterations(9) {
        ridge_route_total = ridge_route_total + ridge_route_cursor + 3;
        ridge_route_cursor = ridge_route_cursor + 1;
    }
    if ridge_route_total % 2 == 0 {
        ridge_route_total = ridge_route_total + 16;
    } else {
        ridge_route_total = ridge_route_total - 1;
    }
    var ridge_route_left = ridge_route_total + seed;
    var ridge_route_right = ridge_route_left * 2;
    var ridge_route_merged = ridge_route_right - ridge_route_left;
    if ridge_route_merged > 18 {
        ridge_route_total = ridge_route_total + ridge_route_merged;
    }
    return ridge_route_total;
}

flow higher_order_latent_flint_flint_score(seed: i32) -> i32 ![]
{
    var ridge_score_total = seed + 7;
    var ridge_score_cursor = 0;
    while ridge_score_cursor < 9 limit Iterations(9) {
        ridge_score_total = ridge_score_total + ridge_score_cursor + 3;
        ridge_score_cursor = ridge_score_cursor + 1;
    }
    if ridge_score_total % 2 == 0 {
        ridge_score_total = ridge_score_total + 16;
    } else {
        ridge_score_total = ridge_score_total - 1;
    }
    var ridge_score_left = ridge_score_total + seed;
    var ridge_score_right = ridge_score_left * 2;
    var ridge_score_merged = ridge_score_right - ridge_score_left;
    if ridge_score_merged > 18 {
        ridge_score_total = ridge_score_total + ridge_score_merged;
    }
    return ridge_score_total;
}

flow higher_order_latent_flint_flint_finish(seed: i32) -> i32 ![]
{
    var ridge_finish_total = seed - 7;
    var ridge_finish_cursor = 0;
    while ridge_finish_cursor < 5 limit Iterations(5) {
        ridge_finish_total = ridge_finish_total + ridge_finish_cursor + 3;
        ridge_finish_cursor = ridge_finish_cursor + 1;
    }
    if ridge_finish_total % 2 == 0 {
        ridge_finish_total = ridge_finish_total + 16;
    } else {
        ridge_finish_total = ridge_finish_total - 1;
    }
    var ridge_finish_left = ridge_finish_total + seed;
    var ridge_finish_right = ridge_finish_left * 2;
    var ridge_finish_merged = ridge_finish_right - ridge_finish_left;
    if ridge_finish_merged > 18 {
        ridge_finish_total = ridge_finish_total + ridge_finish_merged;
    }
    return ridge_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var flint_seed = 4;
    if args.len() > 0 {
        flint_seed = flint_seed + 1;
    } else {
        flint_seed = flint_seed + 2;
    }
    let flint_result = higher_order_latent_flint_flint_entry(flint_seed);
    if flint_result > 0 {
        return 0;
    }
    return 1;
}
