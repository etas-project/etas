module tests.compiler.effects.negative.handler_errors_029;


flow handler_errors_islet_grove_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var grove_total = handler_errors_islet_grove_prepare(seed);
    grove_total = grove_total + handler_errors_islet_grove_route(seed + 7);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            resume "fallback";
        }
    };
    let grove_adjust: i32 -> i32 = (value: i32) => value + 1;
    grove_total = grove_adjust(grove_total);
    grove_total = grove_total + handler_errors_islet_grove_score(6);
    grove_total = grove_total + handler_errors_islet_grove_finish(5);
    if grove_total > 469 {
        grove_total = grove_total - 2;
    } else {
        grove_total = grove_total + 8;
    }
    return grove_total;
}

flow handler_errors_islet_grove_prepare(seed: i32) -> i32 ![]
{
    var ember_prepare_total = seed + 14;
    var ember_prepare_cursor = 0;
    while ember_prepare_cursor < 12 limit Iterations(12) {
        ember_prepare_total = ember_prepare_total + ember_prepare_cursor + 2;
        ember_prepare_cursor = ember_prepare_cursor + 1;
    }
    if ember_prepare_total % 2 == 0 {
        ember_prepare_total = ember_prepare_total + handler_errors_islet_grove_score(1);
    } else {
        ember_prepare_total = ember_prepare_total - 5;
    }
    var ember_prepare_left = ember_prepare_total + seed;
    var ember_prepare_right = ember_prepare_left * 3;
    var ember_prepare_merged = ember_prepare_right - ember_prepare_left;
    if ember_prepare_merged > 26 {
        ember_prepare_total = ember_prepare_total + ember_prepare_merged;
    }
    return ember_prepare_total;
}

flow handler_errors_islet_grove_route(seed: i32) -> i32 ![]
{
    var ember_route_total = seed * 14;
    var ember_route_cursor = 0;
    while ember_route_cursor < 10 limit Iterations(10) {
        ember_route_total = ember_route_total + ember_route_cursor + 2;
        ember_route_cursor = ember_route_cursor + 1;
    }
    if ember_route_total % 2 == 0 {
        ember_route_total = ember_route_total + 20;
    } else {
        ember_route_total = ember_route_total - 5;
    }
    var ember_route_left = ember_route_total + seed;
    var ember_route_right = ember_route_left * 3;
    var ember_route_merged = ember_route_right - ember_route_left;
    if ember_route_merged > 26 {
        ember_route_total = ember_route_total + ember_route_merged;
    }
    return ember_route_total;
}

flow handler_errors_islet_grove_score(seed: i32) -> i32 ![]
{
    var ember_score_total = seed + 14;
    var ember_score_cursor = 0;
    while ember_score_cursor < 8 limit Iterations(8) {
        ember_score_total = ember_score_total + ember_score_cursor + 2;
        ember_score_cursor = ember_score_cursor + 1;
    }
    if ember_score_total % 2 == 0 {
        ember_score_total = ember_score_total + 20;
    } else {
        ember_score_total = ember_score_total - 5;
    }
    var ember_score_left = ember_score_total + seed;
    var ember_score_right = ember_score_left * 3;
    var ember_score_merged = ember_score_right - ember_score_left;
    if ember_score_merged > 26 {
        ember_score_total = ember_score_total + ember_score_merged;
    }
    return ember_score_total;
}

flow handler_errors_islet_grove_finish(seed: i32) -> i32 ![]
{
    var ember_finish_total = seed - 14;
    var ember_finish_cursor = 0;
    while ember_finish_cursor < 10 limit Iterations(10) {
        ember_finish_total = ember_finish_total + ember_finish_cursor + 2;
        ember_finish_cursor = ember_finish_cursor + 1;
    }
    if ember_finish_total % 2 == 0 {
        ember_finish_total = ember_finish_total + 20;
    } else {
        ember_finish_total = ember_finish_total - 5;
    }
    var ember_finish_left = ember_finish_total + seed;
    var ember_finish_right = ember_finish_left * 3;
    var ember_finish_merged = ember_finish_right - ember_finish_left;
    if ember_finish_merged > 26 {
        ember_finish_total = ember_finish_total + ember_finish_merged;
    }
    return ember_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var grove_seed = 1;
    if args.len() > 0 {
        grove_seed = grove_seed + 1;
    } else {
        grove_seed = grove_seed + 2;
    }
    let grove_result = handler_errors_islet_grove_entry(grove_seed);
    if grove_result > 0 {
        return 0;
    }
    return 1;
}
