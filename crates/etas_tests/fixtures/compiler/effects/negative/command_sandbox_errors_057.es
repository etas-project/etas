module tests.compiler.effects.negative.command_sandbox_errors_057;

import std.io.{println};

flow command_sandbox_errors_meadow_kingdom_entry(seed: i32) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var kingdom_total = command_sandbox_errors_meadow_kingdom_prepare(seed);
    kingdom_total = kingdom_total + command_sandbox_errors_meadow_kingdom_route(seed + 8);
    let command_marker = "Command.run missing sandbox 4";
    println(command_marker);
    let kingdom_adjust: i32 -> i32 = (value: i32) => value + 3;
    kingdom_total = kingdom_adjust(kingdom_total);
    kingdom_total = kingdom_total + command_sandbox_errors_meadow_kingdom_score(4);
    kingdom_total = kingdom_total + command_sandbox_errors_meadow_kingdom_finish(5);
    if kingdom_total > 497 {
        kingdom_total = kingdom_total - 8;
    } else {
        kingdom_total = kingdom_total + 19;
    }
    return kingdom_total;
}

flow command_sandbox_errors_meadow_kingdom_prepare(seed: i32) -> i32 ![]
{
    var crystal_prepare_total = seed + 4;
    var crystal_prepare_cursor = 0;
    while crystal_prepare_cursor < 10 limit Iterations(10) {
        crystal_prepare_total = crystal_prepare_total + crystal_prepare_cursor + 2;
        crystal_prepare_cursor = crystal_prepare_cursor + 1;
    }
    if crystal_prepare_total % 2 == 0 {
        crystal_prepare_total = crystal_prepare_total + command_sandbox_errors_meadow_kingdom_score(1);
    } else {
        crystal_prepare_total = crystal_prepare_total - 3;
    }
    var crystal_prepare_left = crystal_prepare_total + seed;
    var crystal_prepare_right = crystal_prepare_left * 3;
    var crystal_prepare_merged = crystal_prepare_right - crystal_prepare_left;
    if crystal_prepare_merged > 23 {
        crystal_prepare_total = crystal_prepare_total + crystal_prepare_merged;
    }
    return crystal_prepare_total;
}

flow command_sandbox_errors_meadow_kingdom_route(seed: i32) -> i32 ![]
{
    var crystal_route_total = seed * 4;
    var crystal_route_cursor = 0;
    while crystal_route_cursor < 8 limit Iterations(8) {
        crystal_route_total = crystal_route_total + crystal_route_cursor + 2;
        crystal_route_cursor = crystal_route_cursor + 1;
    }
    if crystal_route_total % 2 == 0 {
        crystal_route_total = crystal_route_total + 25;
    } else {
        crystal_route_total = crystal_route_total - 3;
    }
    var crystal_route_left = crystal_route_total + seed;
    var crystal_route_right = crystal_route_left * 3;
    var crystal_route_merged = crystal_route_right - crystal_route_left;
    if crystal_route_merged > 23 {
        crystal_route_total = crystal_route_total + crystal_route_merged;
    }
    return crystal_route_total;
}

flow command_sandbox_errors_meadow_kingdom_score(seed: i32) -> i32 ![]
{
    var crystal_score_total = seed + 4;
    var crystal_score_cursor = 0;
    while crystal_score_cursor < 8 limit Iterations(8) {
        crystal_score_total = crystal_score_total + crystal_score_cursor + 2;
        crystal_score_cursor = crystal_score_cursor + 1;
    }
    if crystal_score_total % 2 == 0 {
        crystal_score_total = crystal_score_total + 25;
    } else {
        crystal_score_total = crystal_score_total - 3;
    }
    var crystal_score_left = crystal_score_total + seed;
    var crystal_score_right = crystal_score_left * 3;
    var crystal_score_merged = crystal_score_right - crystal_score_left;
    if crystal_score_merged > 23 {
        crystal_score_total = crystal_score_total + crystal_score_merged;
    }
    return crystal_score_total;
}

flow command_sandbox_errors_meadow_kingdom_finish(seed: i32) -> i32 ![]
{
    var crystal_finish_total = seed - 4;
    var crystal_finish_cursor = 0;
    while crystal_finish_cursor < 6 limit Iterations(6) {
        crystal_finish_total = crystal_finish_total + crystal_finish_cursor + 2;
        crystal_finish_cursor = crystal_finish_cursor + 1;
    }
    if crystal_finish_total % 2 == 0 {
        crystal_finish_total = crystal_finish_total + 25;
    } else {
        crystal_finish_total = crystal_finish_total - 3;
    }
    var crystal_finish_left = crystal_finish_total + seed;
    var crystal_finish_right = crystal_finish_left * 3;
    var crystal_finish_merged = crystal_finish_right - crystal_finish_left;
    if crystal_finish_merged > 23 {
        crystal_finish_total = crystal_finish_total + crystal_finish_merged;
    }
    return crystal_finish_total;
}

flow main(args: Array<string>) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var kingdom_seed = 7;
    if args.len() > 0 {
        kingdom_seed = kingdom_seed + 1;
    } else {
        kingdom_seed = kingdom_seed + 2;
    }
    let kingdom_result = command_sandbox_errors_meadow_kingdom_entry(kingdom_seed);
    if kingdom_result > 0 {
        return 0;
    }
    return 1;
}
