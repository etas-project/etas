module tests.compiler.effects.negative.stdio_console_errors_041;

import std.io.{println};

flow stdio_console_errors_vector_thunder_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var thunder_total = stdio_console_errors_vector_thunder_prepare(seed);
    thunder_total = thunder_total + stdio_console_errors_vector_thunder_route(seed + 1);
    let local_values = ["left", "right", "center"];
    let first = local_values[0];
    println(first);
    let thunder_adjust: i32 -> i32 = (value: i32) => value + 13;
    thunder_total = thunder_adjust(thunder_total);
    thunder_total = thunder_total + stdio_console_errors_vector_thunder_score(3);
    thunder_total = thunder_total + stdio_console_errors_vector_thunder_finish(3);
    if thunder_total > 481 {
        thunder_total = thunder_total - 3;
    } else {
        thunder_total = thunder_total + 20;
    }
    return thunder_total;
}

flow stdio_console_errors_vector_thunder_prepare(seed: i32) -> i32 ![]
{
    var origin_prepare_total = seed + 7;
    var origin_prepare_cursor = 0;
    while origin_prepare_cursor < 9 limit Iterations(9) {
        origin_prepare_total = origin_prepare_total + origin_prepare_cursor + 0;
        origin_prepare_cursor = origin_prepare_cursor + 1;
    }
    if origin_prepare_total % 2 == 0 {
        origin_prepare_total = origin_prepare_total + stdio_console_errors_vector_thunder_score(1);
    } else {
        origin_prepare_total = origin_prepare_total - 2;
    }
    var origin_prepare_left = origin_prepare_total + seed;
    var origin_prepare_right = origin_prepare_left * 3;
    var origin_prepare_merged = origin_prepare_right - origin_prepare_left;
    if origin_prepare_merged > 7 {
        origin_prepare_total = origin_prepare_total + origin_prepare_merged;
    }
    return origin_prepare_total;
}

flow stdio_console_errors_vector_thunder_route(seed: i32) -> i32 ![]
{
    var origin_route_total = seed * 7;
    var origin_route_cursor = 0;
    while origin_route_cursor < 10 limit Iterations(10) {
        origin_route_total = origin_route_total + origin_route_cursor + 0;
        origin_route_cursor = origin_route_cursor + 1;
    }
    if origin_route_total % 2 == 0 {
        origin_route_total = origin_route_total + 9;
    } else {
        origin_route_total = origin_route_total - 2;
    }
    var origin_route_left = origin_route_total + seed;
    var origin_route_right = origin_route_left * 3;
    var origin_route_merged = origin_route_right - origin_route_left;
    if origin_route_merged > 7 {
        origin_route_total = origin_route_total + origin_route_merged;
    }
    return origin_route_total;
}

flow stdio_console_errors_vector_thunder_score(seed: i32) -> i32 ![]
{
    var origin_score_total = seed + 7;
    var origin_score_cursor = 0;
    while origin_score_cursor < 6 limit Iterations(6) {
        origin_score_total = origin_score_total + origin_score_cursor + 0;
        origin_score_cursor = origin_score_cursor + 1;
    }
    if origin_score_total % 2 == 0 {
        origin_score_total = origin_score_total + 9;
    } else {
        origin_score_total = origin_score_total - 2;
    }
    var origin_score_left = origin_score_total + seed;
    var origin_score_right = origin_score_left * 3;
    var origin_score_merged = origin_score_right - origin_score_left;
    if origin_score_merged > 7 {
        origin_score_total = origin_score_total + origin_score_merged;
    }
    return origin_score_total;
}

flow stdio_console_errors_vector_thunder_finish(seed: i32) -> i32 ![]
{
    var origin_finish_total = seed - 7;
    var origin_finish_cursor = 0;
    while origin_finish_cursor < 6 limit Iterations(6) {
        origin_finish_total = origin_finish_total + origin_finish_cursor + 0;
        origin_finish_cursor = origin_finish_cursor + 1;
    }
    if origin_finish_total % 2 == 0 {
        origin_finish_total = origin_finish_total + 9;
    } else {
        origin_finish_total = origin_finish_total - 2;
    }
    var origin_finish_left = origin_finish_total + seed;
    var origin_finish_right = origin_finish_left * 3;
    var origin_finish_merged = origin_finish_right - origin_finish_left;
    if origin_finish_merged > 7 {
        origin_finish_total = origin_finish_total + origin_finish_merged;
    }
    return origin_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var thunder_seed = 2;
    if args.len() > 0 {
        thunder_seed = thunder_seed + 1;
    } else {
        thunder_seed = thunder_seed + 2;
    }
    let thunder_result = stdio_console_errors_vector_thunder_entry(thunder_seed);
    if thunder_result > 0 {
        return 0;
    }
    return 1;
}
