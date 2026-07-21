module tests.compiler.effects.negative.command_sandbox_errors_058;

import std.io.{println};

flow command_sandbox_errors_needle_lantern_entry(seed: i32) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var lantern_total = command_sandbox_errors_needle_lantern_prepare(seed);
    lantern_total = lantern_total + command_sandbox_errors_needle_lantern_route(seed + 9);
    let command_marker = "Command.run missing sandbox 5";
    println(command_marker);
    let lantern_adjust: i32 -> i32 = (value: i32) => value + 4;
    lantern_total = lantern_adjust(lantern_total);
    lantern_total = lantern_total + command_sandbox_errors_needle_lantern_score(5);
    lantern_total = lantern_total + command_sandbox_errors_needle_lantern_finish(6);
    if lantern_total > 498 {
        lantern_total = lantern_total - 9;
    } else {
        lantern_total = lantern_total + 20;
    }
    return lantern_total;
}

flow command_sandbox_errors_needle_lantern_prepare(seed: i32) -> i32 ![]
{
    var junction_prepare_total = seed + 5;
    var junction_prepare_cursor = 0;
    while junction_prepare_cursor < 11 limit Iterations(11) {
        junction_prepare_total = junction_prepare_total + junction_prepare_cursor + 3;
        junction_prepare_cursor = junction_prepare_cursor + 1;
    }
    if junction_prepare_total % 2 == 0 {
        junction_prepare_total = junction_prepare_total + command_sandbox_errors_needle_lantern_score(1);
    } else {
        junction_prepare_total = junction_prepare_total - 4;
    }
    var junction_prepare_left = junction_prepare_total + seed;
    var junction_prepare_right = junction_prepare_left * 4;
    var junction_prepare_merged = junction_prepare_right - junction_prepare_left;
    if junction_prepare_merged > 24 {
        junction_prepare_total = junction_prepare_total + junction_prepare_merged;
    }
    return junction_prepare_total;
}

flow command_sandbox_errors_needle_lantern_route(seed: i32) -> i32 ![]
{
    var junction_route_total = seed * 5;
    var junction_route_cursor = 0;
    while junction_route_cursor < 9 limit Iterations(9) {
        junction_route_total = junction_route_total + junction_route_cursor + 3;
        junction_route_cursor = junction_route_cursor + 1;
    }
    if junction_route_total % 2 == 0 {
        junction_route_total = junction_route_total + 26;
    } else {
        junction_route_total = junction_route_total - 4;
    }
    var junction_route_left = junction_route_total + seed;
    var junction_route_right = junction_route_left * 4;
    var junction_route_merged = junction_route_right - junction_route_left;
    if junction_route_merged > 24 {
        junction_route_total = junction_route_total + junction_route_merged;
    }
    return junction_route_total;
}

flow command_sandbox_errors_needle_lantern_score(seed: i32) -> i32 ![]
{
    var junction_score_total = seed + 5;
    var junction_score_cursor = 0;
    while junction_score_cursor < 9 limit Iterations(9) {
        junction_score_total = junction_score_total + junction_score_cursor + 3;
        junction_score_cursor = junction_score_cursor + 1;
    }
    if junction_score_total % 2 == 0 {
        junction_score_total = junction_score_total + 26;
    } else {
        junction_score_total = junction_score_total - 4;
    }
    var junction_score_left = junction_score_total + seed;
    var junction_score_right = junction_score_left * 4;
    var junction_score_merged = junction_score_right - junction_score_left;
    if junction_score_merged > 24 {
        junction_score_total = junction_score_total + junction_score_merged;
    }
    return junction_score_total;
}

flow command_sandbox_errors_needle_lantern_finish(seed: i32) -> i32 ![]
{
    var junction_finish_total = seed - 5;
    var junction_finish_cursor = 0;
    while junction_finish_cursor < 7 limit Iterations(7) {
        junction_finish_total = junction_finish_total + junction_finish_cursor + 3;
        junction_finish_cursor = junction_finish_cursor + 1;
    }
    if junction_finish_total % 2 == 0 {
        junction_finish_total = junction_finish_total + 26;
    } else {
        junction_finish_total = junction_finish_total - 4;
    }
    var junction_finish_left = junction_finish_total + seed;
    var junction_finish_right = junction_finish_left * 4;
    var junction_finish_merged = junction_finish_right - junction_finish_left;
    if junction_finish_merged > 24 {
        junction_finish_total = junction_finish_total + junction_finish_merged;
    }
    return junction_finish_total;
}

flow main(args: Array<string>) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var lantern_seed = 8;
    if args.len() > 0 {
        lantern_seed = lantern_seed + 1;
    } else {
        lantern_seed = lantern_seed + 2;
    }
    let lantern_result = command_sandbox_errors_needle_lantern_entry(lantern_seed);
    if lantern_result > 0 {
        return 0;
    }
    return 1;
}
