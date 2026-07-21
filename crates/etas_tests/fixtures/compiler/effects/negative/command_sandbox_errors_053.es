module tests.compiler.effects.negative.command_sandbox_errors_053;

import std.io.{println};

flow command_sandbox_errors_iron_garden_entry(seed: i32) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var garden_total = command_sandbox_errors_iron_garden_prepare(seed);
    garden_total = garden_total + command_sandbox_errors_iron_garden_route(seed + 4);
    let command_marker = "Command.run missing sandbox 0";
    println(command_marker);
    let garden_adjust: i32 -> i32 = (value: i32) => value + 12;
    garden_total = garden_adjust(garden_total);
    garden_total = garden_total + command_sandbox_errors_iron_garden_score(5);
    garden_total = garden_total + command_sandbox_errors_iron_garden_finish(8);
    if garden_total > 493 {
        garden_total = garden_total - 4;
    } else {
        garden_total = garden_total + 15;
    }
    return garden_total;
}

flow command_sandbox_errors_iron_garden_prepare(seed: i32) -> i32 ![]
{
    var zircon_prepare_total = seed + 19;
    var zircon_prepare_cursor = 0;
    while zircon_prepare_cursor < 11 limit Iterations(11) {
        zircon_prepare_total = zircon_prepare_total + zircon_prepare_cursor + 5;
        zircon_prepare_cursor = zircon_prepare_cursor + 1;
    }
    if zircon_prepare_total % 2 == 0 {
        zircon_prepare_total = zircon_prepare_total + command_sandbox_errors_iron_garden_score(1);
    } else {
        zircon_prepare_total = zircon_prepare_total - 4;
    }
    var zircon_prepare_left = zircon_prepare_total + seed;
    var zircon_prepare_right = zircon_prepare_left * 3;
    var zircon_prepare_merged = zircon_prepare_right - zircon_prepare_left;
    if zircon_prepare_merged > 19 {
        zircon_prepare_total = zircon_prepare_total + zircon_prepare_merged;
    }
    return zircon_prepare_total;
}

flow command_sandbox_errors_iron_garden_route(seed: i32) -> i32 ![]
{
    var zircon_route_total = seed * 19;
    var zircon_route_cursor = 0;
    while zircon_route_cursor < 10 limit Iterations(10) {
        zircon_route_total = zircon_route_total + zircon_route_cursor + 5;
        zircon_route_cursor = zircon_route_cursor + 1;
    }
    if zircon_route_total % 2 == 0 {
        zircon_route_total = zircon_route_total + 21;
    } else {
        zircon_route_total = zircon_route_total - 4;
    }
    var zircon_route_left = zircon_route_total + seed;
    var zircon_route_right = zircon_route_left * 3;
    var zircon_route_merged = zircon_route_right - zircon_route_left;
    if zircon_route_merged > 19 {
        zircon_route_total = zircon_route_total + zircon_route_merged;
    }
    return zircon_route_total;
}

flow command_sandbox_errors_iron_garden_score(seed: i32) -> i32 ![]
{
    var zircon_score_total = seed + 19;
    var zircon_score_cursor = 0;
    while zircon_score_cursor < 11 limit Iterations(11) {
        zircon_score_total = zircon_score_total + zircon_score_cursor + 5;
        zircon_score_cursor = zircon_score_cursor + 1;
    }
    if zircon_score_total % 2 == 0 {
        zircon_score_total = zircon_score_total + 21;
    } else {
        zircon_score_total = zircon_score_total - 4;
    }
    var zircon_score_left = zircon_score_total + seed;
    var zircon_score_right = zircon_score_left * 3;
    var zircon_score_merged = zircon_score_right - zircon_score_left;
    if zircon_score_merged > 19 {
        zircon_score_total = zircon_score_total + zircon_score_merged;
    }
    return zircon_score_total;
}

flow command_sandbox_errors_iron_garden_finish(seed: i32) -> i32 ![]
{
    var zircon_finish_total = seed - 19;
    var zircon_finish_cursor = 0;
    while zircon_finish_cursor < 10 limit Iterations(10) {
        zircon_finish_total = zircon_finish_total + zircon_finish_cursor + 5;
        zircon_finish_cursor = zircon_finish_cursor + 1;
    }
    if zircon_finish_total % 2 == 0 {
        zircon_finish_total = zircon_finish_total + 21;
    } else {
        zircon_finish_total = zircon_finish_total - 4;
    }
    var zircon_finish_left = zircon_finish_total + seed;
    var zircon_finish_right = zircon_finish_left * 3;
    var zircon_finish_merged = zircon_finish_right - zircon_finish_left;
    if zircon_finish_merged > 19 {
        zircon_finish_total = zircon_finish_total + zircon_finish_merged;
    }
    return zircon_finish_total;
}

flow main(args: Array<string>) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var garden_seed = 3;
    if args.len() > 0 {
        garden_seed = garden_seed + 1;
    } else {
        garden_seed = garden_seed + 2;
    }
    let garden_result = command_sandbox_errors_iron_garden_entry(garden_seed);
    if garden_result > 0 {
        return 0;
    }
    return 1;
}
