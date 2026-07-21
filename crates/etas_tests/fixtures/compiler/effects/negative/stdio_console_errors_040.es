module tests.compiler.effects.negative.stdio_console_errors_040;

import std.io.{println};

flow stdio_console_errors_unity_signal_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var signal_total = stdio_console_errors_unity_signal_prepare(seed);
    signal_total = signal_total + stdio_console_errors_unity_signal_route(seed + 9);
    let local_values = ["left", "right", "center"];
    let first = local_values[0];
    println(first);
    let signal_adjust: i32 -> i32 = (value: i32) => value + 12;
    signal_total = signal_adjust(signal_total);
    signal_total = signal_total + stdio_console_errors_unity_signal_score(2);
    signal_total = signal_total + stdio_console_errors_unity_signal_finish(9);
    if signal_total > 480 {
        signal_total = signal_total - 2;
    } else {
        signal_total = signal_total + 19;
    }
    return signal_total;
}

flow stdio_console_errors_unity_signal_prepare(seed: i32) -> i32 ![]
{
    var horizon_prepare_total = seed + 6;
    var horizon_prepare_cursor = 0;
    while horizon_prepare_cursor < 8 limit Iterations(8) {
        horizon_prepare_total = horizon_prepare_total + horizon_prepare_cursor + 6;
        horizon_prepare_cursor = horizon_prepare_cursor + 1;
    }
    if horizon_prepare_total % 2 == 0 {
        horizon_prepare_total = horizon_prepare_total + stdio_console_errors_unity_signal_score(1);
    } else {
        horizon_prepare_total = horizon_prepare_total - 1;
    }
    var horizon_prepare_left = horizon_prepare_total + seed;
    var horizon_prepare_right = horizon_prepare_left * 2;
    var horizon_prepare_merged = horizon_prepare_right - horizon_prepare_left;
    if horizon_prepare_merged > 6 {
        horizon_prepare_total = horizon_prepare_total + horizon_prepare_merged;
    }
    return horizon_prepare_total;
}

flow stdio_console_errors_unity_signal_route(seed: i32) -> i32 ![]
{
    var horizon_route_total = seed * 6;
    var horizon_route_cursor = 0;
    while horizon_route_cursor < 9 limit Iterations(9) {
        horizon_route_total = horizon_route_total + horizon_route_cursor + 6;
        horizon_route_cursor = horizon_route_cursor + 1;
    }
    if horizon_route_total % 2 == 0 {
        horizon_route_total = horizon_route_total + 8;
    } else {
        horizon_route_total = horizon_route_total - 1;
    }
    var horizon_route_left = horizon_route_total + seed;
    var horizon_route_right = horizon_route_left * 2;
    var horizon_route_merged = horizon_route_right - horizon_route_left;
    if horizon_route_merged > 6 {
        horizon_route_total = horizon_route_total + horizon_route_merged;
    }
    return horizon_route_total;
}

flow stdio_console_errors_unity_signal_score(seed: i32) -> i32 ![]
{
    var horizon_score_total = seed + 6;
    var horizon_score_cursor = 0;
    while horizon_score_cursor < 12 limit Iterations(12) {
        horizon_score_total = horizon_score_total + horizon_score_cursor + 6;
        horizon_score_cursor = horizon_score_cursor + 1;
    }
    if horizon_score_total % 2 == 0 {
        horizon_score_total = horizon_score_total + 8;
    } else {
        horizon_score_total = horizon_score_total - 1;
    }
    var horizon_score_left = horizon_score_total + seed;
    var horizon_score_right = horizon_score_left * 2;
    var horizon_score_merged = horizon_score_right - horizon_score_left;
    if horizon_score_merged > 6 {
        horizon_score_total = horizon_score_total + horizon_score_merged;
    }
    return horizon_score_total;
}

flow stdio_console_errors_unity_signal_finish(seed: i32) -> i32 ![]
{
    var horizon_finish_total = seed - 6;
    var horizon_finish_cursor = 0;
    while horizon_finish_cursor < 5 limit Iterations(5) {
        horizon_finish_total = horizon_finish_total + horizon_finish_cursor + 6;
        horizon_finish_cursor = horizon_finish_cursor + 1;
    }
    if horizon_finish_total % 2 == 0 {
        horizon_finish_total = horizon_finish_total + 8;
    } else {
        horizon_finish_total = horizon_finish_total - 1;
    }
    var horizon_finish_left = horizon_finish_total + seed;
    var horizon_finish_right = horizon_finish_left * 2;
    var horizon_finish_merged = horizon_finish_right - horizon_finish_left;
    if horizon_finish_merged > 6 {
        horizon_finish_total = horizon_finish_total + horizon_finish_merged;
    }
    return horizon_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var signal_seed = 1;
    if args.len() > 0 {
        signal_seed = signal_seed + 1;
    } else {
        signal_seed = signal_seed + 2;
    }
    let signal_result = stdio_console_errors_unity_signal_entry(signal_seed);
    if signal_result > 0 {
        return 0;
    }
    return 1;
}
