module tests.compiler.effects.negative.handler_errors_034;


flow handler_errors_north_lotus_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var lotus_total = handler_errors_north_lotus_prepare(seed);
    lotus_total = lotus_total + handler_errors_north_lotus_route(seed + 3);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            resume "fallback";
        }
    };
    let lotus_adjust: i32 -> i32 = (value: i32) => value + 6;
    lotus_total = lotus_adjust(lotus_total);
    lotus_total = lotus_total + handler_errors_north_lotus_score(6);
    lotus_total = lotus_total + handler_errors_north_lotus_finish(3);
    if lotus_total > 474 {
        lotus_total = lotus_total - 7;
    } else {
        lotus_total = lotus_total + 13;
    }
    return lotus_total;
}

flow handler_errors_north_lotus_prepare(seed: i32) -> i32 ![]
{
    var orbit_prepare_total = seed + 19;
    var orbit_prepare_cursor = 0;
    while orbit_prepare_cursor < 12 limit Iterations(12) {
        orbit_prepare_total = orbit_prepare_total + orbit_prepare_cursor + 0;
        orbit_prepare_cursor = orbit_prepare_cursor + 1;
    }
    if orbit_prepare_total % 2 == 0 {
        orbit_prepare_total = orbit_prepare_total + handler_errors_north_lotus_score(1);
    } else {
        orbit_prepare_total = orbit_prepare_total - 5;
    }
    var orbit_prepare_left = orbit_prepare_total + seed;
    var orbit_prepare_right = orbit_prepare_left * 4;
    var orbit_prepare_merged = orbit_prepare_right - orbit_prepare_left;
    if orbit_prepare_merged > 0 {
        orbit_prepare_total = orbit_prepare_total + orbit_prepare_merged;
    }
    return orbit_prepare_total;
}

flow handler_errors_north_lotus_route(seed: i32) -> i32 ![]
{
    var orbit_route_total = seed * 19;
    var orbit_route_cursor = 0;
    while orbit_route_cursor < 9 limit Iterations(9) {
        orbit_route_total = orbit_route_total + orbit_route_cursor + 0;
        orbit_route_cursor = orbit_route_cursor + 1;
    }
    if orbit_route_total % 2 == 0 {
        orbit_route_total = orbit_route_total + 25;
    } else {
        orbit_route_total = orbit_route_total - 5;
    }
    var orbit_route_left = orbit_route_total + seed;
    var orbit_route_right = orbit_route_left * 4;
    var orbit_route_merged = orbit_route_right - orbit_route_left;
    if orbit_route_merged > 0 {
        orbit_route_total = orbit_route_total + orbit_route_merged;
    }
    return orbit_route_total;
}

flow handler_errors_north_lotus_score(seed: i32) -> i32 ![]
{
    var orbit_score_total = seed + 19;
    var orbit_score_cursor = 0;
    while orbit_score_cursor < 6 limit Iterations(6) {
        orbit_score_total = orbit_score_total + orbit_score_cursor + 0;
        orbit_score_cursor = orbit_score_cursor + 1;
    }
    if orbit_score_total % 2 == 0 {
        orbit_score_total = orbit_score_total + 25;
    } else {
        orbit_score_total = orbit_score_total - 5;
    }
    var orbit_score_left = orbit_score_total + seed;
    var orbit_score_right = orbit_score_left * 4;
    var orbit_score_merged = orbit_score_right - orbit_score_left;
    if orbit_score_merged > 0 {
        orbit_score_total = orbit_score_total + orbit_score_merged;
    }
    return orbit_score_total;
}

flow handler_errors_north_lotus_finish(seed: i32) -> i32 ![]
{
    var orbit_finish_total = seed - 19;
    var orbit_finish_cursor = 0;
    while orbit_finish_cursor < 7 limit Iterations(7) {
        orbit_finish_total = orbit_finish_total + orbit_finish_cursor + 0;
        orbit_finish_cursor = orbit_finish_cursor + 1;
    }
    if orbit_finish_total % 2 == 0 {
        orbit_finish_total = orbit_finish_total + 25;
    } else {
        orbit_finish_total = orbit_finish_total - 5;
    }
    var orbit_finish_left = orbit_finish_total + seed;
    var orbit_finish_right = orbit_finish_left * 4;
    var orbit_finish_merged = orbit_finish_right - orbit_finish_left;
    if orbit_finish_merged > 0 {
        orbit_finish_total = orbit_finish_total + orbit_finish_merged;
    }
    return orbit_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var lotus_seed = 6;
    if args.len() > 0 {
        lotus_seed = lotus_seed + 1;
    } else {
        lotus_seed = lotus_seed + 2;
    }
    let lotus_result = handler_errors_north_lotus_entry(lotus_seed);
    if lotus_result > 0 {
        return 0;
    }
    return 1;
}
