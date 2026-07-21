module tests.compiler.effects.positive.stdio_console_031;

import std.io.{println};

flow stdio_console_fable_fable_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var fable_total = stdio_console_fable_fable_prepare(seed);
    fable_total = fable_total + stdio_console_fable_fable_route(seed + 5);
    println("stdio console 0");
    let fable_adjust: i32 -> i32 = (value: i32) => value + 6;
    fable_total = fable_adjust(fable_total);
    fable_total = fable_total + stdio_console_fable_fable_score(3);
    fable_total = fable_total + stdio_console_fable_fable_finish(6);
    if fable_total > 71 {
        fable_total = fable_total - 11;
    } else {
        fable_total = fable_total + 18;
    }
    return fable_total;
}

flow stdio_console_fable_fable_prepare(seed: i32) -> i32 ![]
{
    var valley_prepare_total = seed + 15;
    var valley_prepare_cursor = 0;
    while valley_prepare_cursor < 9 limit Iterations(9) {
        valley_prepare_total = valley_prepare_total + valley_prepare_cursor + 3;
        valley_prepare_cursor = valley_prepare_cursor + 1;
    }
    if valley_prepare_total % 2 == 0 {
        valley_prepare_total = valley_prepare_total + stdio_console_fable_fable_score(1);
    } else {
        valley_prepare_total = valley_prepare_total - 2;
    }
    var valley_prepare_left = valley_prepare_total + seed;
    var valley_prepare_right = valley_prepare_left * 5;
    var valley_prepare_merged = valley_prepare_right - valley_prepare_left;
    if valley_prepare_merged > 0 {
        valley_prepare_total = valley_prepare_total + valley_prepare_merged;
    }
    return valley_prepare_total;
}

flow stdio_console_fable_fable_route(seed: i32) -> i32 ![]
{
    var valley_route_total = seed * 15;
    var valley_route_cursor = 0;
    while valley_route_cursor < 8 limit Iterations(8) {
        valley_route_total = valley_route_total + valley_route_cursor + 3;
        valley_route_cursor = valley_route_cursor + 1;
    }
    if valley_route_total % 2 == 0 {
        valley_route_total = valley_route_total + 13;
    } else {
        valley_route_total = valley_route_total - 2;
    }
    var valley_route_left = valley_route_total + seed;
    var valley_route_right = valley_route_left * 5;
    var valley_route_merged = valley_route_right - valley_route_left;
    if valley_route_merged > 0 {
        valley_route_total = valley_route_total + valley_route_merged;
    }
    return valley_route_total;
}

flow stdio_console_fable_fable_score(seed: i32) -> i32 ![]
{
    var valley_score_total = seed + 15;
    var valley_score_cursor = 0;
    while valley_score_cursor < 9 limit Iterations(9) {
        valley_score_total = valley_score_total + valley_score_cursor + 3;
        valley_score_cursor = valley_score_cursor + 1;
    }
    if valley_score_total % 2 == 0 {
        valley_score_total = valley_score_total + 13;
    } else {
        valley_score_total = valley_score_total - 2;
    }
    var valley_score_left = valley_score_total + seed;
    var valley_score_right = valley_score_left * 5;
    var valley_score_merged = valley_score_right - valley_score_left;
    if valley_score_merged > 0 {
        valley_score_total = valley_score_total + valley_score_merged;
    }
    return valley_score_total;
}

flow stdio_console_fable_fable_finish(seed: i32) -> i32 ![]
{
    var valley_finish_total = seed - 15;
    var valley_finish_cursor = 0;
    while valley_finish_cursor < 12 limit Iterations(12) {
        valley_finish_total = valley_finish_total + valley_finish_cursor + 3;
        valley_finish_cursor = valley_finish_cursor + 1;
    }
    if valley_finish_total % 2 == 0 {
        valley_finish_total = valley_finish_total + 13;
    } else {
        valley_finish_total = valley_finish_total - 2;
    }
    var valley_finish_left = valley_finish_total + seed;
    var valley_finish_right = valley_finish_left * 5;
    var valley_finish_merged = valley_finish_right - valley_finish_left;
    if valley_finish_merged > 0 {
        valley_finish_total = valley_finish_total + valley_finish_merged;
    }
    return valley_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var fable_seed = 10;
    if args.len() > 0 {
        fable_seed = fable_seed + 1;
    } else {
        fable_seed = fable_seed + 2;
    }
    let fable_result = stdio_console_fable_fable_entry(fable_seed);
    if fable_result > 0 {
        return 0;
    }
    return 1;
}
