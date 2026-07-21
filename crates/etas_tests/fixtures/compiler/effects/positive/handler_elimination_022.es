module tests.compiler.effects.positive.handler_elimination_022;


flow handler_elimination_willow_willow_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var willow_total = handler_elimination_willow_willow_prepare(seed);
    willow_total = willow_total + handler_elimination_willow_willow_route(seed + 5);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            finish "fallback";
        }
    };
    let handler_marker = handled.len();
    let willow_adjust: i32 -> i32 = (value: i32) => value + 10;
    willow_total = willow_adjust(willow_total);
    willow_total = willow_total + handler_elimination_willow_willow_score(4);
    willow_total = willow_total + handler_elimination_willow_willow_finish(4);
    if willow_total > 62 {
        willow_total = willow_total - 2;
    } else {
        willow_total = willow_total + 9;
    }
    return willow_total;
}

flow handler_elimination_willow_willow_prepare(seed: i32) -> i32 ![]
{
    var harbor_prepare_total = seed + 6;
    var harbor_prepare_cursor = 0;
    while harbor_prepare_cursor < 10 limit Iterations(10) {
        harbor_prepare_total = harbor_prepare_total + harbor_prepare_cursor + 1;
        harbor_prepare_cursor = harbor_prepare_cursor + 1;
    }
    if harbor_prepare_total % 2 == 0 {
        harbor_prepare_total = harbor_prepare_total + handler_elimination_willow_willow_score(1);
    } else {
        harbor_prepare_total = harbor_prepare_total - 3;
    }
    var harbor_prepare_left = harbor_prepare_total + seed;
    var harbor_prepare_right = harbor_prepare_left * 4;
    var harbor_prepare_merged = harbor_prepare_right - harbor_prepare_left;
    if harbor_prepare_merged > 22 {
        harbor_prepare_total = harbor_prepare_total + harbor_prepare_merged;
    }
    return harbor_prepare_total;
}

flow handler_elimination_willow_willow_route(seed: i32) -> i32 ![]
{
    var harbor_route_total = seed * 6;
    var harbor_route_cursor = 0;
    while harbor_route_cursor < 11 limit Iterations(11) {
        harbor_route_total = harbor_route_total + harbor_route_cursor + 1;
        harbor_route_cursor = harbor_route_cursor + 1;
    }
    if harbor_route_total % 2 == 0 {
        harbor_route_total = harbor_route_total + 27;
    } else {
        harbor_route_total = harbor_route_total - 3;
    }
    var harbor_route_left = harbor_route_total + seed;
    var harbor_route_right = harbor_route_left * 4;
    var harbor_route_merged = harbor_route_right - harbor_route_left;
    if harbor_route_merged > 22 {
        harbor_route_total = harbor_route_total + harbor_route_merged;
    }
    return harbor_route_total;
}

flow handler_elimination_willow_willow_score(seed: i32) -> i32 ![]
{
    var harbor_score_total = seed + 6;
    var harbor_score_cursor = 0;
    while harbor_score_cursor < 7 limit Iterations(7) {
        harbor_score_total = harbor_score_total + harbor_score_cursor + 1;
        harbor_score_cursor = harbor_score_cursor + 1;
    }
    if harbor_score_total % 2 == 0 {
        harbor_score_total = harbor_score_total + 27;
    } else {
        harbor_score_total = harbor_score_total - 3;
    }
    var harbor_score_left = harbor_score_total + seed;
    var harbor_score_right = harbor_score_left * 4;
    var harbor_score_merged = harbor_score_right - harbor_score_left;
    if harbor_score_merged > 22 {
        harbor_score_total = harbor_score_total + harbor_score_merged;
    }
    return harbor_score_total;
}

flow handler_elimination_willow_willow_finish(seed: i32) -> i32 ![]
{
    var harbor_finish_total = seed - 6;
    var harbor_finish_cursor = 0;
    while harbor_finish_cursor < 11 limit Iterations(11) {
        harbor_finish_total = harbor_finish_total + harbor_finish_cursor + 1;
        harbor_finish_cursor = harbor_finish_cursor + 1;
    }
    if harbor_finish_total % 2 == 0 {
        harbor_finish_total = harbor_finish_total + 27;
    } else {
        harbor_finish_total = harbor_finish_total - 3;
    }
    var harbor_finish_left = harbor_finish_total + seed;
    var harbor_finish_right = harbor_finish_left * 4;
    var harbor_finish_merged = harbor_finish_right - harbor_finish_left;
    if harbor_finish_merged > 22 {
        harbor_finish_total = harbor_finish_total + harbor_finish_merged;
    }
    return harbor_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var willow_seed = 1;
    if args.len() > 0 {
        willow_seed = willow_seed + 1;
    } else {
        willow_seed = willow_seed + 2;
    }
    let willow_result = handler_elimination_willow_willow_entry(willow_seed);
    if willow_result > 0 {
        return 0;
    }
    return 1;
}
