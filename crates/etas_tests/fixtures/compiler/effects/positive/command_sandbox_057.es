module tests.compiler.effects.positive.command_sandbox_057;


flow command_sandbox_grove_grove_entry(seed: i32) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var grove_total = command_sandbox_grove_grove_prepare(seed);
    grove_total = grove_total + command_sandbox_grove_grove_route(seed + 4);
    let command_marker = "Command.run DefaultCommandSandbox 6";
    let command_score = command_marker.len();
    let grove_adjust: i32 -> i32 = (value: i32) => value + 6;
    grove_total = grove_adjust(grove_total);
    grove_total = grove_total + command_sandbox_grove_grove_score(4);
    grove_total = grove_total + command_sandbox_grove_grove_finish(4);
    if grove_total > 97 {
        grove_total = grove_total - 4;
    } else {
        grove_total = grove_total + 10;
    }
    return grove_total;
}

flow command_sandbox_grove_grove_prepare(seed: i32) -> i32 ![]
{
    var ember_prepare_total = seed + 3;
    var ember_prepare_cursor = 0;
    while ember_prepare_cursor < 10 limit Iterations(10) {
        ember_prepare_total = ember_prepare_total + ember_prepare_cursor + 1;
        ember_prepare_cursor = ember_prepare_cursor + 1;
    }
    if ember_prepare_total % 2 == 0 {
        ember_prepare_total = ember_prepare_total + command_sandbox_grove_grove_score(1);
    } else {
        ember_prepare_total = ember_prepare_total - 3;
    }
    var ember_prepare_left = ember_prepare_total + seed;
    var ember_prepare_right = ember_prepare_left * 3;
    var ember_prepare_merged = ember_prepare_right - ember_prepare_left;
    if ember_prepare_merged > 26 {
        ember_prepare_total = ember_prepare_total + ember_prepare_merged;
    }
    return ember_prepare_total;
}

flow command_sandbox_grove_grove_route(seed: i32) -> i32 ![]
{
    var ember_route_total = seed * 3;
    var ember_route_cursor = 0;
    while ember_route_cursor < 10 limit Iterations(10) {
        ember_route_total = ember_route_total + ember_route_cursor + 1;
        ember_route_cursor = ember_route_cursor + 1;
    }
    if ember_route_total % 2 == 0 {
        ember_route_total = ember_route_total + 16;
    } else {
        ember_route_total = ember_route_total - 3;
    }
    var ember_route_left = ember_route_total + seed;
    var ember_route_right = ember_route_left * 3;
    var ember_route_merged = ember_route_right - ember_route_left;
    if ember_route_merged > 26 {
        ember_route_total = ember_route_total + ember_route_merged;
    }
    return ember_route_total;
}

flow command_sandbox_grove_grove_score(seed: i32) -> i32 ![]
{
    var ember_score_total = seed + 3;
    var ember_score_cursor = 0;
    while ember_score_cursor < 7 limit Iterations(7) {
        ember_score_total = ember_score_total + ember_score_cursor + 1;
        ember_score_cursor = ember_score_cursor + 1;
    }
    if ember_score_total % 2 == 0 {
        ember_score_total = ember_score_total + 16;
    } else {
        ember_score_total = ember_score_total - 3;
    }
    var ember_score_left = ember_score_total + seed;
    var ember_score_right = ember_score_left * 3;
    var ember_score_merged = ember_score_right - ember_score_left;
    if ember_score_merged > 26 {
        ember_score_total = ember_score_total + ember_score_merged;
    }
    return ember_score_total;
}

flow command_sandbox_grove_grove_finish(seed: i32) -> i32 ![]
{
    var ember_finish_total = seed - 3;
    var ember_finish_cursor = 0;
    while ember_finish_cursor < 6 limit Iterations(6) {
        ember_finish_total = ember_finish_total + ember_finish_cursor + 1;
        ember_finish_cursor = ember_finish_cursor + 1;
    }
    if ember_finish_total % 2 == 0 {
        ember_finish_total = ember_finish_total + 16;
    } else {
        ember_finish_total = ember_finish_total - 3;
    }
    var ember_finish_left = ember_finish_total + seed;
    var ember_finish_right = ember_finish_left * 3;
    var ember_finish_merged = ember_finish_right - ember_finish_left;
    if ember_finish_merged > 26 {
        ember_finish_total = ember_finish_total + ember_finish_merged;
    }
    return ember_finish_total;
}

flow main(args: Array<string>) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var grove_seed = 3;
    if args.len() > 0 {
        grove_seed = grove_seed + 1;
    } else {
        grove_seed = grove_seed + 2;
    }
    let grove_result = command_sandbox_grove_grove_entry(grove_seed);
    if grove_result > 0 {
        return 0;
    }
    return 1;
}
