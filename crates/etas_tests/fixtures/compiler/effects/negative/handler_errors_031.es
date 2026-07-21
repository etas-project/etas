module tests.compiler.effects.negative.handler_errors_031;


flow handler_errors_kernel_islet_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var islet_total = handler_errors_kernel_islet_prepare(seed);
    islet_total = islet_total + handler_errors_kernel_islet_route(seed + 9);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            resume "fallback";
        }
    };
    let islet_adjust: i32 -> i32 = (value: i32) => value + 3;
    islet_total = islet_adjust(islet_total);
    islet_total = islet_total + handler_errors_kernel_islet_score(3);
    islet_total = islet_total + handler_errors_kernel_islet_finish(7);
    if islet_total > 471 {
        islet_total = islet_total - 4;
    } else {
        islet_total = islet_total + 10;
    }
    return islet_total;
}

flow handler_errors_kernel_islet_prepare(seed: i32) -> i32 ![]
{
    var summit_prepare_total = seed + 16;
    var summit_prepare_cursor = 0;
    while summit_prepare_cursor < 9 limit Iterations(9) {
        summit_prepare_total = summit_prepare_total + summit_prepare_cursor + 4;
        summit_prepare_cursor = summit_prepare_cursor + 1;
    }
    if summit_prepare_total % 2 == 0 {
        summit_prepare_total = summit_prepare_total + handler_errors_kernel_islet_score(1);
    } else {
        summit_prepare_total = summit_prepare_total - 2;
    }
    var summit_prepare_left = summit_prepare_total + seed;
    var summit_prepare_right = summit_prepare_left * 5;
    var summit_prepare_merged = summit_prepare_right - summit_prepare_left;
    if summit_prepare_merged > 28 {
        summit_prepare_total = summit_prepare_total + summit_prepare_merged;
    }
    return summit_prepare_total;
}

flow handler_errors_kernel_islet_route(seed: i32) -> i32 ![]
{
    var summit_route_total = seed * 16;
    var summit_route_cursor = 0;
    while summit_route_cursor < 12 limit Iterations(12) {
        summit_route_total = summit_route_total + summit_route_cursor + 4;
        summit_route_cursor = summit_route_cursor + 1;
    }
    if summit_route_total % 2 == 0 {
        summit_route_total = summit_route_total + 22;
    } else {
        summit_route_total = summit_route_total - 2;
    }
    var summit_route_left = summit_route_total + seed;
    var summit_route_right = summit_route_left * 5;
    var summit_route_merged = summit_route_right - summit_route_left;
    if summit_route_merged > 28 {
        summit_route_total = summit_route_total + summit_route_merged;
    }
    return summit_route_total;
}

flow handler_errors_kernel_islet_score(seed: i32) -> i32 ![]
{
    var summit_score_total = seed + 16;
    var summit_score_cursor = 0;
    while summit_score_cursor < 10 limit Iterations(10) {
        summit_score_total = summit_score_total + summit_score_cursor + 4;
        summit_score_cursor = summit_score_cursor + 1;
    }
    if summit_score_total % 2 == 0 {
        summit_score_total = summit_score_total + 22;
    } else {
        summit_score_total = summit_score_total - 2;
    }
    var summit_score_left = summit_score_total + seed;
    var summit_score_right = summit_score_left * 5;
    var summit_score_merged = summit_score_right - summit_score_left;
    if summit_score_merged > 28 {
        summit_score_total = summit_score_total + summit_score_merged;
    }
    return summit_score_total;
}

flow handler_errors_kernel_islet_finish(seed: i32) -> i32 ![]
{
    var summit_finish_total = seed - 16;
    var summit_finish_cursor = 0;
    while summit_finish_cursor < 12 limit Iterations(12) {
        summit_finish_total = summit_finish_total + summit_finish_cursor + 4;
        summit_finish_cursor = summit_finish_cursor + 1;
    }
    if summit_finish_total % 2 == 0 {
        summit_finish_total = summit_finish_total + 22;
    } else {
        summit_finish_total = summit_finish_total - 2;
    }
    var summit_finish_left = summit_finish_total + seed;
    var summit_finish_right = summit_finish_left * 5;
    var summit_finish_merged = summit_finish_right - summit_finish_left;
    if summit_finish_merged > 28 {
        summit_finish_total = summit_finish_total + summit_finish_merged;
    }
    return summit_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var islet_seed = 3;
    if args.len() > 0 {
        islet_seed = islet_seed + 1;
    } else {
        islet_seed = islet_seed + 2;
    }
    let islet_result = handler_errors_kernel_islet_entry(islet_seed);
    if islet_result > 0 {
        return 0;
    }
    return 1;
}
