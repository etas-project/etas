module tests.compiler.effects.positive.handler_elimination_021;


flow handler_elimination_violet_violet_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var violet_total = handler_elimination_violet_violet_prepare(seed);
    violet_total = violet_total + handler_elimination_violet_violet_route(seed + 4);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            finish "fallback";
        }
    };
    let handler_marker = handled.len();
    let violet_adjust: i32 -> i32 = (value: i32) => value + 9;
    violet_total = violet_adjust(violet_total);
    violet_total = violet_total + handler_elimination_violet_violet_score(3);
    violet_total = violet_total + handler_elimination_violet_violet_finish(3);
    if violet_total > 61 {
        violet_total = violet_total - 12;
    } else {
        violet_total = violet_total + 8;
    }
    return violet_total;
}

flow handler_elimination_violet_violet_prepare(seed: i32) -> i32 ![]
{
    var aurora_prepare_total = seed + 5;
    var aurora_prepare_cursor = 0;
    while aurora_prepare_cursor < 9 limit Iterations(9) {
        aurora_prepare_total = aurora_prepare_total + aurora_prepare_cursor + 0;
        aurora_prepare_cursor = aurora_prepare_cursor + 1;
    }
    if aurora_prepare_total % 2 == 0 {
        aurora_prepare_total = aurora_prepare_total + handler_elimination_violet_violet_score(1);
    } else {
        aurora_prepare_total = aurora_prepare_total - 2;
    }
    var aurora_prepare_left = aurora_prepare_total + seed;
    var aurora_prepare_right = aurora_prepare_left * 3;
    var aurora_prepare_merged = aurora_prepare_right - aurora_prepare_left;
    if aurora_prepare_merged > 21 {
        aurora_prepare_total = aurora_prepare_total + aurora_prepare_merged;
    }
    return aurora_prepare_total;
}

flow handler_elimination_violet_violet_route(seed: i32) -> i32 ![]
{
    var aurora_route_total = seed * 5;
    var aurora_route_cursor = 0;
    while aurora_route_cursor < 10 limit Iterations(10) {
        aurora_route_total = aurora_route_total + aurora_route_cursor + 0;
        aurora_route_cursor = aurora_route_cursor + 1;
    }
    if aurora_route_total % 2 == 0 {
        aurora_route_total = aurora_route_total + 26;
    } else {
        aurora_route_total = aurora_route_total - 2;
    }
    var aurora_route_left = aurora_route_total + seed;
    var aurora_route_right = aurora_route_left * 3;
    var aurora_route_merged = aurora_route_right - aurora_route_left;
    if aurora_route_merged > 21 {
        aurora_route_total = aurora_route_total + aurora_route_merged;
    }
    return aurora_route_total;
}

flow handler_elimination_violet_violet_score(seed: i32) -> i32 ![]
{
    var aurora_score_total = seed + 5;
    var aurora_score_cursor = 0;
    while aurora_score_cursor < 6 limit Iterations(6) {
        aurora_score_total = aurora_score_total + aurora_score_cursor + 0;
        aurora_score_cursor = aurora_score_cursor + 1;
    }
    if aurora_score_total % 2 == 0 {
        aurora_score_total = aurora_score_total + 26;
    } else {
        aurora_score_total = aurora_score_total - 2;
    }
    var aurora_score_left = aurora_score_total + seed;
    var aurora_score_right = aurora_score_left * 3;
    var aurora_score_merged = aurora_score_right - aurora_score_left;
    if aurora_score_merged > 21 {
        aurora_score_total = aurora_score_total + aurora_score_merged;
    }
    return aurora_score_total;
}

flow handler_elimination_violet_violet_finish(seed: i32) -> i32 ![]
{
    var aurora_finish_total = seed - 5;
    var aurora_finish_cursor = 0;
    while aurora_finish_cursor < 10 limit Iterations(10) {
        aurora_finish_total = aurora_finish_total + aurora_finish_cursor + 0;
        aurora_finish_cursor = aurora_finish_cursor + 1;
    }
    if aurora_finish_total % 2 == 0 {
        aurora_finish_total = aurora_finish_total + 26;
    } else {
        aurora_finish_total = aurora_finish_total - 2;
    }
    var aurora_finish_left = aurora_finish_total + seed;
    var aurora_finish_right = aurora_finish_left * 3;
    var aurora_finish_merged = aurora_finish_right - aurora_finish_left;
    if aurora_finish_merged > 21 {
        aurora_finish_total = aurora_finish_total + aurora_finish_merged;
    }
    return aurora_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var violet_seed = 11;
    if args.len() > 0 {
        violet_seed = violet_seed + 1;
    } else {
        violet_seed = violet_seed + 2;
    }
    let violet_result = handler_elimination_violet_violet_entry(violet_seed);
    if violet_result > 0 {
        return 0;
    }
    return 1;
}
