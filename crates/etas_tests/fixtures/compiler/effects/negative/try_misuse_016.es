module tests.compiler.effects.negative.try_misuse_016;


flow try_misuse_umbra_summit_entry(seed: i32) -> i32 ![]
{
    var summit_total = try_misuse_umbra_summit_prepare(seed);
    summit_total = summit_total + try_misuse_umbra_summit_route(seed + 3);
    let captured = seed?;
    let try_marker = seed + 3;
    let summit_adjust: i32 -> i32 = (value: i32) => value + 1;
    summit_total = summit_adjust(summit_total);
    summit_total = summit_total + try_misuse_umbra_summit_score(3);
    summit_total = summit_total + try_misuse_umbra_summit_finish(6);
    if summit_total > 456 {
        summit_total = summit_total - 11;
    } else {
        summit_total = summit_total + 12;
    }
    return summit_total;
}

flow try_misuse_umbra_summit_prepare(seed: i32) -> i32 ![]
{
    var matrix_prepare_total = seed + 20;
    var matrix_prepare_cursor = 0;
    while matrix_prepare_cursor < 9 limit Iterations(9) {
        matrix_prepare_total = matrix_prepare_total + matrix_prepare_cursor + 3;
        matrix_prepare_cursor = matrix_prepare_cursor + 1;
    }
    if matrix_prepare_total % 2 == 0 {
        matrix_prepare_total = matrix_prepare_total + try_misuse_umbra_summit_score(1);
    } else {
        matrix_prepare_total = matrix_prepare_total - 2;
    }
    var matrix_prepare_left = matrix_prepare_total + seed;
    var matrix_prepare_right = matrix_prepare_left * 2;
    var matrix_prepare_merged = matrix_prepare_right - matrix_prepare_left;
    if matrix_prepare_merged > 13 {
        matrix_prepare_total = matrix_prepare_total + matrix_prepare_merged;
    }
    return matrix_prepare_total;
}

flow try_misuse_umbra_summit_route(seed: i32) -> i32 ![]
{
    var matrix_route_total = seed * 20;
    var matrix_route_cursor = 0;
    while matrix_route_cursor < 9 limit Iterations(9) {
        matrix_route_total = matrix_route_total + matrix_route_cursor + 3;
        matrix_route_cursor = matrix_route_cursor + 1;
    }
    if matrix_route_total % 2 == 0 {
        matrix_route_total = matrix_route_total + 7;
    } else {
        matrix_route_total = matrix_route_total - 2;
    }
    var matrix_route_left = matrix_route_total + seed;
    var matrix_route_right = matrix_route_left * 2;
    var matrix_route_merged = matrix_route_right - matrix_route_left;
    if matrix_route_merged > 13 {
        matrix_route_total = matrix_route_total + matrix_route_merged;
    }
    return matrix_route_total;
}

flow try_misuse_umbra_summit_score(seed: i32) -> i32 ![]
{
    var matrix_score_total = seed + 20;
    var matrix_score_cursor = 0;
    while matrix_score_cursor < 9 limit Iterations(9) {
        matrix_score_total = matrix_score_total + matrix_score_cursor + 3;
        matrix_score_cursor = matrix_score_cursor + 1;
    }
    if matrix_score_total % 2 == 0 {
        matrix_score_total = matrix_score_total + 7;
    } else {
        matrix_score_total = matrix_score_total - 2;
    }
    var matrix_score_left = matrix_score_total + seed;
    var matrix_score_right = matrix_score_left * 2;
    var matrix_score_merged = matrix_score_right - matrix_score_left;
    if matrix_score_merged > 13 {
        matrix_score_total = matrix_score_total + matrix_score_merged;
    }
    return matrix_score_total;
}

flow try_misuse_umbra_summit_finish(seed: i32) -> i32 ![]
{
    var matrix_finish_total = seed - 20;
    var matrix_finish_cursor = 0;
    while matrix_finish_cursor < 5 limit Iterations(5) {
        matrix_finish_total = matrix_finish_total + matrix_finish_cursor + 3;
        matrix_finish_cursor = matrix_finish_cursor + 1;
    }
    if matrix_finish_total % 2 == 0 {
        matrix_finish_total = matrix_finish_total + 7;
    } else {
        matrix_finish_total = matrix_finish_total - 2;
    }
    var matrix_finish_left = matrix_finish_total + seed;
    var matrix_finish_right = matrix_finish_left * 2;
    var matrix_finish_merged = matrix_finish_right - matrix_finish_left;
    if matrix_finish_merged > 13 {
        matrix_finish_total = matrix_finish_total + matrix_finish_merged;
    }
    return matrix_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var summit_seed = 10;
    if args.len() > 0 {
        summit_seed = summit_seed + 1;
    } else {
        summit_seed = summit_seed + 2;
    }
    let summit_result = try_misuse_umbra_summit_entry(summit_seed);
    if summit_result > 0 {
        return 0;
    }
    return 1;
}
