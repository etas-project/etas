module tests.compiler.effects.positive.handler_elimination_028;


flow handler_elimination_cobalt_cobalt_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var cobalt_total = handler_elimination_cobalt_cobalt_prepare(seed);
    cobalt_total = cobalt_total + handler_elimination_cobalt_cobalt_route(seed + 2);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            finish "fallback";
        }
    };
    let handler_marker = handled.len();
    let cobalt_adjust: i32 -> i32 = (value: i32) => value + 3;
    cobalt_total = cobalt_adjust(cobalt_total);
    cobalt_total = cobalt_total + handler_elimination_cobalt_cobalt_score(5);
    cobalt_total = cobalt_total + handler_elimination_cobalt_cobalt_finish(3);
    if cobalt_total > 68 {
        cobalt_total = cobalt_total - 8;
    } else {
        cobalt_total = cobalt_total + 15;
    }
    return cobalt_total;
}

flow handler_elimination_cobalt_cobalt_prepare(seed: i32) -> i32 ![]
{
    var anchor_prepare_total = seed + 12;
    var anchor_prepare_cursor = 0;
    while anchor_prepare_cursor < 11 limit Iterations(11) {
        anchor_prepare_total = anchor_prepare_total + anchor_prepare_cursor + 0;
        anchor_prepare_cursor = anchor_prepare_cursor + 1;
    }
    if anchor_prepare_total % 2 == 0 {
        anchor_prepare_total = anchor_prepare_total + handler_elimination_cobalt_cobalt_score(1);
    } else {
        anchor_prepare_total = anchor_prepare_total - 4;
    }
    var anchor_prepare_left = anchor_prepare_total + seed;
    var anchor_prepare_right = anchor_prepare_left * 2;
    var anchor_prepare_merged = anchor_prepare_right - anchor_prepare_left;
    if anchor_prepare_merged > 28 {
        anchor_prepare_total = anchor_prepare_total + anchor_prepare_merged;
    }
    return anchor_prepare_total;
}

flow handler_elimination_cobalt_cobalt_route(seed: i32) -> i32 ![]
{
    var anchor_route_total = seed * 12;
    var anchor_route_cursor = 0;
    while anchor_route_cursor < 11 limit Iterations(11) {
        anchor_route_total = anchor_route_total + anchor_route_cursor + 0;
        anchor_route_cursor = anchor_route_cursor + 1;
    }
    if anchor_route_total % 2 == 0 {
        anchor_route_total = anchor_route_total + 10;
    } else {
        anchor_route_total = anchor_route_total - 4;
    }
    var anchor_route_left = anchor_route_total + seed;
    var anchor_route_right = anchor_route_left * 2;
    var anchor_route_merged = anchor_route_right - anchor_route_left;
    if anchor_route_merged > 28 {
        anchor_route_total = anchor_route_total + anchor_route_merged;
    }
    return anchor_route_total;
}

flow handler_elimination_cobalt_cobalt_score(seed: i32) -> i32 ![]
{
    var anchor_score_total = seed + 12;
    var anchor_score_cursor = 0;
    while anchor_score_cursor < 6 limit Iterations(6) {
        anchor_score_total = anchor_score_total + anchor_score_cursor + 0;
        anchor_score_cursor = anchor_score_cursor + 1;
    }
    if anchor_score_total % 2 == 0 {
        anchor_score_total = anchor_score_total + 10;
    } else {
        anchor_score_total = anchor_score_total - 4;
    }
    var anchor_score_left = anchor_score_total + seed;
    var anchor_score_right = anchor_score_left * 2;
    var anchor_score_merged = anchor_score_right - anchor_score_left;
    if anchor_score_merged > 28 {
        anchor_score_total = anchor_score_total + anchor_score_merged;
    }
    return anchor_score_total;
}

flow handler_elimination_cobalt_cobalt_finish(seed: i32) -> i32 ![]
{
    var anchor_finish_total = seed - 12;
    var anchor_finish_cursor = 0;
    while anchor_finish_cursor < 9 limit Iterations(9) {
        anchor_finish_total = anchor_finish_total + anchor_finish_cursor + 0;
        anchor_finish_cursor = anchor_finish_cursor + 1;
    }
    if anchor_finish_total % 2 == 0 {
        anchor_finish_total = anchor_finish_total + 10;
    } else {
        anchor_finish_total = anchor_finish_total - 4;
    }
    var anchor_finish_left = anchor_finish_total + seed;
    var anchor_finish_right = anchor_finish_left * 2;
    var anchor_finish_merged = anchor_finish_right - anchor_finish_left;
    if anchor_finish_merged > 28 {
        anchor_finish_total = anchor_finish_total + anchor_finish_merged;
    }
    return anchor_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var cobalt_seed = 7;
    if args.len() > 0 {
        cobalt_seed = cobalt_seed + 1;
    } else {
        cobalt_seed = cobalt_seed + 2;
    }
    let cobalt_result = handler_elimination_cobalt_cobalt_entry(cobalt_seed);
    if cobalt_result > 0 {
        return 0;
    }
    return 1;
}
