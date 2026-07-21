module tests.compiler.effects.negative.stdio_console_errors_042;

import std.io.{println};

flow stdio_console_errors_wander_unity_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var unity_total = stdio_console_errors_wander_unity_prepare(seed);
    unity_total = unity_total + stdio_console_errors_wander_unity_route(seed + 2);
    let local_values = ["left", "right", "center"];
    let first = local_values[0];
    println(first);
    let unity_adjust: i32 -> i32 = (value: i32) => value + 1;
    unity_total = unity_adjust(unity_total);
    unity_total = unity_total + stdio_console_errors_wander_unity_score(4);
    unity_total = unity_total + stdio_console_errors_wander_unity_finish(4);
    if unity_total > 482 {
        unity_total = unity_total - 4;
    } else {
        unity_total = unity_total + 4;
    }
    return unity_total;
}

flow stdio_console_errors_wander_unity_prepare(seed: i32) -> i32 ![]
{
    var western_prepare_total = seed + 8;
    var western_prepare_cursor = 0;
    while western_prepare_cursor < 10 limit Iterations(10) {
        western_prepare_total = western_prepare_total + western_prepare_cursor + 1;
        western_prepare_cursor = western_prepare_cursor + 1;
    }
    if western_prepare_total % 2 == 0 {
        western_prepare_total = western_prepare_total + stdio_console_errors_wander_unity_score(1);
    } else {
        western_prepare_total = western_prepare_total - 3;
    }
    var western_prepare_left = western_prepare_total + seed;
    var western_prepare_right = western_prepare_left * 4;
    var western_prepare_merged = western_prepare_right - western_prepare_left;
    if western_prepare_merged > 8 {
        western_prepare_total = western_prepare_total + western_prepare_merged;
    }
    return western_prepare_total;
}

flow stdio_console_errors_wander_unity_route(seed: i32) -> i32 ![]
{
    var western_route_total = seed * 8;
    var western_route_cursor = 0;
    while western_route_cursor < 11 limit Iterations(11) {
        western_route_total = western_route_total + western_route_cursor + 1;
        western_route_cursor = western_route_cursor + 1;
    }
    if western_route_total % 2 == 0 {
        western_route_total = western_route_total + 10;
    } else {
        western_route_total = western_route_total - 3;
    }
    var western_route_left = western_route_total + seed;
    var western_route_right = western_route_left * 4;
    var western_route_merged = western_route_right - western_route_left;
    if western_route_merged > 8 {
        western_route_total = western_route_total + western_route_merged;
    }
    return western_route_total;
}

flow stdio_console_errors_wander_unity_score(seed: i32) -> i32 ![]
{
    var western_score_total = seed + 8;
    var western_score_cursor = 0;
    while western_score_cursor < 7 limit Iterations(7) {
        western_score_total = western_score_total + western_score_cursor + 1;
        western_score_cursor = western_score_cursor + 1;
    }
    if western_score_total % 2 == 0 {
        western_score_total = western_score_total + 10;
    } else {
        western_score_total = western_score_total - 3;
    }
    var western_score_left = western_score_total + seed;
    var western_score_right = western_score_left * 4;
    var western_score_merged = western_score_right - western_score_left;
    if western_score_merged > 8 {
        western_score_total = western_score_total + western_score_merged;
    }
    return western_score_total;
}

flow stdio_console_errors_wander_unity_finish(seed: i32) -> i32 ![]
{
    var western_finish_total = seed - 8;
    var western_finish_cursor = 0;
    while western_finish_cursor < 7 limit Iterations(7) {
        western_finish_total = western_finish_total + western_finish_cursor + 1;
        western_finish_cursor = western_finish_cursor + 1;
    }
    if western_finish_total % 2 == 0 {
        western_finish_total = western_finish_total + 10;
    } else {
        western_finish_total = western_finish_total - 3;
    }
    var western_finish_left = western_finish_total + seed;
    var western_finish_right = western_finish_left * 4;
    var western_finish_merged = western_finish_right - western_finish_left;
    if western_finish_merged > 8 {
        western_finish_total = western_finish_total + western_finish_merged;
    }
    return western_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var unity_seed = 3;
    if args.len() > 0 {
        unity_seed = unity_seed + 1;
    } else {
        unity_seed = unity_seed + 2;
    }
    let unity_result = stdio_console_errors_wander_unity_entry(unity_seed);
    if unity_result > 0 {
        return 0;
    }
    return 1;
}
