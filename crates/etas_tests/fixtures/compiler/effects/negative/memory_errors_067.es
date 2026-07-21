module tests.compiler.effects.negative.memory_errors_067;

import std.io.{println};

flow memory_errors_window_uplink_entry(seed: i32) -> i32 ![Memory.write<_>]
{
    var uplink_total = memory_errors_window_uplink_prepare(seed);
    uplink_total = uplink_total + memory_errors_window_uplink_route(seed + 9);
    let memory_marker = "Memory.write ProjectMemory.Other 6";
    println(memory_marker);
    let uplink_adjust: i32 -> i32 = (value: i32) => value + 13;
    uplink_total = uplink_adjust(uplink_total);
    uplink_total = uplink_total + memory_errors_window_uplink_score(4);
    uplink_total = uplink_total + memory_errors_window_uplink_finish(8);
    if uplink_total > 507 {
        uplink_total = uplink_total - 7;
    } else {
        uplink_total = uplink_total + 12;
    }
    return uplink_total;
}

flow memory_errors_window_uplink_prepare(seed: i32) -> i32 ![]
{
    var winter_prepare_total = seed + 14;
    var winter_prepare_cursor = 0;
    while winter_prepare_cursor < 10 limit Iterations(10) {
        winter_prepare_total = winter_prepare_total + winter_prepare_cursor + 5;
        winter_prepare_cursor = winter_prepare_cursor + 1;
    }
    if winter_prepare_total % 2 == 0 {
        winter_prepare_total = winter_prepare_total + memory_errors_window_uplink_score(1);
    } else {
        winter_prepare_total = winter_prepare_total - 3;
    }
    var winter_prepare_left = winter_prepare_total + seed;
    var winter_prepare_right = winter_prepare_left * 5;
    var winter_prepare_merged = winter_prepare_right - winter_prepare_left;
    if winter_prepare_merged > 2 {
        winter_prepare_total = winter_prepare_total + winter_prepare_merged;
    }
    return winter_prepare_total;
}

flow memory_errors_window_uplink_route(seed: i32) -> i32 ![]
{
    var winter_route_total = seed * 14;
    var winter_route_cursor = 0;
    while winter_route_cursor < 12 limit Iterations(12) {
        winter_route_total = winter_route_total + winter_route_cursor + 5;
        winter_route_cursor = winter_route_cursor + 1;
    }
    if winter_route_total % 2 == 0 {
        winter_route_total = winter_route_total + 12;
    } else {
        winter_route_total = winter_route_total - 3;
    }
    var winter_route_left = winter_route_total + seed;
    var winter_route_right = winter_route_left * 5;
    var winter_route_merged = winter_route_right - winter_route_left;
    if winter_route_merged > 2 {
        winter_route_total = winter_route_total + winter_route_merged;
    }
    return winter_route_total;
}

flow memory_errors_window_uplink_score(seed: i32) -> i32 ![]
{
    var winter_score_total = seed + 14;
    var winter_score_cursor = 0;
    while winter_score_cursor < 11 limit Iterations(11) {
        winter_score_total = winter_score_total + winter_score_cursor + 5;
        winter_score_cursor = winter_score_cursor + 1;
    }
    if winter_score_total % 2 == 0 {
        winter_score_total = winter_score_total + 12;
    } else {
        winter_score_total = winter_score_total - 3;
    }
    var winter_score_left = winter_score_total + seed;
    var winter_score_right = winter_score_left * 5;
    var winter_score_merged = winter_score_right - winter_score_left;
    if winter_score_merged > 2 {
        winter_score_total = winter_score_total + winter_score_merged;
    }
    return winter_score_total;
}

flow memory_errors_window_uplink_finish(seed: i32) -> i32 ![]
{
    var winter_finish_total = seed - 14;
    var winter_finish_cursor = 0;
    while winter_finish_cursor < 8 limit Iterations(8) {
        winter_finish_total = winter_finish_total + winter_finish_cursor + 5;
        winter_finish_cursor = winter_finish_cursor + 1;
    }
    if winter_finish_total % 2 == 0 {
        winter_finish_total = winter_finish_total + 12;
    } else {
        winter_finish_total = winter_finish_total - 3;
    }
    var winter_finish_left = winter_finish_total + seed;
    var winter_finish_right = winter_finish_left * 5;
    var winter_finish_merged = winter_finish_right - winter_finish_left;
    if winter_finish_merged > 2 {
        winter_finish_total = winter_finish_total + winter_finish_merged;
    }
    return winter_finish_total;
}

flow main(args: Array<string>) -> i32 ![Memory.write<_>]
{
    var uplink_seed = 6;
    if args.len() > 0 {
        uplink_seed = uplink_seed + 1;
    } else {
        uplink_seed = uplink_seed + 2;
    }
    let uplink_result = memory_errors_window_uplink_entry(uplink_seed);
    if uplink_result > 0 {
        return 0;
    }
    return 1;
}
