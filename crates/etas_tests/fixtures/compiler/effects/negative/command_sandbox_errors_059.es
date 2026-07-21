module tests.compiler.effects.negative.command_sandbox_errors_059;

import std.io.{println};

flow command_sandbox_errors_oasis_meadow_entry(seed: i32) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var meadow_total = command_sandbox_errors_oasis_meadow_prepare(seed);
    meadow_total = meadow_total + command_sandbox_errors_oasis_meadow_route(seed + 1);
    let command_marker = "Command.run missing sandbox 6";
    println(command_marker);
    let meadow_adjust: i32 -> i32 = (value: i32) => value + 5;
    meadow_total = meadow_adjust(meadow_total);
    meadow_total = meadow_total + command_sandbox_errors_oasis_meadow_score(6);
    meadow_total = meadow_total + command_sandbox_errors_oasis_meadow_finish(7);
    if meadow_total > 499 {
        meadow_total = meadow_total - 10;
    } else {
        meadow_total = meadow_total + 4;
    }
    return meadow_total;
}

flow command_sandbox_errors_oasis_meadow_prepare(seed: i32) -> i32 ![]
{
    var radius_prepare_total = seed + 6;
    var radius_prepare_cursor = 0;
    while radius_prepare_cursor < 12 limit Iterations(12) {
        radius_prepare_total = radius_prepare_total + radius_prepare_cursor + 4;
        radius_prepare_cursor = radius_prepare_cursor + 1;
    }
    if radius_prepare_total % 2 == 0 {
        radius_prepare_total = radius_prepare_total + command_sandbox_errors_oasis_meadow_score(1);
    } else {
        radius_prepare_total = radius_prepare_total - 5;
    }
    var radius_prepare_left = radius_prepare_total + seed;
    var radius_prepare_right = radius_prepare_left * 5;
    var radius_prepare_merged = radius_prepare_right - radius_prepare_left;
    if radius_prepare_merged > 25 {
        radius_prepare_total = radius_prepare_total + radius_prepare_merged;
    }
    return radius_prepare_total;
}

flow command_sandbox_errors_oasis_meadow_route(seed: i32) -> i32 ![]
{
    var radius_route_total = seed * 6;
    var radius_route_cursor = 0;
    while radius_route_cursor < 10 limit Iterations(10) {
        radius_route_total = radius_route_total + radius_route_cursor + 4;
        radius_route_cursor = radius_route_cursor + 1;
    }
    if radius_route_total % 2 == 0 {
        radius_route_total = radius_route_total + 27;
    } else {
        radius_route_total = radius_route_total - 5;
    }
    var radius_route_left = radius_route_total + seed;
    var radius_route_right = radius_route_left * 5;
    var radius_route_merged = radius_route_right - radius_route_left;
    if radius_route_merged > 25 {
        radius_route_total = radius_route_total + radius_route_merged;
    }
    return radius_route_total;
}

flow command_sandbox_errors_oasis_meadow_score(seed: i32) -> i32 ![]
{
    var radius_score_total = seed + 6;
    var radius_score_cursor = 0;
    while radius_score_cursor < 10 limit Iterations(10) {
        radius_score_total = radius_score_total + radius_score_cursor + 4;
        radius_score_cursor = radius_score_cursor + 1;
    }
    if radius_score_total % 2 == 0 {
        radius_score_total = radius_score_total + 27;
    } else {
        radius_score_total = radius_score_total - 5;
    }
    var radius_score_left = radius_score_total + seed;
    var radius_score_right = radius_score_left * 5;
    var radius_score_merged = radius_score_right - radius_score_left;
    if radius_score_merged > 25 {
        radius_score_total = radius_score_total + radius_score_merged;
    }
    return radius_score_total;
}

flow command_sandbox_errors_oasis_meadow_finish(seed: i32) -> i32 ![]
{
    var radius_finish_total = seed - 6;
    var radius_finish_cursor = 0;
    while radius_finish_cursor < 8 limit Iterations(8) {
        radius_finish_total = radius_finish_total + radius_finish_cursor + 4;
        radius_finish_cursor = radius_finish_cursor + 1;
    }
    if radius_finish_total % 2 == 0 {
        radius_finish_total = radius_finish_total + 27;
    } else {
        radius_finish_total = radius_finish_total - 5;
    }
    var radius_finish_left = radius_finish_total + seed;
    var radius_finish_right = radius_finish_left * 5;
    var radius_finish_merged = radius_finish_right - radius_finish_left;
    if radius_finish_merged > 25 {
        radius_finish_total = radius_finish_total + radius_finish_merged;
    }
    return radius_finish_total;
}

flow main(args: Array<string>) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var meadow_seed = 9;
    if args.len() > 0 {
        meadow_seed = meadow_seed + 1;
    } else {
        meadow_seed = meadow_seed + 2;
    }
    let meadow_result = command_sandbox_errors_oasis_meadow_entry(meadow_seed);
    if meadow_result > 0 {
        return 0;
    }
    return 1;
}
