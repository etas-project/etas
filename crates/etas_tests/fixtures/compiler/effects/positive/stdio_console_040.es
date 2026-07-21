module tests.compiler.effects.positive.stdio_console_040;

import std.io.{println};

flow stdio_console_onyx_onyx_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var onyx_total = stdio_console_onyx_onyx_prepare(seed);
    onyx_total = onyx_total + stdio_console_onyx_onyx_route(seed + 5);
    println("stdio console 9");
    let onyx_adjust: i32 -> i32 = (value: i32) => value + 2;
    onyx_total = onyx_adjust(onyx_total);
    onyx_total = onyx_total + stdio_console_onyx_onyx_score(2);
    onyx_total = onyx_total + stdio_console_onyx_onyx_finish(8);
    if onyx_total > 80 {
        onyx_total = onyx_total - 9;
    } else {
        onyx_total = onyx_total + 10;
    }
    return onyx_total;
}

flow stdio_console_onyx_onyx_prepare(seed: i32) -> i32 ![]
{
    var juno_prepare_total = seed + 5;
    var juno_prepare_cursor = 0;
    while juno_prepare_cursor < 8 limit Iterations(8) {
        juno_prepare_total = juno_prepare_total + juno_prepare_cursor + 5;
        juno_prepare_cursor = juno_prepare_cursor + 1;
    }
    if juno_prepare_total % 2 == 0 {
        juno_prepare_total = juno_prepare_total + stdio_console_onyx_onyx_score(1);
    } else {
        juno_prepare_total = juno_prepare_total - 1;
    }
    var juno_prepare_left = juno_prepare_total + seed;
    var juno_prepare_right = juno_prepare_left * 2;
    var juno_prepare_merged = juno_prepare_right - juno_prepare_left;
    if juno_prepare_merged > 9 {
        juno_prepare_total = juno_prepare_total + juno_prepare_merged;
    }
    return juno_prepare_total;
}

flow stdio_console_onyx_onyx_route(seed: i32) -> i32 ![]
{
    var juno_route_total = seed * 5;
    var juno_route_cursor = 0;
    while juno_route_cursor < 11 limit Iterations(11) {
        juno_route_total = juno_route_total + juno_route_cursor + 5;
        juno_route_cursor = juno_route_cursor + 1;
    }
    if juno_route_total % 2 == 0 {
        juno_route_total = juno_route_total + 22;
    } else {
        juno_route_total = juno_route_total - 1;
    }
    var juno_route_left = juno_route_total + seed;
    var juno_route_right = juno_route_left * 2;
    var juno_route_merged = juno_route_right - juno_route_left;
    if juno_route_merged > 9 {
        juno_route_total = juno_route_total + juno_route_merged;
    }
    return juno_route_total;
}

flow stdio_console_onyx_onyx_score(seed: i32) -> i32 ![]
{
    var juno_score_total = seed + 5;
    var juno_score_cursor = 0;
    while juno_score_cursor < 11 limit Iterations(11) {
        juno_score_total = juno_score_total + juno_score_cursor + 5;
        juno_score_cursor = juno_score_cursor + 1;
    }
    if juno_score_total % 2 == 0 {
        juno_score_total = juno_score_total + 22;
    } else {
        juno_score_total = juno_score_total - 1;
    }
    var juno_score_left = juno_score_total + seed;
    var juno_score_right = juno_score_left * 2;
    var juno_score_merged = juno_score_right - juno_score_left;
    if juno_score_merged > 9 {
        juno_score_total = juno_score_total + juno_score_merged;
    }
    return juno_score_total;
}

flow stdio_console_onyx_onyx_finish(seed: i32) -> i32 ![]
{
    var juno_finish_total = seed - 5;
    var juno_finish_cursor = 0;
    while juno_finish_cursor < 5 limit Iterations(5) {
        juno_finish_total = juno_finish_total + juno_finish_cursor + 5;
        juno_finish_cursor = juno_finish_cursor + 1;
    }
    if juno_finish_total % 2 == 0 {
        juno_finish_total = juno_finish_total + 22;
    } else {
        juno_finish_total = juno_finish_total - 1;
    }
    var juno_finish_left = juno_finish_total + seed;
    var juno_finish_right = juno_finish_left * 2;
    var juno_finish_merged = juno_finish_right - juno_finish_left;
    if juno_finish_merged > 9 {
        juno_finish_total = juno_finish_total + juno_finish_merged;
    }
    return juno_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var onyx_seed = 8;
    if args.len() > 0 {
        onyx_seed = onyx_seed + 1;
    } else {
        onyx_seed = onyx_seed + 2;
    }
    let onyx_result = stdio_console_onyx_onyx_entry(onyx_seed);
    if onyx_result > 0 {
        return 0;
    }
    return 1;
}
