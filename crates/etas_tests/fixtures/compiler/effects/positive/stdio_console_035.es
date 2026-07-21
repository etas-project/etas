module tests.compiler.effects.positive.stdio_console_035;

import std.io.{println};

flow stdio_console_juno_juno_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var juno_total = stdio_console_juno_juno_prepare(seed);
    juno_total = juno_total + stdio_console_juno_juno_route(seed + 9);
    println("stdio console 4");
    let juno_adjust: i32 -> i32 = (value: i32) => value + 10;
    juno_total = juno_adjust(juno_total);
    juno_total = juno_total + stdio_console_juno_juno_score(2);
    juno_total = juno_total + stdio_console_juno_juno_finish(3);
    if juno_total > 75 {
        juno_total = juno_total - 4;
    } else {
        juno_total = juno_total + 5;
    }
    return juno_total;
}

flow stdio_console_juno_juno_prepare(seed: i32) -> i32 ![]
{
    var alpha_prepare_total = seed + 19;
    var alpha_prepare_cursor = 0;
    while alpha_prepare_cursor < 8 limit Iterations(8) {
        alpha_prepare_total = alpha_prepare_total + alpha_prepare_cursor + 0;
        alpha_prepare_cursor = alpha_prepare_cursor + 1;
    }
    if alpha_prepare_total % 2 == 0 {
        alpha_prepare_total = alpha_prepare_total + stdio_console_juno_juno_score(1);
    } else {
        alpha_prepare_total = alpha_prepare_total - 1;
    }
    var alpha_prepare_left = alpha_prepare_total + seed;
    var alpha_prepare_right = alpha_prepare_left * 5;
    var alpha_prepare_merged = alpha_prepare_right - alpha_prepare_left;
    if alpha_prepare_merged > 4 {
        alpha_prepare_total = alpha_prepare_total + alpha_prepare_merged;
    }
    return alpha_prepare_total;
}

flow stdio_console_juno_juno_route(seed: i32) -> i32 ![]
{
    var alpha_route_total = seed * 19;
    var alpha_route_cursor = 0;
    while alpha_route_cursor < 12 limit Iterations(12) {
        alpha_route_total = alpha_route_total + alpha_route_cursor + 0;
        alpha_route_cursor = alpha_route_cursor + 1;
    }
    if alpha_route_total % 2 == 0 {
        alpha_route_total = alpha_route_total + 17;
    } else {
        alpha_route_total = alpha_route_total - 1;
    }
    var alpha_route_left = alpha_route_total + seed;
    var alpha_route_right = alpha_route_left * 5;
    var alpha_route_merged = alpha_route_right - alpha_route_left;
    if alpha_route_merged > 4 {
        alpha_route_total = alpha_route_total + alpha_route_merged;
    }
    return alpha_route_total;
}

flow stdio_console_juno_juno_score(seed: i32) -> i32 ![]
{
    var alpha_score_total = seed + 19;
    var alpha_score_cursor = 0;
    while alpha_score_cursor < 6 limit Iterations(6) {
        alpha_score_total = alpha_score_total + alpha_score_cursor + 0;
        alpha_score_cursor = alpha_score_cursor + 1;
    }
    if alpha_score_total % 2 == 0 {
        alpha_score_total = alpha_score_total + 17;
    } else {
        alpha_score_total = alpha_score_total - 1;
    }
    var alpha_score_left = alpha_score_total + seed;
    var alpha_score_right = alpha_score_left * 5;
    var alpha_score_merged = alpha_score_right - alpha_score_left;
    if alpha_score_merged > 4 {
        alpha_score_total = alpha_score_total + alpha_score_merged;
    }
    return alpha_score_total;
}

flow stdio_console_juno_juno_finish(seed: i32) -> i32 ![]
{
    var alpha_finish_total = seed - 19;
    var alpha_finish_cursor = 0;
    while alpha_finish_cursor < 8 limit Iterations(8) {
        alpha_finish_total = alpha_finish_total + alpha_finish_cursor + 0;
        alpha_finish_cursor = alpha_finish_cursor + 1;
    }
    if alpha_finish_total % 2 == 0 {
        alpha_finish_total = alpha_finish_total + 17;
    } else {
        alpha_finish_total = alpha_finish_total - 1;
    }
    var alpha_finish_left = alpha_finish_total + seed;
    var alpha_finish_right = alpha_finish_left * 5;
    var alpha_finish_merged = alpha_finish_right - alpha_finish_left;
    if alpha_finish_merged > 4 {
        alpha_finish_total = alpha_finish_total + alpha_finish_merged;
    }
    return alpha_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var juno_seed = 3;
    if args.len() > 0 {
        juno_seed = juno_seed + 1;
    } else {
        juno_seed = juno_seed + 2;
    }
    let juno_result = stdio_console_juno_juno_entry(juno_seed);
    if juno_result > 0 {
        return 0;
    }
    return 1;
}
