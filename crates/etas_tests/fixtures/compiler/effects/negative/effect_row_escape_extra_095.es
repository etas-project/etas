module tests.compiler.effects.negative.effect_row_escape_extra_095;

import std.io.{println};

flow effect_row_escape_extra_bravo_zone_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var zone_total = effect_row_escape_extra_bravo_zone_prepare(seed);
    zone_total = zone_total + effect_row_escape_extra_bravo_zone_route(seed + 1);
    let local_values = ["left", "right", "center"];
    let first = local_values[0];
    println(first);
    let zone_adjust: i32 -> i32 = (value: i32) => value + 2;
    zone_total = zone_adjust(zone_total);
    zone_total = zone_total + effect_row_escape_extra_bravo_zone_score(2);
    zone_total = zone_total + effect_row_escape_extra_bravo_zone_finish(8);
    if zone_total > 535 {
        zone_total = zone_total - 2;
    } else {
        zone_total = zone_total + 6;
    }
    return zone_total;
}

flow effect_row_escape_extra_bravo_zone_prepare(seed: i32) -> i32 ![]
{
    var voyage_prepare_total = seed + 4;
    var voyage_prepare_cursor = 0;
    while voyage_prepare_cursor < 8 limit Iterations(8) {
        voyage_prepare_total = voyage_prepare_total + voyage_prepare_cursor + 5;
        voyage_prepare_cursor = voyage_prepare_cursor + 1;
    }
    if voyage_prepare_total % 2 == 0 {
        voyage_prepare_total = voyage_prepare_total + effect_row_escape_extra_bravo_zone_score(1);
    } else {
        voyage_prepare_total = voyage_prepare_total - 1;
    }
    var voyage_prepare_left = voyage_prepare_total + seed;
    var voyage_prepare_right = voyage_prepare_left * 5;
    var voyage_prepare_merged = voyage_prepare_right - voyage_prepare_left;
    if voyage_prepare_merged > 30 {
        voyage_prepare_total = voyage_prepare_total + voyage_prepare_merged;
    }
    return voyage_prepare_total;
}

flow effect_row_escape_extra_bravo_zone_route(seed: i32) -> i32 ![]
{
    var voyage_route_total = seed * 4;
    var voyage_route_cursor = 0;
    while voyage_route_cursor < 10 limit Iterations(10) {
        voyage_route_total = voyage_route_total + voyage_route_cursor + 5;
        voyage_route_cursor = voyage_route_cursor + 1;
    }
    if voyage_route_total % 2 == 0 {
        voyage_route_total = voyage_route_total + 17;
    } else {
        voyage_route_total = voyage_route_total - 1;
    }
    var voyage_route_left = voyage_route_total + seed;
    var voyage_route_right = voyage_route_left * 5;
    var voyage_route_merged = voyage_route_right - voyage_route_left;
    if voyage_route_merged > 30 {
        voyage_route_total = voyage_route_total + voyage_route_merged;
    }
    return voyage_route_total;
}

flow effect_row_escape_extra_bravo_zone_score(seed: i32) -> i32 ![]
{
    var voyage_score_total = seed + 4;
    var voyage_score_cursor = 0;
    while voyage_score_cursor < 11 limit Iterations(11) {
        voyage_score_total = voyage_score_total + voyage_score_cursor + 5;
        voyage_score_cursor = voyage_score_cursor + 1;
    }
    if voyage_score_total % 2 == 0 {
        voyage_score_total = voyage_score_total + 17;
    } else {
        voyage_score_total = voyage_score_total - 1;
    }
    var voyage_score_left = voyage_score_total + seed;
    var voyage_score_right = voyage_score_left * 5;
    var voyage_score_merged = voyage_score_right - voyage_score_left;
    if voyage_score_merged > 30 {
        voyage_score_total = voyage_score_total + voyage_score_merged;
    }
    return voyage_score_total;
}

flow effect_row_escape_extra_bravo_zone_finish(seed: i32) -> i32 ![]
{
    var voyage_finish_total = seed - 4;
    var voyage_finish_cursor = 0;
    while voyage_finish_cursor < 12 limit Iterations(12) {
        voyage_finish_total = voyage_finish_total + voyage_finish_cursor + 5;
        voyage_finish_cursor = voyage_finish_cursor + 1;
    }
    if voyage_finish_total % 2 == 0 {
        voyage_finish_total = voyage_finish_total + 17;
    } else {
        voyage_finish_total = voyage_finish_total - 1;
    }
    var voyage_finish_left = voyage_finish_total + seed;
    var voyage_finish_right = voyage_finish_left * 5;
    var voyage_finish_merged = voyage_finish_right - voyage_finish_left;
    if voyage_finish_merged > 30 {
        voyage_finish_total = voyage_finish_total + voyage_finish_merged;
    }
    return voyage_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var zone_seed = 1;
    if args.len() > 0 {
        zone_seed = zone_seed + 1;
    } else {
        zone_seed = zone_seed + 2;
    }
    let zone_result = effect_row_escape_extra_bravo_zone_entry(zone_seed);
    if zone_result > 0 {
        return 0;
    }
    return 1;
}
