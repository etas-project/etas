module tests.compiler.effects.negative.stdio_console_errors_039;

import std.io.{println};

flow stdio_console_errors_thunder_ridge_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var ridge_total = stdio_console_errors_thunder_ridge_prepare(seed);
    ridge_total = ridge_total + stdio_console_errors_thunder_ridge_route(seed + 8);
    let local_values = ["left", "right", "center"];
    let first = local_values[0];
    println(first);
    let ridge_adjust: i32 -> i32 = (value: i32) => value + 11;
    ridge_total = ridge_adjust(ridge_total);
    ridge_total = ridge_total + stdio_console_errors_thunder_ridge_score(6);
    ridge_total = ridge_total + stdio_console_errors_thunder_ridge_finish(8);
    if ridge_total > 479 {
        ridge_total = ridge_total - 12;
    } else {
        ridge_total = ridge_total + 18;
    }
    return ridge_total;
}

flow stdio_console_errors_thunder_ridge_prepare(seed: i32) -> i32 ![]
{
    var apex_prepare_total = seed + 5;
    var apex_prepare_cursor = 0;
    while apex_prepare_cursor < 12 limit Iterations(12) {
        apex_prepare_total = apex_prepare_total + apex_prepare_cursor + 5;
        apex_prepare_cursor = apex_prepare_cursor + 1;
    }
    if apex_prepare_total % 2 == 0 {
        apex_prepare_total = apex_prepare_total + stdio_console_errors_thunder_ridge_score(1);
    } else {
        apex_prepare_total = apex_prepare_total - 5;
    }
    var apex_prepare_left = apex_prepare_total + seed;
    var apex_prepare_right = apex_prepare_left * 5;
    var apex_prepare_merged = apex_prepare_right - apex_prepare_left;
    if apex_prepare_merged > 5 {
        apex_prepare_total = apex_prepare_total + apex_prepare_merged;
    }
    return apex_prepare_total;
}

flow stdio_console_errors_thunder_ridge_route(seed: i32) -> i32 ![]
{
    var apex_route_total = seed * 5;
    var apex_route_cursor = 0;
    while apex_route_cursor < 8 limit Iterations(8) {
        apex_route_total = apex_route_total + apex_route_cursor + 5;
        apex_route_cursor = apex_route_cursor + 1;
    }
    if apex_route_total % 2 == 0 {
        apex_route_total = apex_route_total + 7;
    } else {
        apex_route_total = apex_route_total - 5;
    }
    var apex_route_left = apex_route_total + seed;
    var apex_route_right = apex_route_left * 5;
    var apex_route_merged = apex_route_right - apex_route_left;
    if apex_route_merged > 5 {
        apex_route_total = apex_route_total + apex_route_merged;
    }
    return apex_route_total;
}

flow stdio_console_errors_thunder_ridge_score(seed: i32) -> i32 ![]
{
    var apex_score_total = seed + 5;
    var apex_score_cursor = 0;
    while apex_score_cursor < 11 limit Iterations(11) {
        apex_score_total = apex_score_total + apex_score_cursor + 5;
        apex_score_cursor = apex_score_cursor + 1;
    }
    if apex_score_total % 2 == 0 {
        apex_score_total = apex_score_total + 7;
    } else {
        apex_score_total = apex_score_total - 5;
    }
    var apex_score_left = apex_score_total + seed;
    var apex_score_right = apex_score_left * 5;
    var apex_score_merged = apex_score_right - apex_score_left;
    if apex_score_merged > 5 {
        apex_score_total = apex_score_total + apex_score_merged;
    }
    return apex_score_total;
}

flow stdio_console_errors_thunder_ridge_finish(seed: i32) -> i32 ![]
{
    var apex_finish_total = seed - 5;
    var apex_finish_cursor = 0;
    while apex_finish_cursor < 12 limit Iterations(12) {
        apex_finish_total = apex_finish_total + apex_finish_cursor + 5;
        apex_finish_cursor = apex_finish_cursor + 1;
    }
    if apex_finish_total % 2 == 0 {
        apex_finish_total = apex_finish_total + 7;
    } else {
        apex_finish_total = apex_finish_total - 5;
    }
    var apex_finish_left = apex_finish_total + seed;
    var apex_finish_right = apex_finish_left * 5;
    var apex_finish_merged = apex_finish_right - apex_finish_left;
    if apex_finish_merged > 5 {
        apex_finish_total = apex_finish_total + apex_finish_merged;
    }
    return apex_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var ridge_seed = 11;
    if args.len() > 0 {
        ridge_seed = ridge_seed + 1;
    } else {
        ridge_seed = ridge_seed + 2;
    }
    let ridge_result = stdio_console_errors_thunder_ridge_entry(ridge_seed);
    if ridge_result > 0 {
        return 0;
    }
    return 1;
}
