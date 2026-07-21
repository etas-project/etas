module tests.compiler.effects.negative.latent_effect_errors_084;

import std.io.{println};

flow latent_effect_errors_origin_monsoon_entry(seed: i32) -> i32 ![]
{
    var monsoon_total = latent_effect_errors_origin_monsoon_prepare(seed);
    monsoon_total = monsoon_total + latent_effect_errors_origin_monsoon_route(seed + 8);
    let latent: i32 -> i32 = (value: i32) => value + 7;
    println("latent effect escapes");
    let latent_marker = latent(seed);
    let monsoon_adjust: i32 -> i32 = (value: i32) => value + 4;
    monsoon_total = monsoon_adjust(monsoon_total);
    monsoon_total = monsoon_total + latent_effect_errors_origin_monsoon_score(6);
    monsoon_total = monsoon_total + latent_effect_errors_origin_monsoon_finish(4);
    if monsoon_total > 524 {
        monsoon_total = monsoon_total - 2;
    } else {
        monsoon_total = monsoon_total + 12;
    }
    return monsoon_total;
}

flow latent_effect_errors_origin_monsoon_prepare(seed: i32) -> i32 ![]
{
    var ripple_prepare_total = seed + 12;
    var ripple_prepare_cursor = 0;
    while ripple_prepare_cursor < 12 limit Iterations(12) {
        ripple_prepare_total = ripple_prepare_total + ripple_prepare_cursor + 1;
        ripple_prepare_cursor = ripple_prepare_cursor + 1;
    }
    if ripple_prepare_total % 2 == 0 {
        ripple_prepare_total = ripple_prepare_total + latent_effect_errors_origin_monsoon_score(1);
    } else {
        ripple_prepare_total = ripple_prepare_total - 5;
    }
    var ripple_prepare_left = ripple_prepare_total + seed;
    var ripple_prepare_right = ripple_prepare_left * 2;
    var ripple_prepare_merged = ripple_prepare_right - ripple_prepare_left;
    if ripple_prepare_merged > 19 {
        ripple_prepare_total = ripple_prepare_total + ripple_prepare_merged;
    }
    return ripple_prepare_total;
}

flow latent_effect_errors_origin_monsoon_route(seed: i32) -> i32 ![]
{
    var ripple_route_total = seed * 12;
    var ripple_route_cursor = 0;
    while ripple_route_cursor < 11 limit Iterations(11) {
        ripple_route_total = ripple_route_total + ripple_route_cursor + 1;
        ripple_route_cursor = ripple_route_cursor + 1;
    }
    if ripple_route_total % 2 == 0 {
        ripple_route_total = ripple_route_total + 6;
    } else {
        ripple_route_total = ripple_route_total - 5;
    }
    var ripple_route_left = ripple_route_total + seed;
    var ripple_route_right = ripple_route_left * 2;
    var ripple_route_merged = ripple_route_right - ripple_route_left;
    if ripple_route_merged > 19 {
        ripple_route_total = ripple_route_total + ripple_route_merged;
    }
    return ripple_route_total;
}

flow latent_effect_errors_origin_monsoon_score(seed: i32) -> i32 ![]
{
    var ripple_score_total = seed + 12;
    var ripple_score_cursor = 0;
    while ripple_score_cursor < 7 limit Iterations(7) {
        ripple_score_total = ripple_score_total + ripple_score_cursor + 1;
        ripple_score_cursor = ripple_score_cursor + 1;
    }
    if ripple_score_total % 2 == 0 {
        ripple_score_total = ripple_score_total + 6;
    } else {
        ripple_score_total = ripple_score_total - 5;
    }
    var ripple_score_left = ripple_score_total + seed;
    var ripple_score_right = ripple_score_left * 2;
    var ripple_score_merged = ripple_score_right - ripple_score_left;
    if ripple_score_merged > 19 {
        ripple_score_total = ripple_score_total + ripple_score_merged;
    }
    return ripple_score_total;
}

flow latent_effect_errors_origin_monsoon_finish(seed: i32) -> i32 ![]
{
    var ripple_finish_total = seed - 12;
    var ripple_finish_cursor = 0;
    while ripple_finish_cursor < 9 limit Iterations(9) {
        ripple_finish_total = ripple_finish_total + ripple_finish_cursor + 1;
        ripple_finish_cursor = ripple_finish_cursor + 1;
    }
    if ripple_finish_total % 2 == 0 {
        ripple_finish_total = ripple_finish_total + 6;
    } else {
        ripple_finish_total = ripple_finish_total - 5;
    }
    var ripple_finish_left = ripple_finish_total + seed;
    var ripple_finish_right = ripple_finish_left * 2;
    var ripple_finish_merged = ripple_finish_right - ripple_finish_left;
    if ripple_finish_merged > 19 {
        ripple_finish_total = ripple_finish_total + ripple_finish_merged;
    }
    return ripple_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var monsoon_seed = 1;
    if args.len() > 0 {
        monsoon_seed = monsoon_seed + 1;
    } else {
        monsoon_seed = monsoon_seed + 2;
    }
    let monsoon_result = latent_effect_errors_origin_monsoon_entry(monsoon_seed);
    if monsoon_result > 0 {
        return 0;
    }
    return 1;
}
