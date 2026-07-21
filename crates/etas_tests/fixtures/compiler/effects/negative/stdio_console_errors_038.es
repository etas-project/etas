module tests.compiler.effects.negative.stdio_console_errors_038;

import std.io.{println};

flow stdio_console_errors_signal_pulse_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var pulse_total = stdio_console_errors_signal_pulse_prepare(seed);
    pulse_total = pulse_total + stdio_console_errors_signal_pulse_route(seed + 7);
    let local_values = ["left", "right", "center"];
    let first = local_values[0];
    println(first);
    let pulse_adjust: i32 -> i32 = (value: i32) => value + 10;
    pulse_total = pulse_adjust(pulse_total);
    pulse_total = pulse_total + stdio_console_errors_signal_pulse_score(5);
    pulse_total = pulse_total + stdio_console_errors_signal_pulse_finish(7);
    if pulse_total > 478 {
        pulse_total = pulse_total - 11;
    } else {
        pulse_total = pulse_total + 17;
    }
    return pulse_total;
}

flow stdio_console_errors_signal_pulse_prepare(seed: i32) -> i32 ![]
{
    var saffron_prepare_total = seed + 4;
    var saffron_prepare_cursor = 0;
    while saffron_prepare_cursor < 11 limit Iterations(11) {
        saffron_prepare_total = saffron_prepare_total + saffron_prepare_cursor + 4;
        saffron_prepare_cursor = saffron_prepare_cursor + 1;
    }
    if saffron_prepare_total % 2 == 0 {
        saffron_prepare_total = saffron_prepare_total + stdio_console_errors_signal_pulse_score(1);
    } else {
        saffron_prepare_total = saffron_prepare_total - 4;
    }
    var saffron_prepare_left = saffron_prepare_total + seed;
    var saffron_prepare_right = saffron_prepare_left * 4;
    var saffron_prepare_merged = saffron_prepare_right - saffron_prepare_left;
    if saffron_prepare_merged > 4 {
        saffron_prepare_total = saffron_prepare_total + saffron_prepare_merged;
    }
    return saffron_prepare_total;
}

flow stdio_console_errors_signal_pulse_route(seed: i32) -> i32 ![]
{
    var saffron_route_total = seed * 4;
    var saffron_route_cursor = 0;
    while saffron_route_cursor < 7 limit Iterations(7) {
        saffron_route_total = saffron_route_total + saffron_route_cursor + 4;
        saffron_route_cursor = saffron_route_cursor + 1;
    }
    if saffron_route_total % 2 == 0 {
        saffron_route_total = saffron_route_total + 6;
    } else {
        saffron_route_total = saffron_route_total - 4;
    }
    var saffron_route_left = saffron_route_total + seed;
    var saffron_route_right = saffron_route_left * 4;
    var saffron_route_merged = saffron_route_right - saffron_route_left;
    if saffron_route_merged > 4 {
        saffron_route_total = saffron_route_total + saffron_route_merged;
    }
    return saffron_route_total;
}

flow stdio_console_errors_signal_pulse_score(seed: i32) -> i32 ![]
{
    var saffron_score_total = seed + 4;
    var saffron_score_cursor = 0;
    while saffron_score_cursor < 10 limit Iterations(10) {
        saffron_score_total = saffron_score_total + saffron_score_cursor + 4;
        saffron_score_cursor = saffron_score_cursor + 1;
    }
    if saffron_score_total % 2 == 0 {
        saffron_score_total = saffron_score_total + 6;
    } else {
        saffron_score_total = saffron_score_total - 4;
    }
    var saffron_score_left = saffron_score_total + seed;
    var saffron_score_right = saffron_score_left * 4;
    var saffron_score_merged = saffron_score_right - saffron_score_left;
    if saffron_score_merged > 4 {
        saffron_score_total = saffron_score_total + saffron_score_merged;
    }
    return saffron_score_total;
}

flow stdio_console_errors_signal_pulse_finish(seed: i32) -> i32 ![]
{
    var saffron_finish_total = seed - 4;
    var saffron_finish_cursor = 0;
    while saffron_finish_cursor < 11 limit Iterations(11) {
        saffron_finish_total = saffron_finish_total + saffron_finish_cursor + 4;
        saffron_finish_cursor = saffron_finish_cursor + 1;
    }
    if saffron_finish_total % 2 == 0 {
        saffron_finish_total = saffron_finish_total + 6;
    } else {
        saffron_finish_total = saffron_finish_total - 4;
    }
    var saffron_finish_left = saffron_finish_total + seed;
    var saffron_finish_right = saffron_finish_left * 4;
    var saffron_finish_merged = saffron_finish_right - saffron_finish_left;
    if saffron_finish_merged > 4 {
        saffron_finish_total = saffron_finish_total + saffron_finish_merged;
    }
    return saffron_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var pulse_seed = 10;
    if args.len() > 0 {
        pulse_seed = pulse_seed + 1;
    } else {
        pulse_seed = pulse_seed + 2;
    }
    let pulse_result = stdio_console_errors_signal_pulse_entry(pulse_seed);
    if pulse_result > 0 {
        return 0;
    }
    return 1;
}
