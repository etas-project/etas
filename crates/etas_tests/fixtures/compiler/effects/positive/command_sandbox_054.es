module tests.compiler.effects.positive.command_sandbox_054;


flow command_sandbox_drift_drift_entry(seed: i32) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var drift_total = command_sandbox_drift_drift_prepare(seed);
    drift_total = drift_total + command_sandbox_drift_drift_route(seed + 1);
    let command_marker = "Command.run DefaultCommandSandbox 3";
    let command_score = command_marker.len();
    let drift_adjust: i32 -> i32 = (value: i32) => value + 3;
    drift_total = drift_adjust(drift_total);
    drift_total = drift_total + command_sandbox_drift_drift_score(6);
    drift_total = drift_total + command_sandbox_drift_drift_finish(8);
    if drift_total > 94 {
        drift_total = drift_total - 12;
    } else {
        drift_total = drift_total + 7;
    }
    return drift_total;
}

flow command_sandbox_drift_drift_prepare(seed: i32) -> i32 ![]
{
    var juliet_prepare_total = seed + 19;
    var juliet_prepare_cursor = 0;
    while juliet_prepare_cursor < 12 limit Iterations(12) {
        juliet_prepare_total = juliet_prepare_total + juliet_prepare_cursor + 5;
        juliet_prepare_cursor = juliet_prepare_cursor + 1;
    }
    if juliet_prepare_total % 2 == 0 {
        juliet_prepare_total = juliet_prepare_total + command_sandbox_drift_drift_score(1);
    } else {
        juliet_prepare_total = juliet_prepare_total - 5;
    }
    var juliet_prepare_left = juliet_prepare_total + seed;
    var juliet_prepare_right = juliet_prepare_left * 4;
    var juliet_prepare_merged = juliet_prepare_right - juliet_prepare_left;
    if juliet_prepare_merged > 23 {
        juliet_prepare_total = juliet_prepare_total + juliet_prepare_merged;
    }
    return juliet_prepare_total;
}

flow command_sandbox_drift_drift_route(seed: i32) -> i32 ![]
{
    var juliet_route_total = seed * 19;
    var juliet_route_cursor = 0;
    while juliet_route_cursor < 7 limit Iterations(7) {
        juliet_route_total = juliet_route_total + juliet_route_cursor + 5;
        juliet_route_cursor = juliet_route_cursor + 1;
    }
    if juliet_route_total % 2 == 0 {
        juliet_route_total = juliet_route_total + 13;
    } else {
        juliet_route_total = juliet_route_total - 5;
    }
    var juliet_route_left = juliet_route_total + seed;
    var juliet_route_right = juliet_route_left * 4;
    var juliet_route_merged = juliet_route_right - juliet_route_left;
    if juliet_route_merged > 23 {
        juliet_route_total = juliet_route_total + juliet_route_merged;
    }
    return juliet_route_total;
}

flow command_sandbox_drift_drift_score(seed: i32) -> i32 ![]
{
    var juliet_score_total = seed + 19;
    var juliet_score_cursor = 0;
    while juliet_score_cursor < 11 limit Iterations(11) {
        juliet_score_total = juliet_score_total + juliet_score_cursor + 5;
        juliet_score_cursor = juliet_score_cursor + 1;
    }
    if juliet_score_total % 2 == 0 {
        juliet_score_total = juliet_score_total + 13;
    } else {
        juliet_score_total = juliet_score_total - 5;
    }
    var juliet_score_left = juliet_score_total + seed;
    var juliet_score_right = juliet_score_left * 4;
    var juliet_score_merged = juliet_score_right - juliet_score_left;
    if juliet_score_merged > 23 {
        juliet_score_total = juliet_score_total + juliet_score_merged;
    }
    return juliet_score_total;
}

flow command_sandbox_drift_drift_finish(seed: i32) -> i32 ![]
{
    var juliet_finish_total = seed - 19;
    var juliet_finish_cursor = 0;
    while juliet_finish_cursor < 11 limit Iterations(11) {
        juliet_finish_total = juliet_finish_total + juliet_finish_cursor + 5;
        juliet_finish_cursor = juliet_finish_cursor + 1;
    }
    if juliet_finish_total % 2 == 0 {
        juliet_finish_total = juliet_finish_total + 13;
    } else {
        juliet_finish_total = juliet_finish_total - 5;
    }
    var juliet_finish_left = juliet_finish_total + seed;
    var juliet_finish_right = juliet_finish_left * 4;
    var juliet_finish_merged = juliet_finish_right - juliet_finish_left;
    if juliet_finish_merged > 23 {
        juliet_finish_total = juliet_finish_total + juliet_finish_merged;
    }
    return juliet_finish_total;
}

flow main(args: Array<string>) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var drift_seed = 11;
    if args.len() > 0 {
        drift_seed = drift_seed + 1;
    } else {
        drift_seed = drift_seed + 2;
    }
    let drift_result = command_sandbox_drift_drift_entry(drift_seed);
    if drift_result > 0 {
        return 0;
    }
    return 1;
}
