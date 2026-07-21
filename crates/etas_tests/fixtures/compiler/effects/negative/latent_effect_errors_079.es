module tests.compiler.effects.negative.latent_effect_errors_079;

import std.io.{println};

flow latent_effect_errors_junction_horizon_entry(seed: i32) -> i32 ![]
{
    var horizon_total = latent_effect_errors_junction_horizon_prepare(seed);
    horizon_total = horizon_total + latent_effect_errors_junction_horizon_route(seed + 3);
    let latent: i32 -> i32 = (value: i32) => value + 2;
    println("latent effect escapes");
    let latent_marker = latent(seed);
    let horizon_adjust: i32 -> i32 = (value: i32) => value + 12;
    horizon_total = horizon_adjust(horizon_total);
    horizon_total = horizon_total + latent_effect_errors_junction_horizon_score(6);
    horizon_total = horizon_total + latent_effect_errors_junction_horizon_finish(6);
    if horizon_total > 519 {
        horizon_total = horizon_total - 8;
    } else {
        horizon_total = horizon_total + 7;
    }
    return horizon_total;
}

flow latent_effect_errors_junction_horizon_prepare(seed: i32) -> i32 ![]
{
    var india_prepare_total = seed + 7;
    var india_prepare_cursor = 0;
    while india_prepare_cursor < 12 limit Iterations(12) {
        india_prepare_total = india_prepare_total + india_prepare_cursor + 3;
        india_prepare_cursor = india_prepare_cursor + 1;
    }
    if india_prepare_total % 2 == 0 {
        india_prepare_total = india_prepare_total + latent_effect_errors_junction_horizon_score(1);
    } else {
        india_prepare_total = india_prepare_total - 5;
    }
    var india_prepare_left = india_prepare_total + seed;
    var india_prepare_right = india_prepare_left * 5;
    var india_prepare_merged = india_prepare_right - india_prepare_left;
    if india_prepare_merged > 14 {
        india_prepare_total = india_prepare_total + india_prepare_merged;
    }
    return india_prepare_total;
}

flow latent_effect_errors_junction_horizon_route(seed: i32) -> i32 ![]
{
    var india_route_total = seed * 7;
    var india_route_cursor = 0;
    while india_route_cursor < 12 limit Iterations(12) {
        india_route_total = india_route_total + india_route_cursor + 3;
        india_route_cursor = india_route_cursor + 1;
    }
    if india_route_total % 2 == 0 {
        india_route_total = india_route_total + 24;
    } else {
        india_route_total = india_route_total - 5;
    }
    var india_route_left = india_route_total + seed;
    var india_route_right = india_route_left * 5;
    var india_route_merged = india_route_right - india_route_left;
    if india_route_merged > 14 {
        india_route_total = india_route_total + india_route_merged;
    }
    return india_route_total;
}

flow latent_effect_errors_junction_horizon_score(seed: i32) -> i32 ![]
{
    var india_score_total = seed + 7;
    var india_score_cursor = 0;
    while india_score_cursor < 9 limit Iterations(9) {
        india_score_total = india_score_total + india_score_cursor + 3;
        india_score_cursor = india_score_cursor + 1;
    }
    if india_score_total % 2 == 0 {
        india_score_total = india_score_total + 24;
    } else {
        india_score_total = india_score_total - 5;
    }
    var india_score_left = india_score_total + seed;
    var india_score_right = india_score_left * 5;
    var india_score_merged = india_score_right - india_score_left;
    if india_score_merged > 14 {
        india_score_total = india_score_total + india_score_merged;
    }
    return india_score_total;
}

flow latent_effect_errors_junction_horizon_finish(seed: i32) -> i32 ![]
{
    var india_finish_total = seed - 7;
    var india_finish_cursor = 0;
    while india_finish_cursor < 12 limit Iterations(12) {
        india_finish_total = india_finish_total + india_finish_cursor + 3;
        india_finish_cursor = india_finish_cursor + 1;
    }
    if india_finish_total % 2 == 0 {
        india_finish_total = india_finish_total + 24;
    } else {
        india_finish_total = india_finish_total - 5;
    }
    var india_finish_left = india_finish_total + seed;
    var india_finish_right = india_finish_left * 5;
    var india_finish_merged = india_finish_right - india_finish_left;
    if india_finish_merged > 14 {
        india_finish_total = india_finish_total + india_finish_merged;
    }
    return india_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var horizon_seed = 7;
    if args.len() > 0 {
        horizon_seed = horizon_seed + 1;
    } else {
        horizon_seed = horizon_seed + 2;
    }
    let horizon_result = latent_effect_errors_junction_horizon_entry(horizon_seed);
    if horizon_result > 0 {
        return 0;
    }
    return 1;
}
