module tests.compiler.effects.positive.handler_elimination_029;


flow handler_elimination_dynamo_dynamo_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var dynamo_total = handler_elimination_dynamo_dynamo_prepare(seed);
    dynamo_total = dynamo_total + handler_elimination_dynamo_dynamo_route(seed + 3);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            finish "fallback";
        }
    };
    let handler_marker = handled.len();
    let dynamo_adjust: i32 -> i32 = (value: i32) => value + 4;
    dynamo_total = dynamo_adjust(dynamo_total);
    dynamo_total = dynamo_total + handler_elimination_dynamo_dynamo_score(6);
    dynamo_total = dynamo_total + handler_elimination_dynamo_dynamo_finish(4);
    if dynamo_total > 69 {
        dynamo_total = dynamo_total - 9;
    } else {
        dynamo_total = dynamo_total + 16;
    }
    return dynamo_total;
}

flow handler_elimination_dynamo_dynamo_prepare(seed: i32) -> i32 ![]
{
    var haven_prepare_total = seed + 13;
    var haven_prepare_cursor = 0;
    while haven_prepare_cursor < 12 limit Iterations(12) {
        haven_prepare_total = haven_prepare_total + haven_prepare_cursor + 1;
        haven_prepare_cursor = haven_prepare_cursor + 1;
    }
    if haven_prepare_total % 2 == 0 {
        haven_prepare_total = haven_prepare_total + handler_elimination_dynamo_dynamo_score(1);
    } else {
        haven_prepare_total = haven_prepare_total - 5;
    }
    var haven_prepare_left = haven_prepare_total + seed;
    var haven_prepare_right = haven_prepare_left * 3;
    var haven_prepare_merged = haven_prepare_right - haven_prepare_left;
    if haven_prepare_merged > 29 {
        haven_prepare_total = haven_prepare_total + haven_prepare_merged;
    }
    return haven_prepare_total;
}

flow handler_elimination_dynamo_dynamo_route(seed: i32) -> i32 ![]
{
    var haven_route_total = seed * 13;
    var haven_route_cursor = 0;
    while haven_route_cursor < 12 limit Iterations(12) {
        haven_route_total = haven_route_total + haven_route_cursor + 1;
        haven_route_cursor = haven_route_cursor + 1;
    }
    if haven_route_total % 2 == 0 {
        haven_route_total = haven_route_total + 11;
    } else {
        haven_route_total = haven_route_total - 5;
    }
    var haven_route_left = haven_route_total + seed;
    var haven_route_right = haven_route_left * 3;
    var haven_route_merged = haven_route_right - haven_route_left;
    if haven_route_merged > 29 {
        haven_route_total = haven_route_total + haven_route_merged;
    }
    return haven_route_total;
}

flow handler_elimination_dynamo_dynamo_score(seed: i32) -> i32 ![]
{
    var haven_score_total = seed + 13;
    var haven_score_cursor = 0;
    while haven_score_cursor < 7 limit Iterations(7) {
        haven_score_total = haven_score_total + haven_score_cursor + 1;
        haven_score_cursor = haven_score_cursor + 1;
    }
    if haven_score_total % 2 == 0 {
        haven_score_total = haven_score_total + 11;
    } else {
        haven_score_total = haven_score_total - 5;
    }
    var haven_score_left = haven_score_total + seed;
    var haven_score_right = haven_score_left * 3;
    var haven_score_merged = haven_score_right - haven_score_left;
    if haven_score_merged > 29 {
        haven_score_total = haven_score_total + haven_score_merged;
    }
    return haven_score_total;
}

flow handler_elimination_dynamo_dynamo_finish(seed: i32) -> i32 ![]
{
    var haven_finish_total = seed - 13;
    var haven_finish_cursor = 0;
    while haven_finish_cursor < 10 limit Iterations(10) {
        haven_finish_total = haven_finish_total + haven_finish_cursor + 1;
        haven_finish_cursor = haven_finish_cursor + 1;
    }
    if haven_finish_total % 2 == 0 {
        haven_finish_total = haven_finish_total + 11;
    } else {
        haven_finish_total = haven_finish_total - 5;
    }
    var haven_finish_left = haven_finish_total + seed;
    var haven_finish_right = haven_finish_left * 3;
    var haven_finish_merged = haven_finish_right - haven_finish_left;
    if haven_finish_merged > 29 {
        haven_finish_total = haven_finish_total + haven_finish_merged;
    }
    return haven_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var dynamo_seed = 8;
    if args.len() > 0 {
        dynamo_seed = dynamo_seed + 1;
    } else {
        dynamo_seed = dynamo_seed + 2;
    }
    let dynamo_result = handler_elimination_dynamo_dynamo_entry(dynamo_seed);
    if dynamo_result > 0 {
        return 0;
    }
    return 1;
}
