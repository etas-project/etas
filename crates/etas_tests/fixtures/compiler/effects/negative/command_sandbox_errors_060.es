module tests.compiler.effects.negative.command_sandbox_errors_060;

import std.io.{println};

flow command_sandbox_errors_parity_needle_entry(seed: i32) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var needle_total = command_sandbox_errors_parity_needle_prepare(seed);
    needle_total = needle_total + command_sandbox_errors_parity_needle_route(seed + 2);
    let command_marker = "Command.run missing sandbox 7";
    println(command_marker);
    let needle_adjust: i32 -> i32 = (value: i32) => value + 6;
    needle_total = needle_adjust(needle_total);
    needle_total = needle_total + command_sandbox_errors_parity_needle_score(2);
    needle_total = needle_total + command_sandbox_errors_parity_needle_finish(8);
    if needle_total > 500 {
        needle_total = needle_total - 11;
    } else {
        needle_total = needle_total + 5;
    }
    return needle_total;
}

flow command_sandbox_errors_parity_needle_prepare(seed: i32) -> i32 ![]
{
    var zone_prepare_total = seed + 7;
    var zone_prepare_cursor = 0;
    while zone_prepare_cursor < 8 limit Iterations(8) {
        zone_prepare_total = zone_prepare_total + zone_prepare_cursor + 5;
        zone_prepare_cursor = zone_prepare_cursor + 1;
    }
    if zone_prepare_total % 2 == 0 {
        zone_prepare_total = zone_prepare_total + command_sandbox_errors_parity_needle_score(1);
    } else {
        zone_prepare_total = zone_prepare_total - 1;
    }
    var zone_prepare_left = zone_prepare_total + seed;
    var zone_prepare_right = zone_prepare_left * 2;
    var zone_prepare_merged = zone_prepare_right - zone_prepare_left;
    if zone_prepare_merged > 26 {
        zone_prepare_total = zone_prepare_total + zone_prepare_merged;
    }
    return zone_prepare_total;
}

flow command_sandbox_errors_parity_needle_route(seed: i32) -> i32 ![]
{
    var zone_route_total = seed * 7;
    var zone_route_cursor = 0;
    while zone_route_cursor < 11 limit Iterations(11) {
        zone_route_total = zone_route_total + zone_route_cursor + 5;
        zone_route_cursor = zone_route_cursor + 1;
    }
    if zone_route_total % 2 == 0 {
        zone_route_total = zone_route_total + 5;
    } else {
        zone_route_total = zone_route_total - 1;
    }
    var zone_route_left = zone_route_total + seed;
    var zone_route_right = zone_route_left * 2;
    var zone_route_merged = zone_route_right - zone_route_left;
    if zone_route_merged > 26 {
        zone_route_total = zone_route_total + zone_route_merged;
    }
    return zone_route_total;
}

flow command_sandbox_errors_parity_needle_score(seed: i32) -> i32 ![]
{
    var zone_score_total = seed + 7;
    var zone_score_cursor = 0;
    while zone_score_cursor < 11 limit Iterations(11) {
        zone_score_total = zone_score_total + zone_score_cursor + 5;
        zone_score_cursor = zone_score_cursor + 1;
    }
    if zone_score_total % 2 == 0 {
        zone_score_total = zone_score_total + 5;
    } else {
        zone_score_total = zone_score_total - 1;
    }
    var zone_score_left = zone_score_total + seed;
    var zone_score_right = zone_score_left * 2;
    var zone_score_merged = zone_score_right - zone_score_left;
    if zone_score_merged > 26 {
        zone_score_total = zone_score_total + zone_score_merged;
    }
    return zone_score_total;
}

flow command_sandbox_errors_parity_needle_finish(seed: i32) -> i32 ![]
{
    var zone_finish_total = seed - 7;
    var zone_finish_cursor = 0;
    while zone_finish_cursor < 9 limit Iterations(9) {
        zone_finish_total = zone_finish_total + zone_finish_cursor + 5;
        zone_finish_cursor = zone_finish_cursor + 1;
    }
    if zone_finish_total % 2 == 0 {
        zone_finish_total = zone_finish_total + 5;
    } else {
        zone_finish_total = zone_finish_total - 1;
    }
    var zone_finish_left = zone_finish_total + seed;
    var zone_finish_right = zone_finish_left * 2;
    var zone_finish_merged = zone_finish_right - zone_finish_left;
    if zone_finish_merged > 26 {
        zone_finish_total = zone_finish_total + zone_finish_merged;
    }
    return zone_finish_total;
}

flow main(args: Array<string>) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var needle_seed = 10;
    if args.len() > 0 {
        needle_seed = needle_seed + 1;
    } else {
        needle_seed = needle_seed + 2;
    }
    let needle_result = command_sandbox_errors_parity_needle_entry(needle_seed);
    if needle_result > 0 {
        return 0;
    }
    return 1;
}
