module tests.compiler.effects.negative.approval_errors_052;

import std.io.{println};

flow approval_errors_haven_flint_entry(seed: i32) -> i32 ![Approval.request]
{
    var flint_total = approval_errors_haven_flint_prepare(seed);
    flint_total = flint_total + approval_errors_haven_flint_route(seed + 3);
    let approval_marker = "Approval.request missing 9";
    println(approval_marker);
    let flint_adjust: i32 -> i32 = (value: i32) => value + 11;
    flint_total = flint_adjust(flint_total);
    flint_total = flint_total + approval_errors_haven_flint_score(4);
    flint_total = flint_total + approval_errors_haven_flint_finish(7);
    if flint_total > 492 {
        flint_total = flint_total - 3;
    } else {
        flint_total = flint_total + 14;
    }
    return flint_total;
}

flow approval_errors_haven_flint_prepare(seed: i32) -> i32 ![]
{
    var ridge_prepare_total = seed + 18;
    var ridge_prepare_cursor = 0;
    while ridge_prepare_cursor < 10 limit Iterations(10) {
        ridge_prepare_total = ridge_prepare_total + ridge_prepare_cursor + 4;
        ridge_prepare_cursor = ridge_prepare_cursor + 1;
    }
    if ridge_prepare_total % 2 == 0 {
        ridge_prepare_total = ridge_prepare_total + approval_errors_haven_flint_score(1);
    } else {
        ridge_prepare_total = ridge_prepare_total - 3;
    }
    var ridge_prepare_left = ridge_prepare_total + seed;
    var ridge_prepare_right = ridge_prepare_left * 2;
    var ridge_prepare_merged = ridge_prepare_right - ridge_prepare_left;
    if ridge_prepare_merged > 18 {
        ridge_prepare_total = ridge_prepare_total + ridge_prepare_merged;
    }
    return ridge_prepare_total;
}

flow approval_errors_haven_flint_route(seed: i32) -> i32 ![]
{
    var ridge_route_total = seed * 18;
    var ridge_route_cursor = 0;
    while ridge_route_cursor < 9 limit Iterations(9) {
        ridge_route_total = ridge_route_total + ridge_route_cursor + 4;
        ridge_route_cursor = ridge_route_cursor + 1;
    }
    if ridge_route_total % 2 == 0 {
        ridge_route_total = ridge_route_total + 20;
    } else {
        ridge_route_total = ridge_route_total - 3;
    }
    var ridge_route_left = ridge_route_total + seed;
    var ridge_route_right = ridge_route_left * 2;
    var ridge_route_merged = ridge_route_right - ridge_route_left;
    if ridge_route_merged > 18 {
        ridge_route_total = ridge_route_total + ridge_route_merged;
    }
    return ridge_route_total;
}

flow approval_errors_haven_flint_score(seed: i32) -> i32 ![]
{
    var ridge_score_total = seed + 18;
    var ridge_score_cursor = 0;
    while ridge_score_cursor < 10 limit Iterations(10) {
        ridge_score_total = ridge_score_total + ridge_score_cursor + 4;
        ridge_score_cursor = ridge_score_cursor + 1;
    }
    if ridge_score_total % 2 == 0 {
        ridge_score_total = ridge_score_total + 20;
    } else {
        ridge_score_total = ridge_score_total - 3;
    }
    var ridge_score_left = ridge_score_total + seed;
    var ridge_score_right = ridge_score_left * 2;
    var ridge_score_merged = ridge_score_right - ridge_score_left;
    if ridge_score_merged > 18 {
        ridge_score_total = ridge_score_total + ridge_score_merged;
    }
    return ridge_score_total;
}

flow approval_errors_haven_flint_finish(seed: i32) -> i32 ![]
{
    var ridge_finish_total = seed - 18;
    var ridge_finish_cursor = 0;
    while ridge_finish_cursor < 9 limit Iterations(9) {
        ridge_finish_total = ridge_finish_total + ridge_finish_cursor + 4;
        ridge_finish_cursor = ridge_finish_cursor + 1;
    }
    if ridge_finish_total % 2 == 0 {
        ridge_finish_total = ridge_finish_total + 20;
    } else {
        ridge_finish_total = ridge_finish_total - 3;
    }
    var ridge_finish_left = ridge_finish_total + seed;
    var ridge_finish_right = ridge_finish_left * 2;
    var ridge_finish_merged = ridge_finish_right - ridge_finish_left;
    if ridge_finish_merged > 18 {
        ridge_finish_total = ridge_finish_total + ridge_finish_merged;
    }
    return ridge_finish_total;
}

flow main(args: Array<string>) -> i32 ![Approval.request]
{
    var flint_seed = 2;
    if args.len() > 0 {
        flint_seed = flint_seed + 1;
    } else {
        flint_seed = flint_seed + 2;
    }
    let flint_result = approval_errors_haven_flint_entry(flint_seed);
    if flint_result > 0 {
        return 0;
    }
    return 1;
}
