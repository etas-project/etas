module tests.compiler.effects.positive.handler_elimination_030;


flow handler_elimination_ember_ember_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var ember_total = handler_elimination_ember_ember_prepare(seed);
    ember_total = ember_total + handler_elimination_ember_ember_route(seed + 4);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            finish "fallback";
        }
    };
    let handler_marker = handled.len();
    let ember_adjust: i32 -> i32 = (value: i32) => value + 5;
    ember_total = ember_adjust(ember_total);
    ember_total = ember_total + handler_elimination_ember_ember_score(2);
    ember_total = ember_total + handler_elimination_ember_ember_finish(5);
    if ember_total > 70 {
        ember_total = ember_total - 10;
    } else {
        ember_total = ember_total + 17;
    }
    return ember_total;
}

flow handler_elimination_ember_ember_prepare(seed: i32) -> i32 ![]
{
    var oasis_prepare_total = seed + 14;
    var oasis_prepare_cursor = 0;
    while oasis_prepare_cursor < 8 limit Iterations(8) {
        oasis_prepare_total = oasis_prepare_total + oasis_prepare_cursor + 2;
        oasis_prepare_cursor = oasis_prepare_cursor + 1;
    }
    if oasis_prepare_total % 2 == 0 {
        oasis_prepare_total = oasis_prepare_total + handler_elimination_ember_ember_score(1);
    } else {
        oasis_prepare_total = oasis_prepare_total - 1;
    }
    var oasis_prepare_left = oasis_prepare_total + seed;
    var oasis_prepare_right = oasis_prepare_left * 4;
    var oasis_prepare_merged = oasis_prepare_right - oasis_prepare_left;
    if oasis_prepare_merged > 30 {
        oasis_prepare_total = oasis_prepare_total + oasis_prepare_merged;
    }
    return oasis_prepare_total;
}

flow handler_elimination_ember_ember_route(seed: i32) -> i32 ![]
{
    var oasis_route_total = seed * 14;
    var oasis_route_cursor = 0;
    while oasis_route_cursor < 7 limit Iterations(7) {
        oasis_route_total = oasis_route_total + oasis_route_cursor + 2;
        oasis_route_cursor = oasis_route_cursor + 1;
    }
    if oasis_route_total % 2 == 0 {
        oasis_route_total = oasis_route_total + 12;
    } else {
        oasis_route_total = oasis_route_total - 1;
    }
    var oasis_route_left = oasis_route_total + seed;
    var oasis_route_right = oasis_route_left * 4;
    var oasis_route_merged = oasis_route_right - oasis_route_left;
    if oasis_route_merged > 30 {
        oasis_route_total = oasis_route_total + oasis_route_merged;
    }
    return oasis_route_total;
}

flow handler_elimination_ember_ember_score(seed: i32) -> i32 ![]
{
    var oasis_score_total = seed + 14;
    var oasis_score_cursor = 0;
    while oasis_score_cursor < 8 limit Iterations(8) {
        oasis_score_total = oasis_score_total + oasis_score_cursor + 2;
        oasis_score_cursor = oasis_score_cursor + 1;
    }
    if oasis_score_total % 2 == 0 {
        oasis_score_total = oasis_score_total + 12;
    } else {
        oasis_score_total = oasis_score_total - 1;
    }
    var oasis_score_left = oasis_score_total + seed;
    var oasis_score_right = oasis_score_left * 4;
    var oasis_score_merged = oasis_score_right - oasis_score_left;
    if oasis_score_merged > 30 {
        oasis_score_total = oasis_score_total + oasis_score_merged;
    }
    return oasis_score_total;
}

flow handler_elimination_ember_ember_finish(seed: i32) -> i32 ![]
{
    var oasis_finish_total = seed - 14;
    var oasis_finish_cursor = 0;
    while oasis_finish_cursor < 11 limit Iterations(11) {
        oasis_finish_total = oasis_finish_total + oasis_finish_cursor + 2;
        oasis_finish_cursor = oasis_finish_cursor + 1;
    }
    if oasis_finish_total % 2 == 0 {
        oasis_finish_total = oasis_finish_total + 12;
    } else {
        oasis_finish_total = oasis_finish_total - 1;
    }
    var oasis_finish_left = oasis_finish_total + seed;
    var oasis_finish_right = oasis_finish_left * 4;
    var oasis_finish_merged = oasis_finish_right - oasis_finish_left;
    if oasis_finish_merged > 30 {
        oasis_finish_total = oasis_finish_total + oasis_finish_merged;
    }
    return oasis_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var ember_seed = 9;
    if args.len() > 0 {
        ember_seed = ember_seed + 1;
    } else {
        ember_seed = ember_seed + 2;
    }
    let ember_result = handler_elimination_ember_ember_entry(ember_seed);
    if ember_result > 0 {
        return 0;
    }
    return 1;
}
