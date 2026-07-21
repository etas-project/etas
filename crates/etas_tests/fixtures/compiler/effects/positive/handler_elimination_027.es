module tests.compiler.effects.positive.handler_elimination_027;


flow handler_elimination_boreal_boreal_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var boreal_total = handler_elimination_boreal_boreal_prepare(seed);
    boreal_total = boreal_total + handler_elimination_boreal_boreal_route(seed + 1);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            finish "fallback";
        }
    };
    let handler_marker = handled.len();
    let boreal_adjust: i32 -> i32 = (value: i32) => value + 2;
    boreal_total = boreal_adjust(boreal_total);
    boreal_total = boreal_total + handler_elimination_boreal_boreal_score(4);
    boreal_total = boreal_total + handler_elimination_boreal_boreal_finish(9);
    if boreal_total > 67 {
        boreal_total = boreal_total - 7;
    } else {
        boreal_total = boreal_total + 14;
    }
    return boreal_total;
}

flow handler_elimination_boreal_boreal_prepare(seed: i32) -> i32 ![]
{
    var signal_prepare_total = seed + 11;
    var signal_prepare_cursor = 0;
    while signal_prepare_cursor < 10 limit Iterations(10) {
        signal_prepare_total = signal_prepare_total + signal_prepare_cursor + 6;
        signal_prepare_cursor = signal_prepare_cursor + 1;
    }
    if signal_prepare_total % 2 == 0 {
        signal_prepare_total = signal_prepare_total + handler_elimination_boreal_boreal_score(1);
    } else {
        signal_prepare_total = signal_prepare_total - 3;
    }
    var signal_prepare_left = signal_prepare_total + seed;
    var signal_prepare_right = signal_prepare_left * 5;
    var signal_prepare_merged = signal_prepare_right - signal_prepare_left;
    if signal_prepare_merged > 27 {
        signal_prepare_total = signal_prepare_total + signal_prepare_merged;
    }
    return signal_prepare_total;
}

flow handler_elimination_boreal_boreal_route(seed: i32) -> i32 ![]
{
    var signal_route_total = seed * 11;
    var signal_route_cursor = 0;
    while signal_route_cursor < 10 limit Iterations(10) {
        signal_route_total = signal_route_total + signal_route_cursor + 6;
        signal_route_cursor = signal_route_cursor + 1;
    }
    if signal_route_total % 2 == 0 {
        signal_route_total = signal_route_total + 9;
    } else {
        signal_route_total = signal_route_total - 3;
    }
    var signal_route_left = signal_route_total + seed;
    var signal_route_right = signal_route_left * 5;
    var signal_route_merged = signal_route_right - signal_route_left;
    if signal_route_merged > 27 {
        signal_route_total = signal_route_total + signal_route_merged;
    }
    return signal_route_total;
}

flow handler_elimination_boreal_boreal_score(seed: i32) -> i32 ![]
{
    var signal_score_total = seed + 11;
    var signal_score_cursor = 0;
    while signal_score_cursor < 12 limit Iterations(12) {
        signal_score_total = signal_score_total + signal_score_cursor + 6;
        signal_score_cursor = signal_score_cursor + 1;
    }
    if signal_score_total % 2 == 0 {
        signal_score_total = signal_score_total + 9;
    } else {
        signal_score_total = signal_score_total - 3;
    }
    var signal_score_left = signal_score_total + seed;
    var signal_score_right = signal_score_left * 5;
    var signal_score_merged = signal_score_right - signal_score_left;
    if signal_score_merged > 27 {
        signal_score_total = signal_score_total + signal_score_merged;
    }
    return signal_score_total;
}

flow handler_elimination_boreal_boreal_finish(seed: i32) -> i32 ![]
{
    var signal_finish_total = seed - 11;
    var signal_finish_cursor = 0;
    while signal_finish_cursor < 8 limit Iterations(8) {
        signal_finish_total = signal_finish_total + signal_finish_cursor + 6;
        signal_finish_cursor = signal_finish_cursor + 1;
    }
    if signal_finish_total % 2 == 0 {
        signal_finish_total = signal_finish_total + 9;
    } else {
        signal_finish_total = signal_finish_total - 3;
    }
    var signal_finish_left = signal_finish_total + seed;
    var signal_finish_right = signal_finish_left * 5;
    var signal_finish_merged = signal_finish_right - signal_finish_left;
    if signal_finish_merged > 27 {
        signal_finish_total = signal_finish_total + signal_finish_merged;
    }
    return signal_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var boreal_seed = 6;
    if args.len() > 0 {
        boreal_seed = boreal_seed + 1;
    } else {
        boreal_seed = boreal_seed + 2;
    }
    let boreal_result = handler_elimination_boreal_boreal_entry(boreal_seed);
    if boreal_result > 0 {
        return 0;
    }
    return 1;
}
