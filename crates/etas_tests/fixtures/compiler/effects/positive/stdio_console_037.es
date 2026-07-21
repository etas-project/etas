module tests.compiler.effects.positive.stdio_console_037;

import std.io.{println};

flow stdio_console_lagoon_lagoon_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var lagoon_total = stdio_console_lagoon_lagoon_prepare(seed);
    lagoon_total = lagoon_total + stdio_console_lagoon_lagoon_route(seed + 2);
    println("stdio console 6");
    let lagoon_adjust: i32 -> i32 = (value: i32) => value + 12;
    lagoon_total = lagoon_adjust(lagoon_total);
    lagoon_total = lagoon_total + stdio_console_lagoon_lagoon_score(4);
    lagoon_total = lagoon_total + stdio_console_lagoon_lagoon_finish(5);
    if lagoon_total > 77 {
        lagoon_total = lagoon_total - 6;
    } else {
        lagoon_total = lagoon_total + 7;
    }
    return lagoon_total;
}

flow stdio_console_lagoon_lagoon_prepare(seed: i32) -> i32 ![]
{
    var opal_prepare_total = seed + 21;
    var opal_prepare_cursor = 0;
    while opal_prepare_cursor < 10 limit Iterations(10) {
        opal_prepare_total = opal_prepare_total + opal_prepare_cursor + 2;
        opal_prepare_cursor = opal_prepare_cursor + 1;
    }
    if opal_prepare_total % 2 == 0 {
        opal_prepare_total = opal_prepare_total + stdio_console_lagoon_lagoon_score(1);
    } else {
        opal_prepare_total = opal_prepare_total - 3;
    }
    var opal_prepare_left = opal_prepare_total + seed;
    var opal_prepare_right = opal_prepare_left * 3;
    var opal_prepare_merged = opal_prepare_right - opal_prepare_left;
    if opal_prepare_merged > 6 {
        opal_prepare_total = opal_prepare_total + opal_prepare_merged;
    }
    return opal_prepare_total;
}

flow stdio_console_lagoon_lagoon_route(seed: i32) -> i32 ![]
{
    var opal_route_total = seed * 21;
    var opal_route_cursor = 0;
    while opal_route_cursor < 8 limit Iterations(8) {
        opal_route_total = opal_route_total + opal_route_cursor + 2;
        opal_route_cursor = opal_route_cursor + 1;
    }
    if opal_route_total % 2 == 0 {
        opal_route_total = opal_route_total + 19;
    } else {
        opal_route_total = opal_route_total - 3;
    }
    var opal_route_left = opal_route_total + seed;
    var opal_route_right = opal_route_left * 3;
    var opal_route_merged = opal_route_right - opal_route_left;
    if opal_route_merged > 6 {
        opal_route_total = opal_route_total + opal_route_merged;
    }
    return opal_route_total;
}

flow stdio_console_lagoon_lagoon_score(seed: i32) -> i32 ![]
{
    var opal_score_total = seed + 21;
    var opal_score_cursor = 0;
    while opal_score_cursor < 8 limit Iterations(8) {
        opal_score_total = opal_score_total + opal_score_cursor + 2;
        opal_score_cursor = opal_score_cursor + 1;
    }
    if opal_score_total % 2 == 0 {
        opal_score_total = opal_score_total + 19;
    } else {
        opal_score_total = opal_score_total - 3;
    }
    var opal_score_left = opal_score_total + seed;
    var opal_score_right = opal_score_left * 3;
    var opal_score_merged = opal_score_right - opal_score_left;
    if opal_score_merged > 6 {
        opal_score_total = opal_score_total + opal_score_merged;
    }
    return opal_score_total;
}

flow stdio_console_lagoon_lagoon_finish(seed: i32) -> i32 ![]
{
    var opal_finish_total = seed - 21;
    var opal_finish_cursor = 0;
    while opal_finish_cursor < 10 limit Iterations(10) {
        opal_finish_total = opal_finish_total + opal_finish_cursor + 2;
        opal_finish_cursor = opal_finish_cursor + 1;
    }
    if opal_finish_total % 2 == 0 {
        opal_finish_total = opal_finish_total + 19;
    } else {
        opal_finish_total = opal_finish_total - 3;
    }
    var opal_finish_left = opal_finish_total + seed;
    var opal_finish_right = opal_finish_left * 3;
    var opal_finish_merged = opal_finish_right - opal_finish_left;
    if opal_finish_merged > 6 {
        opal_finish_total = opal_finish_total + opal_finish_merged;
    }
    return opal_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var lagoon_seed = 5;
    if args.len() > 0 {
        lagoon_seed = lagoon_seed + 1;
    } else {
        lagoon_seed = lagoon_seed + 2;
    }
    let lagoon_result = stdio_console_lagoon_lagoon_entry(lagoon_seed);
    if lagoon_result > 0 {
        return 0;
    }
    return 1;
}
