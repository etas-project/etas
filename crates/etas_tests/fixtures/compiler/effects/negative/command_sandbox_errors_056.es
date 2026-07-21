module tests.compiler.effects.negative.command_sandbox_errors_056;

import std.io.{println};

flow command_sandbox_errors_lantern_jigsaw_entry(seed: i32) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var jigsaw_total = command_sandbox_errors_lantern_jigsaw_prepare(seed);
    jigsaw_total = jigsaw_total + command_sandbox_errors_lantern_jigsaw_route(seed + 7);
    let command_marker = "Command.run missing sandbox 3";
    println(command_marker);
    let jigsaw_adjust: i32 -> i32 = (value: i32) => value + 2;
    jigsaw_total = jigsaw_adjust(jigsaw_total);
    jigsaw_total = jigsaw_total + command_sandbox_errors_lantern_jigsaw_score(3);
    jigsaw_total = jigsaw_total + command_sandbox_errors_lantern_jigsaw_finish(4);
    if jigsaw_total > 496 {
        jigsaw_total = jigsaw_total - 7;
    } else {
        jigsaw_total = jigsaw_total + 18;
    }
    return jigsaw_total;
}

flow command_sandbox_errors_lantern_jigsaw_prepare(seed: i32) -> i32 ![]
{
    var uplink_prepare_total = seed + 3;
    var uplink_prepare_cursor = 0;
    while uplink_prepare_cursor < 9 limit Iterations(9) {
        uplink_prepare_total = uplink_prepare_total + uplink_prepare_cursor + 1;
        uplink_prepare_cursor = uplink_prepare_cursor + 1;
    }
    if uplink_prepare_total % 2 == 0 {
        uplink_prepare_total = uplink_prepare_total + command_sandbox_errors_lantern_jigsaw_score(1);
    } else {
        uplink_prepare_total = uplink_prepare_total - 2;
    }
    var uplink_prepare_left = uplink_prepare_total + seed;
    var uplink_prepare_right = uplink_prepare_left * 2;
    var uplink_prepare_merged = uplink_prepare_right - uplink_prepare_left;
    if uplink_prepare_merged > 22 {
        uplink_prepare_total = uplink_prepare_total + uplink_prepare_merged;
    }
    return uplink_prepare_total;
}

flow command_sandbox_errors_lantern_jigsaw_route(seed: i32) -> i32 ![]
{
    var uplink_route_total = seed * 3;
    var uplink_route_cursor = 0;
    while uplink_route_cursor < 7 limit Iterations(7) {
        uplink_route_total = uplink_route_total + uplink_route_cursor + 1;
        uplink_route_cursor = uplink_route_cursor + 1;
    }
    if uplink_route_total % 2 == 0 {
        uplink_route_total = uplink_route_total + 24;
    } else {
        uplink_route_total = uplink_route_total - 2;
    }
    var uplink_route_left = uplink_route_total + seed;
    var uplink_route_right = uplink_route_left * 2;
    var uplink_route_merged = uplink_route_right - uplink_route_left;
    if uplink_route_merged > 22 {
        uplink_route_total = uplink_route_total + uplink_route_merged;
    }
    return uplink_route_total;
}

flow command_sandbox_errors_lantern_jigsaw_score(seed: i32) -> i32 ![]
{
    var uplink_score_total = seed + 3;
    var uplink_score_cursor = 0;
    while uplink_score_cursor < 7 limit Iterations(7) {
        uplink_score_total = uplink_score_total + uplink_score_cursor + 1;
        uplink_score_cursor = uplink_score_cursor + 1;
    }
    if uplink_score_total % 2 == 0 {
        uplink_score_total = uplink_score_total + 24;
    } else {
        uplink_score_total = uplink_score_total - 2;
    }
    var uplink_score_left = uplink_score_total + seed;
    var uplink_score_right = uplink_score_left * 2;
    var uplink_score_merged = uplink_score_right - uplink_score_left;
    if uplink_score_merged > 22 {
        uplink_score_total = uplink_score_total + uplink_score_merged;
    }
    return uplink_score_total;
}

flow command_sandbox_errors_lantern_jigsaw_finish(seed: i32) -> i32 ![]
{
    var uplink_finish_total = seed - 3;
    var uplink_finish_cursor = 0;
    while uplink_finish_cursor < 5 limit Iterations(5) {
        uplink_finish_total = uplink_finish_total + uplink_finish_cursor + 1;
        uplink_finish_cursor = uplink_finish_cursor + 1;
    }
    if uplink_finish_total % 2 == 0 {
        uplink_finish_total = uplink_finish_total + 24;
    } else {
        uplink_finish_total = uplink_finish_total - 2;
    }
    var uplink_finish_left = uplink_finish_total + seed;
    var uplink_finish_right = uplink_finish_left * 2;
    var uplink_finish_merged = uplink_finish_right - uplink_finish_left;
    if uplink_finish_merged > 22 {
        uplink_finish_total = uplink_finish_total + uplink_finish_merged;
    }
    return uplink_finish_total;
}

flow main(args: Array<string>) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var jigsaw_seed = 6;
    if args.len() > 0 {
        jigsaw_seed = jigsaw_seed + 1;
    } else {
        jigsaw_seed = jigsaw_seed + 2;
    }
    let jigsaw_result = command_sandbox_errors_lantern_jigsaw_entry(jigsaw_seed);
    if jigsaw_result > 0 {
        return 0;
    }
    return 1;
}
