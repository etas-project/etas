module tests.compiler.effects.negative.handler_errors_030;


flow handler_errors_jade_helios_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var helios_total = handler_errors_jade_helios_prepare(seed);
    helios_total = helios_total + handler_errors_jade_helios_route(seed + 8);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            resume "fallback";
        }
    };
    let helios_adjust: i32 -> i32 = (value: i32) => value + 2;
    helios_total = helios_adjust(helios_total);
    helios_total = helios_total + handler_errors_jade_helios_score(2);
    helios_total = helios_total + handler_errors_jade_helios_finish(6);
    if helios_total > 470 {
        helios_total = helios_total - 3;
    } else {
        helios_total = helios_total + 9;
    }
    return helios_total;
}

flow handler_errors_jade_helios_prepare(seed: i32) -> i32 ![]
{
    var lagoon_prepare_total = seed + 15;
    var lagoon_prepare_cursor = 0;
    while lagoon_prepare_cursor < 8 limit Iterations(8) {
        lagoon_prepare_total = lagoon_prepare_total + lagoon_prepare_cursor + 3;
        lagoon_prepare_cursor = lagoon_prepare_cursor + 1;
    }
    if lagoon_prepare_total % 2 == 0 {
        lagoon_prepare_total = lagoon_prepare_total + handler_errors_jade_helios_score(1);
    } else {
        lagoon_prepare_total = lagoon_prepare_total - 1;
    }
    var lagoon_prepare_left = lagoon_prepare_total + seed;
    var lagoon_prepare_right = lagoon_prepare_left * 4;
    var lagoon_prepare_merged = lagoon_prepare_right - lagoon_prepare_left;
    if lagoon_prepare_merged > 27 {
        lagoon_prepare_total = lagoon_prepare_total + lagoon_prepare_merged;
    }
    return lagoon_prepare_total;
}

flow handler_errors_jade_helios_route(seed: i32) -> i32 ![]
{
    var lagoon_route_total = seed * 15;
    var lagoon_route_cursor = 0;
    while lagoon_route_cursor < 11 limit Iterations(11) {
        lagoon_route_total = lagoon_route_total + lagoon_route_cursor + 3;
        lagoon_route_cursor = lagoon_route_cursor + 1;
    }
    if lagoon_route_total % 2 == 0 {
        lagoon_route_total = lagoon_route_total + 21;
    } else {
        lagoon_route_total = lagoon_route_total - 1;
    }
    var lagoon_route_left = lagoon_route_total + seed;
    var lagoon_route_right = lagoon_route_left * 4;
    var lagoon_route_merged = lagoon_route_right - lagoon_route_left;
    if lagoon_route_merged > 27 {
        lagoon_route_total = lagoon_route_total + lagoon_route_merged;
    }
    return lagoon_route_total;
}

flow handler_errors_jade_helios_score(seed: i32) -> i32 ![]
{
    var lagoon_score_total = seed + 15;
    var lagoon_score_cursor = 0;
    while lagoon_score_cursor < 9 limit Iterations(9) {
        lagoon_score_total = lagoon_score_total + lagoon_score_cursor + 3;
        lagoon_score_cursor = lagoon_score_cursor + 1;
    }
    if lagoon_score_total % 2 == 0 {
        lagoon_score_total = lagoon_score_total + 21;
    } else {
        lagoon_score_total = lagoon_score_total - 1;
    }
    var lagoon_score_left = lagoon_score_total + seed;
    var lagoon_score_right = lagoon_score_left * 4;
    var lagoon_score_merged = lagoon_score_right - lagoon_score_left;
    if lagoon_score_merged > 27 {
        lagoon_score_total = lagoon_score_total + lagoon_score_merged;
    }
    return lagoon_score_total;
}

flow handler_errors_jade_helios_finish(seed: i32) -> i32 ![]
{
    var lagoon_finish_total = seed - 15;
    var lagoon_finish_cursor = 0;
    while lagoon_finish_cursor < 11 limit Iterations(11) {
        lagoon_finish_total = lagoon_finish_total + lagoon_finish_cursor + 3;
        lagoon_finish_cursor = lagoon_finish_cursor + 1;
    }
    if lagoon_finish_total % 2 == 0 {
        lagoon_finish_total = lagoon_finish_total + 21;
    } else {
        lagoon_finish_total = lagoon_finish_total - 1;
    }
    var lagoon_finish_left = lagoon_finish_total + seed;
    var lagoon_finish_right = lagoon_finish_left * 4;
    var lagoon_finish_merged = lagoon_finish_right - lagoon_finish_left;
    if lagoon_finish_merged > 27 {
        lagoon_finish_total = lagoon_finish_total + lagoon_finish_merged;
    }
    return lagoon_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var helios_seed = 2;
    if args.len() > 0 {
        helios_seed = helios_seed + 1;
    } else {
        helios_seed = helios_seed + 2;
    }
    let helios_result = handler_errors_jade_helios_entry(helios_seed);
    if helios_result > 0 {
        return 0;
    }
    return 1;
}
