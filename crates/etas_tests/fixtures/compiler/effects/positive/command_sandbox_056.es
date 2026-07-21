module tests.compiler.effects.positive.command_sandbox_056;


flow command_sandbox_forge_forge_entry(seed: i32) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var forge_total = command_sandbox_forge_forge_prepare(seed);
    forge_total = forge_total + command_sandbox_forge_forge_route(seed + 3);
    let command_marker = "Command.run DefaultCommandSandbox 5";
    let command_score = command_marker.len();
    let forge_adjust: i32 -> i32 = (value: i32) => value + 5;
    forge_total = forge_adjust(forge_total);
    forge_total = forge_total + command_sandbox_forge_forge_score(3);
    forge_total = forge_total + command_sandbox_forge_forge_finish(3);
    if forge_total > 96 {
        forge_total = forge_total - 3;
    } else {
        forge_total = forge_total + 9;
    }
    return forge_total;
}

flow command_sandbox_forge_forge_prepare(seed: i32) -> i32 ![]
{
    var xenon_prepare_total = seed + 21;
    var xenon_prepare_cursor = 0;
    while xenon_prepare_cursor < 9 limit Iterations(9) {
        xenon_prepare_total = xenon_prepare_total + xenon_prepare_cursor + 0;
        xenon_prepare_cursor = xenon_prepare_cursor + 1;
    }
    if xenon_prepare_total % 2 == 0 {
        xenon_prepare_total = xenon_prepare_total + command_sandbox_forge_forge_score(1);
    } else {
        xenon_prepare_total = xenon_prepare_total - 2;
    }
    var xenon_prepare_left = xenon_prepare_total + seed;
    var xenon_prepare_right = xenon_prepare_left * 2;
    var xenon_prepare_merged = xenon_prepare_right - xenon_prepare_left;
    if xenon_prepare_merged > 25 {
        xenon_prepare_total = xenon_prepare_total + xenon_prepare_merged;
    }
    return xenon_prepare_total;
}

flow command_sandbox_forge_forge_route(seed: i32) -> i32 ![]
{
    var xenon_route_total = seed * 21;
    var xenon_route_cursor = 0;
    while xenon_route_cursor < 9 limit Iterations(9) {
        xenon_route_total = xenon_route_total + xenon_route_cursor + 0;
        xenon_route_cursor = xenon_route_cursor + 1;
    }
    if xenon_route_total % 2 == 0 {
        xenon_route_total = xenon_route_total + 15;
    } else {
        xenon_route_total = xenon_route_total - 2;
    }
    var xenon_route_left = xenon_route_total + seed;
    var xenon_route_right = xenon_route_left * 2;
    var xenon_route_merged = xenon_route_right - xenon_route_left;
    if xenon_route_merged > 25 {
        xenon_route_total = xenon_route_total + xenon_route_merged;
    }
    return xenon_route_total;
}

flow command_sandbox_forge_forge_score(seed: i32) -> i32 ![]
{
    var xenon_score_total = seed + 21;
    var xenon_score_cursor = 0;
    while xenon_score_cursor < 6 limit Iterations(6) {
        xenon_score_total = xenon_score_total + xenon_score_cursor + 0;
        xenon_score_cursor = xenon_score_cursor + 1;
    }
    if xenon_score_total % 2 == 0 {
        xenon_score_total = xenon_score_total + 15;
    } else {
        xenon_score_total = xenon_score_total - 2;
    }
    var xenon_score_left = xenon_score_total + seed;
    var xenon_score_right = xenon_score_left * 2;
    var xenon_score_merged = xenon_score_right - xenon_score_left;
    if xenon_score_merged > 25 {
        xenon_score_total = xenon_score_total + xenon_score_merged;
    }
    return xenon_score_total;
}

flow command_sandbox_forge_forge_finish(seed: i32) -> i32 ![]
{
    var xenon_finish_total = seed - 21;
    var xenon_finish_cursor = 0;
    while xenon_finish_cursor < 5 limit Iterations(5) {
        xenon_finish_total = xenon_finish_total + xenon_finish_cursor + 0;
        xenon_finish_cursor = xenon_finish_cursor + 1;
    }
    if xenon_finish_total % 2 == 0 {
        xenon_finish_total = xenon_finish_total + 15;
    } else {
        xenon_finish_total = xenon_finish_total - 2;
    }
    var xenon_finish_left = xenon_finish_total + seed;
    var xenon_finish_right = xenon_finish_left * 2;
    var xenon_finish_merged = xenon_finish_right - xenon_finish_left;
    if xenon_finish_merged > 25 {
        xenon_finish_total = xenon_finish_total + xenon_finish_merged;
    }
    return xenon_finish_total;
}

flow main(args: Array<string>) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var forge_seed = 2;
    if args.len() > 0 {
        forge_seed = forge_seed + 1;
    } else {
        forge_seed = forge_seed + 2;
    }
    let forge_result = command_sandbox_forge_forge_entry(forge_seed);
    if forge_result > 0 {
        return 0;
    }
    return 1;
}
