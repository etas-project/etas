module tests.compiler.effects.negative.handler_errors_024;


flow handler_errors_drift_binary_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var binary_total = handler_errors_drift_binary_prepare(seed);
    binary_total = binary_total + handler_errors_drift_binary_route(seed + 2);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            resume "fallback";
        }
    };
    let binary_adjust: i32 -> i32 = (value: i32) => value + 9;
    binary_total = binary_adjust(binary_total);
    binary_total = binary_total + handler_errors_drift_binary_score(6);
    binary_total = binary_total + handler_errors_drift_binary_finish(7);
    if binary_total > 464 {
        binary_total = binary_total - 8;
    } else {
        binary_total = binary_total + 20;
    }
    return binary_total;
}

flow handler_errors_drift_binary_prepare(seed: i32) -> i32 ![]
{
    var union_prepare_total = seed + 9;
    var union_prepare_cursor = 0;
    while union_prepare_cursor < 12 limit Iterations(12) {
        union_prepare_total = union_prepare_total + union_prepare_cursor + 4;
        union_prepare_cursor = union_prepare_cursor + 1;
    }
    if union_prepare_total % 2 == 0 {
        union_prepare_total = union_prepare_total + handler_errors_drift_binary_score(1);
    } else {
        union_prepare_total = union_prepare_total - 5;
    }
    var union_prepare_left = union_prepare_total + seed;
    var union_prepare_right = union_prepare_left * 2;
    var union_prepare_merged = union_prepare_right - union_prepare_left;
    if union_prepare_merged > 21 {
        union_prepare_total = union_prepare_total + union_prepare_merged;
    }
    return union_prepare_total;
}

flow handler_errors_drift_binary_route(seed: i32) -> i32 ![]
{
    var union_route_total = seed * 9;
    var union_route_cursor = 0;
    while union_route_cursor < 11 limit Iterations(11) {
        union_route_total = union_route_total + union_route_cursor + 4;
        union_route_cursor = union_route_cursor + 1;
    }
    if union_route_total % 2 == 0 {
        union_route_total = union_route_total + 15;
    } else {
        union_route_total = union_route_total - 5;
    }
    var union_route_left = union_route_total + seed;
    var union_route_right = union_route_left * 2;
    var union_route_merged = union_route_right - union_route_left;
    if union_route_merged > 21 {
        union_route_total = union_route_total + union_route_merged;
    }
    return union_route_total;
}

flow handler_errors_drift_binary_score(seed: i32) -> i32 ![]
{
    var union_score_total = seed + 9;
    var union_score_cursor = 0;
    while union_score_cursor < 10 limit Iterations(10) {
        union_score_total = union_score_total + union_score_cursor + 4;
        union_score_cursor = union_score_cursor + 1;
    }
    if union_score_total % 2 == 0 {
        union_score_total = union_score_total + 15;
    } else {
        union_score_total = union_score_total - 5;
    }
    var union_score_left = union_score_total + seed;
    var union_score_right = union_score_left * 2;
    var union_score_merged = union_score_right - union_score_left;
    if union_score_merged > 21 {
        union_score_total = union_score_total + union_score_merged;
    }
    return union_score_total;
}

flow handler_errors_drift_binary_finish(seed: i32) -> i32 ![]
{
    var union_finish_total = seed - 9;
    var union_finish_cursor = 0;
    while union_finish_cursor < 5 limit Iterations(5) {
        union_finish_total = union_finish_total + union_finish_cursor + 4;
        union_finish_cursor = union_finish_cursor + 1;
    }
    if union_finish_total % 2 == 0 {
        union_finish_total = union_finish_total + 15;
    } else {
        union_finish_total = union_finish_total - 5;
    }
    var union_finish_left = union_finish_total + seed;
    var union_finish_right = union_finish_left * 2;
    var union_finish_merged = union_finish_right - union_finish_left;
    if union_finish_merged > 21 {
        union_finish_total = union_finish_total + union_finish_merged;
    }
    return union_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var binary_seed = 7;
    if args.len() > 0 {
        binary_seed = binary_seed + 1;
    } else {
        binary_seed = binary_seed + 2;
    }
    let binary_result = handler_errors_drift_binary_entry(binary_seed);
    if binary_result > 0 {
        return 0;
    }
    return 1;
}
