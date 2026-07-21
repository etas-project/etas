module tests.compiler.effects.positive.command_sandbox_055;


flow command_sandbox_equinox_equinox_entry(seed: i32) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var equinox_total = command_sandbox_equinox_equinox_prepare(seed);
    equinox_total = equinox_total + command_sandbox_equinox_equinox_route(seed + 2);
    let command_marker = "Command.run DefaultCommandSandbox 4";
    let command_score = command_marker.len();
    let equinox_adjust: i32 -> i32 = (value: i32) => value + 4;
    equinox_total = equinox_adjust(equinox_total);
    equinox_total = equinox_total + command_sandbox_equinox_equinox_score(2);
    equinox_total = equinox_total + command_sandbox_equinox_equinox_finish(9);
    if equinox_total > 95 {
        equinox_total = equinox_total - 2;
    } else {
        equinox_total = equinox_total + 8;
    }
    return equinox_total;
}

flow command_sandbox_equinox_equinox_prepare(seed: i32) -> i32 ![]
{
    var quartz_prepare_total = seed + 20;
    var quartz_prepare_cursor = 0;
    while quartz_prepare_cursor < 8 limit Iterations(8) {
        quartz_prepare_total = quartz_prepare_total + quartz_prepare_cursor + 6;
        quartz_prepare_cursor = quartz_prepare_cursor + 1;
    }
    if quartz_prepare_total % 2 == 0 {
        quartz_prepare_total = quartz_prepare_total + command_sandbox_equinox_equinox_score(1);
    } else {
        quartz_prepare_total = quartz_prepare_total - 1;
    }
    var quartz_prepare_left = quartz_prepare_total + seed;
    var quartz_prepare_right = quartz_prepare_left * 5;
    var quartz_prepare_merged = quartz_prepare_right - quartz_prepare_left;
    if quartz_prepare_merged > 24 {
        quartz_prepare_total = quartz_prepare_total + quartz_prepare_merged;
    }
    return quartz_prepare_total;
}

flow command_sandbox_equinox_equinox_route(seed: i32) -> i32 ![]
{
    var quartz_route_total = seed * 20;
    var quartz_route_cursor = 0;
    while quartz_route_cursor < 8 limit Iterations(8) {
        quartz_route_total = quartz_route_total + quartz_route_cursor + 6;
        quartz_route_cursor = quartz_route_cursor + 1;
    }
    if quartz_route_total % 2 == 0 {
        quartz_route_total = quartz_route_total + 14;
    } else {
        quartz_route_total = quartz_route_total - 1;
    }
    var quartz_route_left = quartz_route_total + seed;
    var quartz_route_right = quartz_route_left * 5;
    var quartz_route_merged = quartz_route_right - quartz_route_left;
    if quartz_route_merged > 24 {
        quartz_route_total = quartz_route_total + quartz_route_merged;
    }
    return quartz_route_total;
}

flow command_sandbox_equinox_equinox_score(seed: i32) -> i32 ![]
{
    var quartz_score_total = seed + 20;
    var quartz_score_cursor = 0;
    while quartz_score_cursor < 12 limit Iterations(12) {
        quartz_score_total = quartz_score_total + quartz_score_cursor + 6;
        quartz_score_cursor = quartz_score_cursor + 1;
    }
    if quartz_score_total % 2 == 0 {
        quartz_score_total = quartz_score_total + 14;
    } else {
        quartz_score_total = quartz_score_total - 1;
    }
    var quartz_score_left = quartz_score_total + seed;
    var quartz_score_right = quartz_score_left * 5;
    var quartz_score_merged = quartz_score_right - quartz_score_left;
    if quartz_score_merged > 24 {
        quartz_score_total = quartz_score_total + quartz_score_merged;
    }
    return quartz_score_total;
}

flow command_sandbox_equinox_equinox_finish(seed: i32) -> i32 ![]
{
    var quartz_finish_total = seed - 20;
    var quartz_finish_cursor = 0;
    while quartz_finish_cursor < 12 limit Iterations(12) {
        quartz_finish_total = quartz_finish_total + quartz_finish_cursor + 6;
        quartz_finish_cursor = quartz_finish_cursor + 1;
    }
    if quartz_finish_total % 2 == 0 {
        quartz_finish_total = quartz_finish_total + 14;
    } else {
        quartz_finish_total = quartz_finish_total - 1;
    }
    var quartz_finish_left = quartz_finish_total + seed;
    var quartz_finish_right = quartz_finish_left * 5;
    var quartz_finish_merged = quartz_finish_right - quartz_finish_left;
    if quartz_finish_merged > 24 {
        quartz_finish_total = quartz_finish_total + quartz_finish_merged;
    }
    return quartz_finish_total;
}

flow main(args: Array<string>) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var equinox_seed = 1;
    if args.len() > 0 {
        equinox_seed = equinox_seed + 1;
    } else {
        equinox_seed = equinox_seed + 2;
    }
    let equinox_result = command_sandbox_equinox_equinox_entry(equinox_seed);
    if equinox_result > 0 {
        return 0;
    }
    return 1;
}
