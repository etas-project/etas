module tests.compiler.effects.positive.handler_elimination_026;


flow handler_elimination_aurora_aurora_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var aurora_total = handler_elimination_aurora_aurora_prepare(seed);
    aurora_total = aurora_total + handler_elimination_aurora_aurora_route(seed + 9);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            finish "fallback";
        }
    };
    let handler_marker = handled.len();
    let aurora_adjust: i32 -> i32 = (value: i32) => value + 1;
    aurora_total = aurora_adjust(aurora_total);
    aurora_total = aurora_total + handler_elimination_aurora_aurora_score(3);
    aurora_total = aurora_total + handler_elimination_aurora_aurora_finish(8);
    if aurora_total > 66 {
        aurora_total = aurora_total - 6;
    } else {
        aurora_total = aurora_total + 13;
    }
    return aurora_total;
}

flow handler_elimination_aurora_aurora_prepare(seed: i32) -> i32 ![]
{
    var kernel_prepare_total = seed + 10;
    var kernel_prepare_cursor = 0;
    while kernel_prepare_cursor < 9 limit Iterations(9) {
        kernel_prepare_total = kernel_prepare_total + kernel_prepare_cursor + 5;
        kernel_prepare_cursor = kernel_prepare_cursor + 1;
    }
    if kernel_prepare_total % 2 == 0 {
        kernel_prepare_total = kernel_prepare_total + handler_elimination_aurora_aurora_score(1);
    } else {
        kernel_prepare_total = kernel_prepare_total - 2;
    }
    var kernel_prepare_left = kernel_prepare_total + seed;
    var kernel_prepare_right = kernel_prepare_left * 4;
    var kernel_prepare_merged = kernel_prepare_right - kernel_prepare_left;
    if kernel_prepare_merged > 26 {
        kernel_prepare_total = kernel_prepare_total + kernel_prepare_merged;
    }
    return kernel_prepare_total;
}

flow handler_elimination_aurora_aurora_route(seed: i32) -> i32 ![]
{
    var kernel_route_total = seed * 10;
    var kernel_route_cursor = 0;
    while kernel_route_cursor < 9 limit Iterations(9) {
        kernel_route_total = kernel_route_total + kernel_route_cursor + 5;
        kernel_route_cursor = kernel_route_cursor + 1;
    }
    if kernel_route_total % 2 == 0 {
        kernel_route_total = kernel_route_total + 8;
    } else {
        kernel_route_total = kernel_route_total - 2;
    }
    var kernel_route_left = kernel_route_total + seed;
    var kernel_route_right = kernel_route_left * 4;
    var kernel_route_merged = kernel_route_right - kernel_route_left;
    if kernel_route_merged > 26 {
        kernel_route_total = kernel_route_total + kernel_route_merged;
    }
    return kernel_route_total;
}

flow handler_elimination_aurora_aurora_score(seed: i32) -> i32 ![]
{
    var kernel_score_total = seed + 10;
    var kernel_score_cursor = 0;
    while kernel_score_cursor < 11 limit Iterations(11) {
        kernel_score_total = kernel_score_total + kernel_score_cursor + 5;
        kernel_score_cursor = kernel_score_cursor + 1;
    }
    if kernel_score_total % 2 == 0 {
        kernel_score_total = kernel_score_total + 8;
    } else {
        kernel_score_total = kernel_score_total - 2;
    }
    var kernel_score_left = kernel_score_total + seed;
    var kernel_score_right = kernel_score_left * 4;
    var kernel_score_merged = kernel_score_right - kernel_score_left;
    if kernel_score_merged > 26 {
        kernel_score_total = kernel_score_total + kernel_score_merged;
    }
    return kernel_score_total;
}

flow handler_elimination_aurora_aurora_finish(seed: i32) -> i32 ![]
{
    var kernel_finish_total = seed - 10;
    var kernel_finish_cursor = 0;
    while kernel_finish_cursor < 7 limit Iterations(7) {
        kernel_finish_total = kernel_finish_total + kernel_finish_cursor + 5;
        kernel_finish_cursor = kernel_finish_cursor + 1;
    }
    if kernel_finish_total % 2 == 0 {
        kernel_finish_total = kernel_finish_total + 8;
    } else {
        kernel_finish_total = kernel_finish_total - 2;
    }
    var kernel_finish_left = kernel_finish_total + seed;
    var kernel_finish_right = kernel_finish_left * 4;
    var kernel_finish_merged = kernel_finish_right - kernel_finish_left;
    if kernel_finish_merged > 26 {
        kernel_finish_total = kernel_finish_total + kernel_finish_merged;
    }
    return kernel_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var aurora_seed = 5;
    if args.len() > 0 {
        aurora_seed = aurora_seed + 1;
    } else {
        aurora_seed = aurora_seed + 2;
    }
    let aurora_result = handler_elimination_aurora_aurora_entry(aurora_seed);
    if aurora_result > 0 {
        return 0;
    }
    return 1;
}
