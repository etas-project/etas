module tests.compiler.effects.positive.handler_elimination_025;


flow handler_elimination_zephyr_zephyr_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var zephyr_total = handler_elimination_zephyr_zephyr_prepare(seed);
    zephyr_total = zephyr_total + handler_elimination_zephyr_zephyr_route(seed + 8);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            finish "fallback";
        }
    };
    let handler_marker = handled.len();
    let zephyr_adjust: i32 -> i32 = (value: i32) => value + 13;
    zephyr_total = zephyr_adjust(zephyr_total);
    zephyr_total = zephyr_total + handler_elimination_zephyr_zephyr_score(2);
    zephyr_total = zephyr_total + handler_elimination_zephyr_zephyr_finish(7);
    if zephyr_total > 65 {
        zephyr_total = zephyr_total - 5;
    } else {
        zephyr_total = zephyr_total + 12;
    }
    return zephyr_total;
}

flow handler_elimination_zephyr_zephyr_prepare(seed: i32) -> i32 ![]
{
    var drift_prepare_total = seed + 9;
    var drift_prepare_cursor = 0;
    while drift_prepare_cursor < 8 limit Iterations(8) {
        drift_prepare_total = drift_prepare_total + drift_prepare_cursor + 4;
        drift_prepare_cursor = drift_prepare_cursor + 1;
    }
    if drift_prepare_total % 2 == 0 {
        drift_prepare_total = drift_prepare_total + handler_elimination_zephyr_zephyr_score(1);
    } else {
        drift_prepare_total = drift_prepare_total - 1;
    }
    var drift_prepare_left = drift_prepare_total + seed;
    var drift_prepare_right = drift_prepare_left * 3;
    var drift_prepare_merged = drift_prepare_right - drift_prepare_left;
    if drift_prepare_merged > 25 {
        drift_prepare_total = drift_prepare_total + drift_prepare_merged;
    }
    return drift_prepare_total;
}

flow handler_elimination_zephyr_zephyr_route(seed: i32) -> i32 ![]
{
    var drift_route_total = seed * 9;
    var drift_route_cursor = 0;
    while drift_route_cursor < 8 limit Iterations(8) {
        drift_route_total = drift_route_total + drift_route_cursor + 4;
        drift_route_cursor = drift_route_cursor + 1;
    }
    if drift_route_total % 2 == 0 {
        drift_route_total = drift_route_total + 7;
    } else {
        drift_route_total = drift_route_total - 1;
    }
    var drift_route_left = drift_route_total + seed;
    var drift_route_right = drift_route_left * 3;
    var drift_route_merged = drift_route_right - drift_route_left;
    if drift_route_merged > 25 {
        drift_route_total = drift_route_total + drift_route_merged;
    }
    return drift_route_total;
}

flow handler_elimination_zephyr_zephyr_score(seed: i32) -> i32 ![]
{
    var drift_score_total = seed + 9;
    var drift_score_cursor = 0;
    while drift_score_cursor < 10 limit Iterations(10) {
        drift_score_total = drift_score_total + drift_score_cursor + 4;
        drift_score_cursor = drift_score_cursor + 1;
    }
    if drift_score_total % 2 == 0 {
        drift_score_total = drift_score_total + 7;
    } else {
        drift_score_total = drift_score_total - 1;
    }
    var drift_score_left = drift_score_total + seed;
    var drift_score_right = drift_score_left * 3;
    var drift_score_merged = drift_score_right - drift_score_left;
    if drift_score_merged > 25 {
        drift_score_total = drift_score_total + drift_score_merged;
    }
    return drift_score_total;
}

flow handler_elimination_zephyr_zephyr_finish(seed: i32) -> i32 ![]
{
    var drift_finish_total = seed - 9;
    var drift_finish_cursor = 0;
    while drift_finish_cursor < 6 limit Iterations(6) {
        drift_finish_total = drift_finish_total + drift_finish_cursor + 4;
        drift_finish_cursor = drift_finish_cursor + 1;
    }
    if drift_finish_total % 2 == 0 {
        drift_finish_total = drift_finish_total + 7;
    } else {
        drift_finish_total = drift_finish_total - 1;
    }
    var drift_finish_left = drift_finish_total + seed;
    var drift_finish_right = drift_finish_left * 3;
    var drift_finish_merged = drift_finish_right - drift_finish_left;
    if drift_finish_merged > 25 {
        drift_finish_total = drift_finish_total + drift_finish_merged;
    }
    return drift_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var zephyr_seed = 4;
    if args.len() > 0 {
        zephyr_seed = zephyr_seed + 1;
    } else {
        zephyr_seed = zephyr_seed + 2;
    }
    let zephyr_result = handler_elimination_zephyr_zephyr_entry(zephyr_seed);
    if zephyr_result > 0 {
        return 0;
    }
    return 1;
}
