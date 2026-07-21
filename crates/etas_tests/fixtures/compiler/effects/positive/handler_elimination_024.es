module tests.compiler.effects.positive.handler_elimination_024;


flow handler_elimination_yarrow_yarrow_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var yarrow_total = handler_elimination_yarrow_yarrow_prepare(seed);
    yarrow_total = yarrow_total + handler_elimination_yarrow_yarrow_route(seed + 7);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            finish "fallback";
        }
    };
    let handler_marker = handled.len();
    let yarrow_adjust: i32 -> i32 = (value: i32) => value + 12;
    yarrow_total = yarrow_adjust(yarrow_total);
    yarrow_total = yarrow_total + handler_elimination_yarrow_yarrow_score(6);
    yarrow_total = yarrow_total + handler_elimination_yarrow_yarrow_finish(6);
    if yarrow_total > 64 {
        yarrow_total = yarrow_total - 4;
    } else {
        yarrow_total = yarrow_total + 11;
    }
    return yarrow_total;
}

flow handler_elimination_yarrow_yarrow_prepare(seed: i32) -> i32 ![]
{
    var velvet_prepare_total = seed + 8;
    var velvet_prepare_cursor = 0;
    while velvet_prepare_cursor < 12 limit Iterations(12) {
        velvet_prepare_total = velvet_prepare_total + velvet_prepare_cursor + 3;
        velvet_prepare_cursor = velvet_prepare_cursor + 1;
    }
    if velvet_prepare_total % 2 == 0 {
        velvet_prepare_total = velvet_prepare_total + handler_elimination_yarrow_yarrow_score(1);
    } else {
        velvet_prepare_total = velvet_prepare_total - 5;
    }
    var velvet_prepare_left = velvet_prepare_total + seed;
    var velvet_prepare_right = velvet_prepare_left * 2;
    var velvet_prepare_merged = velvet_prepare_right - velvet_prepare_left;
    if velvet_prepare_merged > 24 {
        velvet_prepare_total = velvet_prepare_total + velvet_prepare_merged;
    }
    return velvet_prepare_total;
}

flow handler_elimination_yarrow_yarrow_route(seed: i32) -> i32 ![]
{
    var velvet_route_total = seed * 8;
    var velvet_route_cursor = 0;
    while velvet_route_cursor < 7 limit Iterations(7) {
        velvet_route_total = velvet_route_total + velvet_route_cursor + 3;
        velvet_route_cursor = velvet_route_cursor + 1;
    }
    if velvet_route_total % 2 == 0 {
        velvet_route_total = velvet_route_total + 6;
    } else {
        velvet_route_total = velvet_route_total - 5;
    }
    var velvet_route_left = velvet_route_total + seed;
    var velvet_route_right = velvet_route_left * 2;
    var velvet_route_merged = velvet_route_right - velvet_route_left;
    if velvet_route_merged > 24 {
        velvet_route_total = velvet_route_total + velvet_route_merged;
    }
    return velvet_route_total;
}

flow handler_elimination_yarrow_yarrow_score(seed: i32) -> i32 ![]
{
    var velvet_score_total = seed + 8;
    var velvet_score_cursor = 0;
    while velvet_score_cursor < 9 limit Iterations(9) {
        velvet_score_total = velvet_score_total + velvet_score_cursor + 3;
        velvet_score_cursor = velvet_score_cursor + 1;
    }
    if velvet_score_total % 2 == 0 {
        velvet_score_total = velvet_score_total + 6;
    } else {
        velvet_score_total = velvet_score_total - 5;
    }
    var velvet_score_left = velvet_score_total + seed;
    var velvet_score_right = velvet_score_left * 2;
    var velvet_score_merged = velvet_score_right - velvet_score_left;
    if velvet_score_merged > 24 {
        velvet_score_total = velvet_score_total + velvet_score_merged;
    }
    return velvet_score_total;
}

flow handler_elimination_yarrow_yarrow_finish(seed: i32) -> i32 ![]
{
    var velvet_finish_total = seed - 8;
    var velvet_finish_cursor = 0;
    while velvet_finish_cursor < 5 limit Iterations(5) {
        velvet_finish_total = velvet_finish_total + velvet_finish_cursor + 3;
        velvet_finish_cursor = velvet_finish_cursor + 1;
    }
    if velvet_finish_total % 2 == 0 {
        velvet_finish_total = velvet_finish_total + 6;
    } else {
        velvet_finish_total = velvet_finish_total - 5;
    }
    var velvet_finish_left = velvet_finish_total + seed;
    var velvet_finish_right = velvet_finish_left * 2;
    var velvet_finish_merged = velvet_finish_right - velvet_finish_left;
    if velvet_finish_merged > 24 {
        velvet_finish_total = velvet_finish_total + velvet_finish_merged;
    }
    return velvet_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var yarrow_seed = 3;
    if args.len() > 0 {
        yarrow_seed = yarrow_seed + 1;
    } else {
        yarrow_seed = yarrow_seed + 2;
    }
    let yarrow_result = handler_elimination_yarrow_yarrow_entry(yarrow_seed);
    if yarrow_result > 0 {
        return 0;
    }
    return 1;
}
