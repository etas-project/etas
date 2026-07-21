module tests.compiler.effects.negative.latent_effect_errors_080;

import std.io.{println};

flow latent_effect_errors_keeper_isotope_entry(seed: i32) -> i32 ![]
{
    var isotope_total = latent_effect_errors_keeper_isotope_prepare(seed);
    isotope_total = isotope_total + latent_effect_errors_keeper_isotope_route(seed + 4);
    let latent: i32 -> i32 = (value: i32) => value + 3;
    println("latent effect escapes");
    let latent_marker = latent(seed);
    let isotope_adjust: i32 -> i32 = (value: i32) => value + 13;
    isotope_total = isotope_adjust(isotope_total);
    isotope_total = isotope_total + latent_effect_errors_keeper_isotope_score(2);
    isotope_total = isotope_total + latent_effect_errors_keeper_isotope_finish(7);
    if isotope_total > 520 {
        isotope_total = isotope_total - 9;
    } else {
        isotope_total = isotope_total + 8;
    }
    return isotope_total;
}

flow latent_effect_errors_keeper_isotope_prepare(seed: i32) -> i32 ![]
{
    var pearl_prepare_total = seed + 8;
    var pearl_prepare_cursor = 0;
    while pearl_prepare_cursor < 8 limit Iterations(8) {
        pearl_prepare_total = pearl_prepare_total + pearl_prepare_cursor + 4;
        pearl_prepare_cursor = pearl_prepare_cursor + 1;
    }
    if pearl_prepare_total % 2 == 0 {
        pearl_prepare_total = pearl_prepare_total + latent_effect_errors_keeper_isotope_score(1);
    } else {
        pearl_prepare_total = pearl_prepare_total - 1;
    }
    var pearl_prepare_left = pearl_prepare_total + seed;
    var pearl_prepare_right = pearl_prepare_left * 2;
    var pearl_prepare_merged = pearl_prepare_right - pearl_prepare_left;
    if pearl_prepare_merged > 15 {
        pearl_prepare_total = pearl_prepare_total + pearl_prepare_merged;
    }
    return pearl_prepare_total;
}

flow latent_effect_errors_keeper_isotope_route(seed: i32) -> i32 ![]
{
    var pearl_route_total = seed * 8;
    var pearl_route_cursor = 0;
    while pearl_route_cursor < 7 limit Iterations(7) {
        pearl_route_total = pearl_route_total + pearl_route_cursor + 4;
        pearl_route_cursor = pearl_route_cursor + 1;
    }
    if pearl_route_total % 2 == 0 {
        pearl_route_total = pearl_route_total + 25;
    } else {
        pearl_route_total = pearl_route_total - 1;
    }
    var pearl_route_left = pearl_route_total + seed;
    var pearl_route_right = pearl_route_left * 2;
    var pearl_route_merged = pearl_route_right - pearl_route_left;
    if pearl_route_merged > 15 {
        pearl_route_total = pearl_route_total + pearl_route_merged;
    }
    return pearl_route_total;
}

flow latent_effect_errors_keeper_isotope_score(seed: i32) -> i32 ![]
{
    var pearl_score_total = seed + 8;
    var pearl_score_cursor = 0;
    while pearl_score_cursor < 10 limit Iterations(10) {
        pearl_score_total = pearl_score_total + pearl_score_cursor + 4;
        pearl_score_cursor = pearl_score_cursor + 1;
    }
    if pearl_score_total % 2 == 0 {
        pearl_score_total = pearl_score_total + 25;
    } else {
        pearl_score_total = pearl_score_total - 1;
    }
    var pearl_score_left = pearl_score_total + seed;
    var pearl_score_right = pearl_score_left * 2;
    var pearl_score_merged = pearl_score_right - pearl_score_left;
    if pearl_score_merged > 15 {
        pearl_score_total = pearl_score_total + pearl_score_merged;
    }
    return pearl_score_total;
}

flow latent_effect_errors_keeper_isotope_finish(seed: i32) -> i32 ![]
{
    var pearl_finish_total = seed - 8;
    var pearl_finish_cursor = 0;
    while pearl_finish_cursor < 5 limit Iterations(5) {
        pearl_finish_total = pearl_finish_total + pearl_finish_cursor + 4;
        pearl_finish_cursor = pearl_finish_cursor + 1;
    }
    if pearl_finish_total % 2 == 0 {
        pearl_finish_total = pearl_finish_total + 25;
    } else {
        pearl_finish_total = pearl_finish_total - 1;
    }
    var pearl_finish_left = pearl_finish_total + seed;
    var pearl_finish_right = pearl_finish_left * 2;
    var pearl_finish_merged = pearl_finish_right - pearl_finish_left;
    if pearl_finish_merged > 15 {
        pearl_finish_total = pearl_finish_total + pearl_finish_merged;
    }
    return pearl_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var isotope_seed = 8;
    if args.len() > 0 {
        isotope_seed = isotope_seed + 1;
    } else {
        isotope_seed = isotope_seed + 2;
    }
    let isotope_result = latent_effect_errors_keeper_isotope_entry(isotope_seed);
    if isotope_result > 0 {
        return 0;
    }
    return 1;
}
