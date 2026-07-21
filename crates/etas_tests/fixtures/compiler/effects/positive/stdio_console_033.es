module tests.compiler.effects.positive.stdio_console_033;

import std.io.{println};

flow stdio_console_harbor_harbor_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var harbor_total = stdio_console_harbor_harbor_prepare(seed);
    harbor_total = harbor_total + stdio_console_harbor_harbor_route(seed + 7);
    println("stdio console 2");
    let harbor_adjust: i32 -> i32 = (value: i32) => value + 8;
    harbor_total = harbor_adjust(harbor_total);
    harbor_total = harbor_total + stdio_console_harbor_harbor_score(5);
    harbor_total = harbor_total + stdio_console_harbor_harbor_finish(8);
    if harbor_total > 73 {
        harbor_total = harbor_total - 2;
    } else {
        harbor_total = harbor_total + 20;
    }
    return harbor_total;
}

flow stdio_console_harbor_harbor_prepare(seed: i32) -> i32 ![]
{
    var keeper_prepare_total = seed + 17;
    var keeper_prepare_cursor = 0;
    while keeper_prepare_cursor < 11 limit Iterations(11) {
        keeper_prepare_total = keeper_prepare_total + keeper_prepare_cursor + 5;
        keeper_prepare_cursor = keeper_prepare_cursor + 1;
    }
    if keeper_prepare_total % 2 == 0 {
        keeper_prepare_total = keeper_prepare_total + stdio_console_harbor_harbor_score(1);
    } else {
        keeper_prepare_total = keeper_prepare_total - 4;
    }
    var keeper_prepare_left = keeper_prepare_total + seed;
    var keeper_prepare_right = keeper_prepare_left * 3;
    var keeper_prepare_merged = keeper_prepare_right - keeper_prepare_left;
    if keeper_prepare_merged > 2 {
        keeper_prepare_total = keeper_prepare_total + keeper_prepare_merged;
    }
    return keeper_prepare_total;
}

flow stdio_console_harbor_harbor_route(seed: i32) -> i32 ![]
{
    var keeper_route_total = seed * 17;
    var keeper_route_cursor = 0;
    while keeper_route_cursor < 10 limit Iterations(10) {
        keeper_route_total = keeper_route_total + keeper_route_cursor + 5;
        keeper_route_cursor = keeper_route_cursor + 1;
    }
    if keeper_route_total % 2 == 0 {
        keeper_route_total = keeper_route_total + 15;
    } else {
        keeper_route_total = keeper_route_total - 4;
    }
    var keeper_route_left = keeper_route_total + seed;
    var keeper_route_right = keeper_route_left * 3;
    var keeper_route_merged = keeper_route_right - keeper_route_left;
    if keeper_route_merged > 2 {
        keeper_route_total = keeper_route_total + keeper_route_merged;
    }
    return keeper_route_total;
}

flow stdio_console_harbor_harbor_score(seed: i32) -> i32 ![]
{
    var keeper_score_total = seed + 17;
    var keeper_score_cursor = 0;
    while keeper_score_cursor < 11 limit Iterations(11) {
        keeper_score_total = keeper_score_total + keeper_score_cursor + 5;
        keeper_score_cursor = keeper_score_cursor + 1;
    }
    if keeper_score_total % 2 == 0 {
        keeper_score_total = keeper_score_total + 15;
    } else {
        keeper_score_total = keeper_score_total - 4;
    }
    var keeper_score_left = keeper_score_total + seed;
    var keeper_score_right = keeper_score_left * 3;
    var keeper_score_merged = keeper_score_right - keeper_score_left;
    if keeper_score_merged > 2 {
        keeper_score_total = keeper_score_total + keeper_score_merged;
    }
    return keeper_score_total;
}

flow stdio_console_harbor_harbor_finish(seed: i32) -> i32 ![]
{
    var keeper_finish_total = seed - 17;
    var keeper_finish_cursor = 0;
    while keeper_finish_cursor < 6 limit Iterations(6) {
        keeper_finish_total = keeper_finish_total + keeper_finish_cursor + 5;
        keeper_finish_cursor = keeper_finish_cursor + 1;
    }
    if keeper_finish_total % 2 == 0 {
        keeper_finish_total = keeper_finish_total + 15;
    } else {
        keeper_finish_total = keeper_finish_total - 4;
    }
    var keeper_finish_left = keeper_finish_total + seed;
    var keeper_finish_right = keeper_finish_left * 3;
    var keeper_finish_merged = keeper_finish_right - keeper_finish_left;
    if keeper_finish_merged > 2 {
        keeper_finish_total = keeper_finish_total + keeper_finish_merged;
    }
    return keeper_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var harbor_seed = 1;
    if args.len() > 0 {
        harbor_seed = harbor_seed + 1;
    } else {
        harbor_seed = harbor_seed + 2;
    }
    let harbor_result = stdio_console_harbor_harbor_entry(harbor_seed);
    if harbor_result > 0 {
        return 0;
    }
    return 1;
}
