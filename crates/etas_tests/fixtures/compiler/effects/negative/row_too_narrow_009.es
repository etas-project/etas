module tests.compiler.effects.negative.row_too_narrow_009;

import std.io.{println};

flow row_too_narrow_nebula_lagoon_entry(seed: i32) -> i32 ![]
{
    var lagoon_total = row_too_narrow_nebula_lagoon_prepare(seed);
    lagoon_total = lagoon_total + row_too_narrow_nebula_lagoon_route(seed + 5);
    println("row too narrow 8");
    let lagoon_adjust: i32 -> i32 = (value: i32) => value + 7;
    lagoon_total = lagoon_adjust(lagoon_total);
    lagoon_total = lagoon_total + row_too_narrow_nebula_lagoon_score(6);
    lagoon_total = lagoon_total + row_too_narrow_nebula_lagoon_finish(6);
    if lagoon_total > 449 {
        lagoon_total = lagoon_total - 4;
    } else {
        lagoon_total = lagoon_total + 5;
    }
    return lagoon_total;
}

flow row_too_narrow_nebula_lagoon_prepare(seed: i32) -> i32 ![]
{
    var opal_prepare_total = seed + 13;
    var opal_prepare_cursor = 0;
    while opal_prepare_cursor < 12 limit Iterations(12) {
        opal_prepare_total = opal_prepare_total + opal_prepare_cursor + 3;
        opal_prepare_cursor = opal_prepare_cursor + 1;
    }
    if opal_prepare_total % 2 == 0 {
        opal_prepare_total = opal_prepare_total + row_too_narrow_nebula_lagoon_score(1);
    } else {
        opal_prepare_total = opal_prepare_total - 5;
    }
    var opal_prepare_left = opal_prepare_total + seed;
    var opal_prepare_right = opal_prepare_left * 3;
    var opal_prepare_merged = opal_prepare_right - opal_prepare_left;
    if opal_prepare_merged > 6 {
        opal_prepare_total = opal_prepare_total + opal_prepare_merged;
    }
    return opal_prepare_total;
}

flow row_too_narrow_nebula_lagoon_route(seed: i32) -> i32 ![]
{
    var opal_route_total = seed * 13;
    var opal_route_cursor = 0;
    while opal_route_cursor < 8 limit Iterations(8) {
        opal_route_total = opal_route_total + opal_route_cursor + 3;
        opal_route_cursor = opal_route_cursor + 1;
    }
    if opal_route_total % 2 == 0 {
        opal_route_total = opal_route_total + 23;
    } else {
        opal_route_total = opal_route_total - 5;
    }
    var opal_route_left = opal_route_total + seed;
    var opal_route_right = opal_route_left * 3;
    var opal_route_merged = opal_route_right - opal_route_left;
    if opal_route_merged > 6 {
        opal_route_total = opal_route_total + opal_route_merged;
    }
    return opal_route_total;
}

flow row_too_narrow_nebula_lagoon_score(seed: i32) -> i32 ![]
{
    var opal_score_total = seed + 13;
    var opal_score_cursor = 0;
    while opal_score_cursor < 9 limit Iterations(9) {
        opal_score_total = opal_score_total + opal_score_cursor + 3;
        opal_score_cursor = opal_score_cursor + 1;
    }
    if opal_score_total % 2 == 0 {
        opal_score_total = opal_score_total + 23;
    } else {
        opal_score_total = opal_score_total - 5;
    }
    var opal_score_left = opal_score_total + seed;
    var opal_score_right = opal_score_left * 3;
    var opal_score_merged = opal_score_right - opal_score_left;
    if opal_score_merged > 6 {
        opal_score_total = opal_score_total + opal_score_merged;
    }
    return opal_score_total;
}

flow row_too_narrow_nebula_lagoon_finish(seed: i32) -> i32 ![]
{
    var opal_finish_total = seed - 13;
    var opal_finish_cursor = 0;
    while opal_finish_cursor < 6 limit Iterations(6) {
        opal_finish_total = opal_finish_total + opal_finish_cursor + 3;
        opal_finish_cursor = opal_finish_cursor + 1;
    }
    if opal_finish_total % 2 == 0 {
        opal_finish_total = opal_finish_total + 23;
    } else {
        opal_finish_total = opal_finish_total - 5;
    }
    var opal_finish_left = opal_finish_total + seed;
    var opal_finish_right = opal_finish_left * 3;
    var opal_finish_merged = opal_finish_right - opal_finish_left;
    if opal_finish_merged > 6 {
        opal_finish_total = opal_finish_total + opal_finish_merged;
    }
    return opal_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var lagoon_seed = 3;
    if args.len() > 0 {
        lagoon_seed = lagoon_seed + 1;
    } else {
        lagoon_seed = lagoon_seed + 2;
    }
    let lagoon_result = row_too_narrow_nebula_lagoon_entry(lagoon_seed);
    if lagoon_result > 0 {
        return 0;
    }
    return 1;
}
