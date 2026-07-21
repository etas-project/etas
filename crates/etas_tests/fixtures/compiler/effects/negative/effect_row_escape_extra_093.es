module tests.compiler.effects.negative.effect_row_escape_extra_093;

import std.io.{println};

flow effect_row_escape_extra_zone_western_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var western_total = effect_row_escape_extra_zone_western_prepare(seed);
    western_total = western_total + effect_row_escape_extra_zone_western_route(seed + 8);
    let local_values = ["left", "right", "center"];
    let first = local_values[0];
    println(first);
    let western_adjust: i32 -> i32 = (value: i32) => value + 13;
    western_total = western_adjust(western_total);
    western_total = western_total + effect_row_escape_extra_zone_western_score(5);
    western_total = western_total + effect_row_escape_extra_zone_western_finish(6);
    if western_total > 533 {
        western_total = western_total - 11;
    } else {
        western_total = western_total + 4;
    }
    return western_total;
}

flow effect_row_escape_extra_zone_western_prepare(seed: i32) -> i32 ![]
{
    var galaxy_prepare_total = seed + 21;
    var galaxy_prepare_cursor = 0;
    while galaxy_prepare_cursor < 11 limit Iterations(11) {
        galaxy_prepare_total = galaxy_prepare_total + galaxy_prepare_cursor + 3;
        galaxy_prepare_cursor = galaxy_prepare_cursor + 1;
    }
    if galaxy_prepare_total % 2 == 0 {
        galaxy_prepare_total = galaxy_prepare_total + effect_row_escape_extra_zone_western_score(1);
    } else {
        galaxy_prepare_total = galaxy_prepare_total - 4;
    }
    var galaxy_prepare_left = galaxy_prepare_total + seed;
    var galaxy_prepare_right = galaxy_prepare_left * 3;
    var galaxy_prepare_merged = galaxy_prepare_right - galaxy_prepare_left;
    if galaxy_prepare_merged > 28 {
        galaxy_prepare_total = galaxy_prepare_total + galaxy_prepare_merged;
    }
    return galaxy_prepare_total;
}

flow effect_row_escape_extra_zone_western_route(seed: i32) -> i32 ![]
{
    var galaxy_route_total = seed * 21;
    var galaxy_route_cursor = 0;
    while galaxy_route_cursor < 8 limit Iterations(8) {
        galaxy_route_total = galaxy_route_total + galaxy_route_cursor + 3;
        galaxy_route_cursor = galaxy_route_cursor + 1;
    }
    if galaxy_route_total % 2 == 0 {
        galaxy_route_total = galaxy_route_total + 15;
    } else {
        galaxy_route_total = galaxy_route_total - 4;
    }
    var galaxy_route_left = galaxy_route_total + seed;
    var galaxy_route_right = galaxy_route_left * 3;
    var galaxy_route_merged = galaxy_route_right - galaxy_route_left;
    if galaxy_route_merged > 28 {
        galaxy_route_total = galaxy_route_total + galaxy_route_merged;
    }
    return galaxy_route_total;
}

flow effect_row_escape_extra_zone_western_score(seed: i32) -> i32 ![]
{
    var galaxy_score_total = seed + 21;
    var galaxy_score_cursor = 0;
    while galaxy_score_cursor < 9 limit Iterations(9) {
        galaxy_score_total = galaxy_score_total + galaxy_score_cursor + 3;
        galaxy_score_cursor = galaxy_score_cursor + 1;
    }
    if galaxy_score_total % 2 == 0 {
        galaxy_score_total = galaxy_score_total + 15;
    } else {
        galaxy_score_total = galaxy_score_total - 4;
    }
    var galaxy_score_left = galaxy_score_total + seed;
    var galaxy_score_right = galaxy_score_left * 3;
    var galaxy_score_merged = galaxy_score_right - galaxy_score_left;
    if galaxy_score_merged > 28 {
        galaxy_score_total = galaxy_score_total + galaxy_score_merged;
    }
    return galaxy_score_total;
}

flow effect_row_escape_extra_zone_western_finish(seed: i32) -> i32 ![]
{
    var galaxy_finish_total = seed - 21;
    var galaxy_finish_cursor = 0;
    while galaxy_finish_cursor < 10 limit Iterations(10) {
        galaxy_finish_total = galaxy_finish_total + galaxy_finish_cursor + 3;
        galaxy_finish_cursor = galaxy_finish_cursor + 1;
    }
    if galaxy_finish_total % 2 == 0 {
        galaxy_finish_total = galaxy_finish_total + 15;
    } else {
        galaxy_finish_total = galaxy_finish_total - 4;
    }
    var galaxy_finish_left = galaxy_finish_total + seed;
    var galaxy_finish_right = galaxy_finish_left * 3;
    var galaxy_finish_merged = galaxy_finish_right - galaxy_finish_left;
    if galaxy_finish_merged > 28 {
        galaxy_finish_total = galaxy_finish_total + galaxy_finish_merged;
    }
    return galaxy_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var western_seed = 10;
    if args.len() > 0 {
        western_seed = western_seed + 1;
    } else {
        western_seed = western_seed + 2;
    }
    let western_result = effect_row_escape_extra_zone_western_entry(western_seed);
    if western_result > 0 {
        return 0;
    }
    return 1;
}
