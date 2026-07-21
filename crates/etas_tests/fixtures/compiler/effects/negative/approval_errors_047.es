module tests.compiler.effects.negative.approval_errors_047;

import std.io.{println};

flow approval_errors_cascade_anchor_entry(seed: i32) -> i32 ![Approval.request]
{
    var anchor_total = approval_errors_cascade_anchor_prepare(seed);
    anchor_total = anchor_total + approval_errors_cascade_anchor_route(seed + 7);
    let approval_marker = "Approval.request missing 4";
    println(approval_marker);
    let anchor_adjust: i32 -> i32 = (value: i32) => value + 6;
    anchor_total = anchor_adjust(anchor_total);
    anchor_total = anchor_total + approval_errors_cascade_anchor_score(4);
    anchor_total = anchor_total + approval_errors_cascade_anchor_finish(9);
    if anchor_total > 487 {
        anchor_total = anchor_total - 9;
    } else {
        anchor_total = anchor_total + 9;
    }
    return anchor_total;
}

flow approval_errors_cascade_anchor_prepare(seed: i32) -> i32 ![]
{
    var glacier_prepare_total = seed + 13;
    var glacier_prepare_cursor = 0;
    while glacier_prepare_cursor < 10 limit Iterations(10) {
        glacier_prepare_total = glacier_prepare_total + glacier_prepare_cursor + 6;
        glacier_prepare_cursor = glacier_prepare_cursor + 1;
    }
    if glacier_prepare_total % 2 == 0 {
        glacier_prepare_total = glacier_prepare_total + approval_errors_cascade_anchor_score(1);
    } else {
        glacier_prepare_total = glacier_prepare_total - 3;
    }
    var glacier_prepare_left = glacier_prepare_total + seed;
    var glacier_prepare_right = glacier_prepare_left * 5;
    var glacier_prepare_merged = glacier_prepare_right - glacier_prepare_left;
    if glacier_prepare_merged > 13 {
        glacier_prepare_total = glacier_prepare_total + glacier_prepare_merged;
    }
    return glacier_prepare_total;
}

flow approval_errors_cascade_anchor_route(seed: i32) -> i32 ![]
{
    var glacier_route_total = seed * 13;
    var glacier_route_cursor = 0;
    while glacier_route_cursor < 10 limit Iterations(10) {
        glacier_route_total = glacier_route_total + glacier_route_cursor + 6;
        glacier_route_cursor = glacier_route_cursor + 1;
    }
    if glacier_route_total % 2 == 0 {
        glacier_route_total = glacier_route_total + 15;
    } else {
        glacier_route_total = glacier_route_total - 3;
    }
    var glacier_route_left = glacier_route_total + seed;
    var glacier_route_right = glacier_route_left * 5;
    var glacier_route_merged = glacier_route_right - glacier_route_left;
    if glacier_route_merged > 13 {
        glacier_route_total = glacier_route_total + glacier_route_merged;
    }
    return glacier_route_total;
}

flow approval_errors_cascade_anchor_score(seed: i32) -> i32 ![]
{
    var glacier_score_total = seed + 13;
    var glacier_score_cursor = 0;
    while glacier_score_cursor < 12 limit Iterations(12) {
        glacier_score_total = glacier_score_total + glacier_score_cursor + 6;
        glacier_score_cursor = glacier_score_cursor + 1;
    }
    if glacier_score_total % 2 == 0 {
        glacier_score_total = glacier_score_total + 15;
    } else {
        glacier_score_total = glacier_score_total - 3;
    }
    var glacier_score_left = glacier_score_total + seed;
    var glacier_score_right = glacier_score_left * 5;
    var glacier_score_merged = glacier_score_right - glacier_score_left;
    if glacier_score_merged > 13 {
        glacier_score_total = glacier_score_total + glacier_score_merged;
    }
    return glacier_score_total;
}

flow approval_errors_cascade_anchor_finish(seed: i32) -> i32 ![]
{
    var glacier_finish_total = seed - 13;
    var glacier_finish_cursor = 0;
    while glacier_finish_cursor < 12 limit Iterations(12) {
        glacier_finish_total = glacier_finish_total + glacier_finish_cursor + 6;
        glacier_finish_cursor = glacier_finish_cursor + 1;
    }
    if glacier_finish_total % 2 == 0 {
        glacier_finish_total = glacier_finish_total + 15;
    } else {
        glacier_finish_total = glacier_finish_total - 3;
    }
    var glacier_finish_left = glacier_finish_total + seed;
    var glacier_finish_right = glacier_finish_left * 5;
    var glacier_finish_merged = glacier_finish_right - glacier_finish_left;
    if glacier_finish_merged > 13 {
        glacier_finish_total = glacier_finish_total + glacier_finish_merged;
    }
    return glacier_finish_total;
}

flow main(args: Array<string>) -> i32 ![Approval.request]
{
    var anchor_seed = 8;
    if args.len() > 0 {
        anchor_seed = anchor_seed + 1;
    } else {
        anchor_seed = anchor_seed + 2;
    }
    let anchor_result = approval_errors_cascade_anchor_entry(anchor_seed);
    if anchor_result > 0 {
        return 0;
    }
    return 1;
}
