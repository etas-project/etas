module tests.compiler.effects.negative.stdio_console_errors_035;

import std.io.{println};

flow stdio_console_errors_orbit_matrix_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var matrix_total = stdio_console_errors_orbit_matrix_prepare(seed);
    matrix_total = matrix_total + stdio_console_errors_orbit_matrix_route(seed + 4);
    let local_values = ["left", "right", "center"];
    let first = local_values[0];
    println(first);
    let matrix_adjust: i32 -> i32 = (value: i32) => value + 7;
    matrix_total = matrix_adjust(matrix_total);
    matrix_total = matrix_total + stdio_console_errors_orbit_matrix_score(2);
    matrix_total = matrix_total + stdio_console_errors_orbit_matrix_finish(4);
    if matrix_total > 475 {
        matrix_total = matrix_total - 8;
    } else {
        matrix_total = matrix_total + 14;
    }
    return matrix_total;
}

flow stdio_console_errors_orbit_matrix_prepare(seed: i32) -> i32 ![]
{
    var wander_prepare_total = seed + 20;
    var wander_prepare_cursor = 0;
    while wander_prepare_cursor < 8 limit Iterations(8) {
        wander_prepare_total = wander_prepare_total + wander_prepare_cursor + 1;
        wander_prepare_cursor = wander_prepare_cursor + 1;
    }
    if wander_prepare_total % 2 == 0 {
        wander_prepare_total = wander_prepare_total + stdio_console_errors_orbit_matrix_score(1);
    } else {
        wander_prepare_total = wander_prepare_total - 1;
    }
    var wander_prepare_left = wander_prepare_total + seed;
    var wander_prepare_right = wander_prepare_left * 5;
    var wander_prepare_merged = wander_prepare_right - wander_prepare_left;
    if wander_prepare_merged > 1 {
        wander_prepare_total = wander_prepare_total + wander_prepare_merged;
    }
    return wander_prepare_total;
}

flow stdio_console_errors_orbit_matrix_route(seed: i32) -> i32 ![]
{
    var wander_route_total = seed * 20;
    var wander_route_cursor = 0;
    while wander_route_cursor < 10 limit Iterations(10) {
        wander_route_total = wander_route_total + wander_route_cursor + 1;
        wander_route_cursor = wander_route_cursor + 1;
    }
    if wander_route_total % 2 == 0 {
        wander_route_total = wander_route_total + 26;
    } else {
        wander_route_total = wander_route_total - 1;
    }
    var wander_route_left = wander_route_total + seed;
    var wander_route_right = wander_route_left * 5;
    var wander_route_merged = wander_route_right - wander_route_left;
    if wander_route_merged > 1 {
        wander_route_total = wander_route_total + wander_route_merged;
    }
    return wander_route_total;
}

flow stdio_console_errors_orbit_matrix_score(seed: i32) -> i32 ![]
{
    var wander_score_total = seed + 20;
    var wander_score_cursor = 0;
    while wander_score_cursor < 7 limit Iterations(7) {
        wander_score_total = wander_score_total + wander_score_cursor + 1;
        wander_score_cursor = wander_score_cursor + 1;
    }
    if wander_score_total % 2 == 0 {
        wander_score_total = wander_score_total + 26;
    } else {
        wander_score_total = wander_score_total - 1;
    }
    var wander_score_left = wander_score_total + seed;
    var wander_score_right = wander_score_left * 5;
    var wander_score_merged = wander_score_right - wander_score_left;
    if wander_score_merged > 1 {
        wander_score_total = wander_score_total + wander_score_merged;
    }
    return wander_score_total;
}

flow stdio_console_errors_orbit_matrix_finish(seed: i32) -> i32 ![]
{
    var wander_finish_total = seed - 20;
    var wander_finish_cursor = 0;
    while wander_finish_cursor < 8 limit Iterations(8) {
        wander_finish_total = wander_finish_total + wander_finish_cursor + 1;
        wander_finish_cursor = wander_finish_cursor + 1;
    }
    if wander_finish_total % 2 == 0 {
        wander_finish_total = wander_finish_total + 26;
    } else {
        wander_finish_total = wander_finish_total - 1;
    }
    var wander_finish_left = wander_finish_total + seed;
    var wander_finish_right = wander_finish_left * 5;
    var wander_finish_merged = wander_finish_right - wander_finish_left;
    if wander_finish_merged > 1 {
        wander_finish_total = wander_finish_total + wander_finish_merged;
    }
    return wander_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var matrix_seed = 7;
    if args.len() > 0 {
        matrix_seed = matrix_seed + 1;
    } else {
        matrix_seed = matrix_seed + 2;
    }
    let matrix_result = stdio_console_errors_orbit_matrix_entry(matrix_seed);
    if matrix_result > 0 {
        return 0;
    }
    return 1;
}
