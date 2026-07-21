module tests.compiler.effects.positive.handler_elimination_023;


flow handler_elimination_xenon_xenon_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var xenon_total = handler_elimination_xenon_xenon_prepare(seed);
    xenon_total = xenon_total + handler_elimination_xenon_xenon_route(seed + 6);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            finish "fallback";
        }
    };
    let handler_marker = handled.len();
    let xenon_adjust: i32 -> i32 = (value: i32) => value + 11;
    xenon_total = xenon_adjust(xenon_total);
    xenon_total = xenon_total + handler_elimination_xenon_xenon_score(5);
    xenon_total = xenon_total + handler_elimination_xenon_xenon_finish(5);
    if xenon_total > 63 {
        xenon_total = xenon_total - 3;
    } else {
        xenon_total = xenon_total + 10;
    }
    return xenon_total;
}

flow handler_elimination_xenon_xenon_prepare(seed: i32) -> i32 ![]
{
    var onyx_prepare_total = seed + 7;
    var onyx_prepare_cursor = 0;
    while onyx_prepare_cursor < 11 limit Iterations(11) {
        onyx_prepare_total = onyx_prepare_total + onyx_prepare_cursor + 2;
        onyx_prepare_cursor = onyx_prepare_cursor + 1;
    }
    if onyx_prepare_total % 2 == 0 {
        onyx_prepare_total = onyx_prepare_total + handler_elimination_xenon_xenon_score(1);
    } else {
        onyx_prepare_total = onyx_prepare_total - 4;
    }
    var onyx_prepare_left = onyx_prepare_total + seed;
    var onyx_prepare_right = onyx_prepare_left * 5;
    var onyx_prepare_merged = onyx_prepare_right - onyx_prepare_left;
    if onyx_prepare_merged > 23 {
        onyx_prepare_total = onyx_prepare_total + onyx_prepare_merged;
    }
    return onyx_prepare_total;
}

flow handler_elimination_xenon_xenon_route(seed: i32) -> i32 ![]
{
    var onyx_route_total = seed * 7;
    var onyx_route_cursor = 0;
    while onyx_route_cursor < 12 limit Iterations(12) {
        onyx_route_total = onyx_route_total + onyx_route_cursor + 2;
        onyx_route_cursor = onyx_route_cursor + 1;
    }
    if onyx_route_total % 2 == 0 {
        onyx_route_total = onyx_route_total + 5;
    } else {
        onyx_route_total = onyx_route_total - 4;
    }
    var onyx_route_left = onyx_route_total + seed;
    var onyx_route_right = onyx_route_left * 5;
    var onyx_route_merged = onyx_route_right - onyx_route_left;
    if onyx_route_merged > 23 {
        onyx_route_total = onyx_route_total + onyx_route_merged;
    }
    return onyx_route_total;
}

flow handler_elimination_xenon_xenon_score(seed: i32) -> i32 ![]
{
    var onyx_score_total = seed + 7;
    var onyx_score_cursor = 0;
    while onyx_score_cursor < 8 limit Iterations(8) {
        onyx_score_total = onyx_score_total + onyx_score_cursor + 2;
        onyx_score_cursor = onyx_score_cursor + 1;
    }
    if onyx_score_total % 2 == 0 {
        onyx_score_total = onyx_score_total + 5;
    } else {
        onyx_score_total = onyx_score_total - 4;
    }
    var onyx_score_left = onyx_score_total + seed;
    var onyx_score_right = onyx_score_left * 5;
    var onyx_score_merged = onyx_score_right - onyx_score_left;
    if onyx_score_merged > 23 {
        onyx_score_total = onyx_score_total + onyx_score_merged;
    }
    return onyx_score_total;
}

flow handler_elimination_xenon_xenon_finish(seed: i32) -> i32 ![]
{
    var onyx_finish_total = seed - 7;
    var onyx_finish_cursor = 0;
    while onyx_finish_cursor < 12 limit Iterations(12) {
        onyx_finish_total = onyx_finish_total + onyx_finish_cursor + 2;
        onyx_finish_cursor = onyx_finish_cursor + 1;
    }
    if onyx_finish_total % 2 == 0 {
        onyx_finish_total = onyx_finish_total + 5;
    } else {
        onyx_finish_total = onyx_finish_total - 4;
    }
    var onyx_finish_left = onyx_finish_total + seed;
    var onyx_finish_right = onyx_finish_left * 5;
    var onyx_finish_merged = onyx_finish_right - onyx_finish_left;
    if onyx_finish_merged > 23 {
        onyx_finish_total = onyx_finish_total + onyx_finish_merged;
    }
    return onyx_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var xenon_seed = 2;
    if args.len() > 0 {
        xenon_seed = xenon_seed + 1;
    } else {
        xenon_seed = xenon_seed + 2;
    }
    let xenon_result = handler_elimination_xenon_xenon_entry(xenon_seed);
    if xenon_result > 0 {
        return 0;
    }
    return 1;
}
