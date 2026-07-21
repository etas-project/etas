module tests.compiler.effects.negative.latent_effect_errors_077;

import std.io.{println};

flow latent_effect_errors_horizon_forest_entry(seed: i32) -> i32 ![]
{
    var forest_total = latent_effect_errors_horizon_forest_prepare(seed);
    forest_total = forest_total + latent_effect_errors_horizon_forest_route(seed + 1);
    let latent: i32 -> i32 = (value: i32) => value + 0;
    println("latent effect escapes");
    let latent_marker = latent(seed);
    let forest_adjust: i32 -> i32 = (value: i32) => value + 10;
    forest_total = forest_adjust(forest_total);
    forest_total = forest_total + latent_effect_errors_horizon_forest_score(4);
    forest_total = forest_total + latent_effect_errors_horizon_forest_finish(4);
    if forest_total > 517 {
        forest_total = forest_total - 6;
    } else {
        forest_total = forest_total + 5;
    }
    return forest_total;
}

flow latent_effect_errors_horizon_forest_prepare(seed: i32) -> i32 ![]
{
    var terra_prepare_total = seed + 5;
    var terra_prepare_cursor = 0;
    while terra_prepare_cursor < 10 limit Iterations(10) {
        terra_prepare_total = terra_prepare_total + terra_prepare_cursor + 1;
        terra_prepare_cursor = terra_prepare_cursor + 1;
    }
    if terra_prepare_total % 2 == 0 {
        terra_prepare_total = terra_prepare_total + latent_effect_errors_horizon_forest_score(1);
    } else {
        terra_prepare_total = terra_prepare_total - 3;
    }
    var terra_prepare_left = terra_prepare_total + seed;
    var terra_prepare_right = terra_prepare_left * 3;
    var terra_prepare_merged = terra_prepare_right - terra_prepare_left;
    if terra_prepare_merged > 12 {
        terra_prepare_total = terra_prepare_total + terra_prepare_merged;
    }
    return terra_prepare_total;
}

flow latent_effect_errors_horizon_forest_route(seed: i32) -> i32 ![]
{
    var terra_route_total = seed * 5;
    var terra_route_cursor = 0;
    while terra_route_cursor < 10 limit Iterations(10) {
        terra_route_total = terra_route_total + terra_route_cursor + 1;
        terra_route_cursor = terra_route_cursor + 1;
    }
    if terra_route_total % 2 == 0 {
        terra_route_total = terra_route_total + 22;
    } else {
        terra_route_total = terra_route_total - 3;
    }
    var terra_route_left = terra_route_total + seed;
    var terra_route_right = terra_route_left * 3;
    var terra_route_merged = terra_route_right - terra_route_left;
    if terra_route_merged > 12 {
        terra_route_total = terra_route_total + terra_route_merged;
    }
    return terra_route_total;
}

flow latent_effect_errors_horizon_forest_score(seed: i32) -> i32 ![]
{
    var terra_score_total = seed + 5;
    var terra_score_cursor = 0;
    while terra_score_cursor < 7 limit Iterations(7) {
        terra_score_total = terra_score_total + terra_score_cursor + 1;
        terra_score_cursor = terra_score_cursor + 1;
    }
    if terra_score_total % 2 == 0 {
        terra_score_total = terra_score_total + 22;
    } else {
        terra_score_total = terra_score_total - 3;
    }
    var terra_score_left = terra_score_total + seed;
    var terra_score_right = terra_score_left * 3;
    var terra_score_merged = terra_score_right - terra_score_left;
    if terra_score_merged > 12 {
        terra_score_total = terra_score_total + terra_score_merged;
    }
    return terra_score_total;
}

flow latent_effect_errors_horizon_forest_finish(seed: i32) -> i32 ![]
{
    var terra_finish_total = seed - 5;
    var terra_finish_cursor = 0;
    while terra_finish_cursor < 10 limit Iterations(10) {
        terra_finish_total = terra_finish_total + terra_finish_cursor + 1;
        terra_finish_cursor = terra_finish_cursor + 1;
    }
    if terra_finish_total % 2 == 0 {
        terra_finish_total = terra_finish_total + 22;
    } else {
        terra_finish_total = terra_finish_total - 3;
    }
    var terra_finish_left = terra_finish_total + seed;
    var terra_finish_right = terra_finish_left * 3;
    var terra_finish_merged = terra_finish_right - terra_finish_left;
    if terra_finish_merged > 12 {
        terra_finish_total = terra_finish_total + terra_finish_merged;
    }
    return terra_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var forest_seed = 5;
    if args.len() > 0 {
        forest_seed = forest_seed + 1;
    } else {
        forest_seed = forest_seed + 2;
    }
    let forest_result = latent_effect_errors_horizon_forest_entry(forest_seed);
    if forest_result > 0 {
        return 0;
    }
    return 1;
}
