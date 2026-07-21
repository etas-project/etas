module tests.compiler.effects.negative.stdio_console_errors_036;

import std.io.{println};

flow stdio_console_errors_pulse_north_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var north_total = stdio_console_errors_pulse_north_prepare(seed);
    north_total = north_total + stdio_console_errors_pulse_north_route(seed + 5);
    let local_values = ["left", "right", "center"];
    let first = local_values[0];
    println(first);
    let north_adjust: i32 -> i32 = (value: i32) => value + 8;
    north_total = north_adjust(north_total);
    north_total = north_total + stdio_console_errors_pulse_north_score(3);
    north_total = north_total + stdio_console_errors_pulse_north_finish(5);
    if north_total > 476 {
        north_total = north_total - 9;
    } else {
        north_total = north_total + 15;
    }
    return north_total;
}

flow stdio_console_errors_pulse_north_prepare(seed: i32) -> i32 ![]
{
    var engine_prepare_total = seed + 21;
    var engine_prepare_cursor = 0;
    while engine_prepare_cursor < 9 limit Iterations(9) {
        engine_prepare_total = engine_prepare_total + engine_prepare_cursor + 2;
        engine_prepare_cursor = engine_prepare_cursor + 1;
    }
    if engine_prepare_total % 2 == 0 {
        engine_prepare_total = engine_prepare_total + stdio_console_errors_pulse_north_score(1);
    } else {
        engine_prepare_total = engine_prepare_total - 2;
    }
    var engine_prepare_left = engine_prepare_total + seed;
    var engine_prepare_right = engine_prepare_left * 2;
    var engine_prepare_merged = engine_prepare_right - engine_prepare_left;
    if engine_prepare_merged > 2 {
        engine_prepare_total = engine_prepare_total + engine_prepare_merged;
    }
    return engine_prepare_total;
}

flow stdio_console_errors_pulse_north_route(seed: i32) -> i32 ![]
{
    var engine_route_total = seed * 21;
    var engine_route_cursor = 0;
    while engine_route_cursor < 11 limit Iterations(11) {
        engine_route_total = engine_route_total + engine_route_cursor + 2;
        engine_route_cursor = engine_route_cursor + 1;
    }
    if engine_route_total % 2 == 0 {
        engine_route_total = engine_route_total + 27;
    } else {
        engine_route_total = engine_route_total - 2;
    }
    var engine_route_left = engine_route_total + seed;
    var engine_route_right = engine_route_left * 2;
    var engine_route_merged = engine_route_right - engine_route_left;
    if engine_route_merged > 2 {
        engine_route_total = engine_route_total + engine_route_merged;
    }
    return engine_route_total;
}

flow stdio_console_errors_pulse_north_score(seed: i32) -> i32 ![]
{
    var engine_score_total = seed + 21;
    var engine_score_cursor = 0;
    while engine_score_cursor < 8 limit Iterations(8) {
        engine_score_total = engine_score_total + engine_score_cursor + 2;
        engine_score_cursor = engine_score_cursor + 1;
    }
    if engine_score_total % 2 == 0 {
        engine_score_total = engine_score_total + 27;
    } else {
        engine_score_total = engine_score_total - 2;
    }
    var engine_score_left = engine_score_total + seed;
    var engine_score_right = engine_score_left * 2;
    var engine_score_merged = engine_score_right - engine_score_left;
    if engine_score_merged > 2 {
        engine_score_total = engine_score_total + engine_score_merged;
    }
    return engine_score_total;
}

flow stdio_console_errors_pulse_north_finish(seed: i32) -> i32 ![]
{
    var engine_finish_total = seed - 21;
    var engine_finish_cursor = 0;
    while engine_finish_cursor < 9 limit Iterations(9) {
        engine_finish_total = engine_finish_total + engine_finish_cursor + 2;
        engine_finish_cursor = engine_finish_cursor + 1;
    }
    if engine_finish_total % 2 == 0 {
        engine_finish_total = engine_finish_total + 27;
    } else {
        engine_finish_total = engine_finish_total - 2;
    }
    var engine_finish_left = engine_finish_total + seed;
    var engine_finish_right = engine_finish_left * 2;
    var engine_finish_merged = engine_finish_right - engine_finish_left;
    if engine_finish_merged > 2 {
        engine_finish_total = engine_finish_total + engine_finish_merged;
    }
    return engine_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var north_seed = 8;
    if args.len() > 0 {
        north_seed = north_seed + 1;
    } else {
        north_seed = north_seed + 2;
    }
    let north_result = stdio_console_errors_pulse_north_entry(north_seed);
    if north_result > 0 {
        return 0;
    }
    return 1;
}
