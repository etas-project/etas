module tests.compiler.effects.negative.latent_effect_errors_086;

import std.io.{println};

flow latent_effect_errors_radius_origin_entry(seed: i32) -> i32 ![]
{
    var origin_total = latent_effect_errors_radius_origin_prepare(seed);
    origin_total = origin_total + latent_effect_errors_radius_origin_route(seed + 1);
    let latent: i32 -> i32 = (value: i32) => value + 9;
    println("latent effect escapes");
    let latent_marker = latent(seed);
    let origin_adjust: i32 -> i32 = (value: i32) => value + 6;
    origin_total = origin_adjust(origin_total);
    origin_total = origin_total + latent_effect_errors_radius_origin_score(3);
    origin_total = origin_total + latent_effect_errors_radius_origin_finish(6);
    if origin_total > 526 {
        origin_total = origin_total - 4;
    } else {
        origin_total = origin_total + 14;
    }
    return origin_total;
}

flow latent_effect_errors_radius_origin_prepare(seed: i32) -> i32 ![]
{
    var grove_prepare_total = seed + 14;
    var grove_prepare_cursor = 0;
    while grove_prepare_cursor < 9 limit Iterations(9) {
        grove_prepare_total = grove_prepare_total + grove_prepare_cursor + 3;
        grove_prepare_cursor = grove_prepare_cursor + 1;
    }
    if grove_prepare_total % 2 == 0 {
        grove_prepare_total = grove_prepare_total + latent_effect_errors_radius_origin_score(1);
    } else {
        grove_prepare_total = grove_prepare_total - 2;
    }
    var grove_prepare_left = grove_prepare_total + seed;
    var grove_prepare_right = grove_prepare_left * 4;
    var grove_prepare_merged = grove_prepare_right - grove_prepare_left;
    if grove_prepare_merged > 21 {
        grove_prepare_total = grove_prepare_total + grove_prepare_merged;
    }
    return grove_prepare_total;
}

flow latent_effect_errors_radius_origin_route(seed: i32) -> i32 ![]
{
    var grove_route_total = seed * 14;
    var grove_route_cursor = 0;
    while grove_route_cursor < 7 limit Iterations(7) {
        grove_route_total = grove_route_total + grove_route_cursor + 3;
        grove_route_cursor = grove_route_cursor + 1;
    }
    if grove_route_total % 2 == 0 {
        grove_route_total = grove_route_total + 8;
    } else {
        grove_route_total = grove_route_total - 2;
    }
    var grove_route_left = grove_route_total + seed;
    var grove_route_right = grove_route_left * 4;
    var grove_route_merged = grove_route_right - grove_route_left;
    if grove_route_merged > 21 {
        grove_route_total = grove_route_total + grove_route_merged;
    }
    return grove_route_total;
}

flow latent_effect_errors_radius_origin_score(seed: i32) -> i32 ![]
{
    var grove_score_total = seed + 14;
    var grove_score_cursor = 0;
    while grove_score_cursor < 9 limit Iterations(9) {
        grove_score_total = grove_score_total + grove_score_cursor + 3;
        grove_score_cursor = grove_score_cursor + 1;
    }
    if grove_score_total % 2 == 0 {
        grove_score_total = grove_score_total + 8;
    } else {
        grove_score_total = grove_score_total - 2;
    }
    var grove_score_left = grove_score_total + seed;
    var grove_score_right = grove_score_left * 4;
    var grove_score_merged = grove_score_right - grove_score_left;
    if grove_score_merged > 21 {
        grove_score_total = grove_score_total + grove_score_merged;
    }
    return grove_score_total;
}

flow latent_effect_errors_radius_origin_finish(seed: i32) -> i32 ![]
{
    var grove_finish_total = seed - 14;
    var grove_finish_cursor = 0;
    while grove_finish_cursor < 11 limit Iterations(11) {
        grove_finish_total = grove_finish_total + grove_finish_cursor + 3;
        grove_finish_cursor = grove_finish_cursor + 1;
    }
    if grove_finish_total % 2 == 0 {
        grove_finish_total = grove_finish_total + 8;
    } else {
        grove_finish_total = grove_finish_total - 2;
    }
    var grove_finish_left = grove_finish_total + seed;
    var grove_finish_right = grove_finish_left * 4;
    var grove_finish_merged = grove_finish_right - grove_finish_left;
    if grove_finish_merged > 21 {
        grove_finish_total = grove_finish_total + grove_finish_merged;
    }
    return grove_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var origin_seed = 3;
    if args.len() > 0 {
        origin_seed = origin_seed + 1;
    } else {
        origin_seed = origin_seed + 2;
    }
    let origin_result = latent_effect_errors_radius_origin_entry(origin_seed);
    if origin_result > 0 {
        return 0;
    }
    return 1;
}
