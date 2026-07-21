module tests.compiler.effects.negative.handler_errors_026;


flow handler_errors_forge_drift_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var drift_total = handler_errors_forge_drift_prepare(seed);
    drift_total = drift_total + handler_errors_forge_drift_route(seed + 4);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            resume "fallback";
        }
    };
    let drift_adjust: i32 -> i32 = (value: i32) => value + 11;
    drift_total = drift_adjust(drift_total);
    drift_total = drift_total + handler_errors_forge_drift_score(3);
    drift_total = drift_total + handler_errors_forge_drift_finish(9);
    if drift_total > 466 {
        drift_total = drift_total - 10;
    } else {
        drift_total = drift_total + 5;
    }
    return drift_total;
}

flow handler_errors_forge_drift_prepare(seed: i32) -> i32 ![]
{
    var juliet_prepare_total = seed + 11;
    var juliet_prepare_cursor = 0;
    while juliet_prepare_cursor < 9 limit Iterations(9) {
        juliet_prepare_total = juliet_prepare_total + juliet_prepare_cursor + 6;
        juliet_prepare_cursor = juliet_prepare_cursor + 1;
    }
    if juliet_prepare_total % 2 == 0 {
        juliet_prepare_total = juliet_prepare_total + handler_errors_forge_drift_score(1);
    } else {
        juliet_prepare_total = juliet_prepare_total - 2;
    }
    var juliet_prepare_left = juliet_prepare_total + seed;
    var juliet_prepare_right = juliet_prepare_left * 4;
    var juliet_prepare_merged = juliet_prepare_right - juliet_prepare_left;
    if juliet_prepare_merged > 23 {
        juliet_prepare_total = juliet_prepare_total + juliet_prepare_merged;
    }
    return juliet_prepare_total;
}

flow handler_errors_forge_drift_route(seed: i32) -> i32 ![]
{
    var juliet_route_total = seed * 11;
    var juliet_route_cursor = 0;
    while juliet_route_cursor < 7 limit Iterations(7) {
        juliet_route_total = juliet_route_total + juliet_route_cursor + 6;
        juliet_route_cursor = juliet_route_cursor + 1;
    }
    if juliet_route_total % 2 == 0 {
        juliet_route_total = juliet_route_total + 17;
    } else {
        juliet_route_total = juliet_route_total - 2;
    }
    var juliet_route_left = juliet_route_total + seed;
    var juliet_route_right = juliet_route_left * 4;
    var juliet_route_merged = juliet_route_right - juliet_route_left;
    if juliet_route_merged > 23 {
        juliet_route_total = juliet_route_total + juliet_route_merged;
    }
    return juliet_route_total;
}

flow handler_errors_forge_drift_score(seed: i32) -> i32 ![]
{
    var juliet_score_total = seed + 11;
    var juliet_score_cursor = 0;
    while juliet_score_cursor < 12 limit Iterations(12) {
        juliet_score_total = juliet_score_total + juliet_score_cursor + 6;
        juliet_score_cursor = juliet_score_cursor + 1;
    }
    if juliet_score_total % 2 == 0 {
        juliet_score_total = juliet_score_total + 17;
    } else {
        juliet_score_total = juliet_score_total - 2;
    }
    var juliet_score_left = juliet_score_total + seed;
    var juliet_score_right = juliet_score_left * 4;
    var juliet_score_merged = juliet_score_right - juliet_score_left;
    if juliet_score_merged > 23 {
        juliet_score_total = juliet_score_total + juliet_score_merged;
    }
    return juliet_score_total;
}

flow handler_errors_forge_drift_finish(seed: i32) -> i32 ![]
{
    var juliet_finish_total = seed - 11;
    var juliet_finish_cursor = 0;
    while juliet_finish_cursor < 7 limit Iterations(7) {
        juliet_finish_total = juliet_finish_total + juliet_finish_cursor + 6;
        juliet_finish_cursor = juliet_finish_cursor + 1;
    }
    if juliet_finish_total % 2 == 0 {
        juliet_finish_total = juliet_finish_total + 17;
    } else {
        juliet_finish_total = juliet_finish_total - 2;
    }
    var juliet_finish_left = juliet_finish_total + seed;
    var juliet_finish_right = juliet_finish_left * 4;
    var juliet_finish_merged = juliet_finish_right - juliet_finish_left;
    if juliet_finish_merged > 23 {
        juliet_finish_total = juliet_finish_total + juliet_finish_merged;
    }
    return juliet_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var drift_seed = 9;
    if args.len() > 0 {
        drift_seed = drift_seed + 1;
    } else {
        drift_seed = drift_seed + 2;
    }
    let drift_result = handler_errors_forge_drift_entry(drift_seed);
    if drift_result > 0 {
        return 0;
    }
    return 1;
}
