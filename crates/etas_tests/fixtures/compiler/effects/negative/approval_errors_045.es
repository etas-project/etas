module tests.compiler.effects.negative.approval_errors_045;

import std.io.{println};

flow approval_errors_anchor_yearling_entry(seed: i32) -> i32 ![Approval.request]
{
    var yearling_total = approval_errors_anchor_yearling_prepare(seed);
    yearling_total = yearling_total + approval_errors_anchor_yearling_route(seed + 5);
    let approval_marker = "Approval.request missing 2";
    println(approval_marker);
    let yearling_adjust: i32 -> i32 = (value: i32) => value + 4;
    yearling_total = yearling_adjust(yearling_total);
    yearling_total = yearling_total + approval_errors_anchor_yearling_score(2);
    yearling_total = yearling_total + approval_errors_anchor_yearling_finish(7);
    if yearling_total > 485 {
        yearling_total = yearling_total - 7;
    } else {
        yearling_total = yearling_total + 7;
    }
    return yearling_total;
}

flow approval_errors_anchor_yearling_prepare(seed: i32) -> i32 ![]
{
    var silver_prepare_total = seed + 11;
    var silver_prepare_cursor = 0;
    while silver_prepare_cursor < 8 limit Iterations(8) {
        silver_prepare_total = silver_prepare_total + silver_prepare_cursor + 4;
        silver_prepare_cursor = silver_prepare_cursor + 1;
    }
    if silver_prepare_total % 2 == 0 {
        silver_prepare_total = silver_prepare_total + approval_errors_anchor_yearling_score(1);
    } else {
        silver_prepare_total = silver_prepare_total - 1;
    }
    var silver_prepare_left = silver_prepare_total + seed;
    var silver_prepare_right = silver_prepare_left * 3;
    var silver_prepare_merged = silver_prepare_right - silver_prepare_left;
    if silver_prepare_merged > 11 {
        silver_prepare_total = silver_prepare_total + silver_prepare_merged;
    }
    return silver_prepare_total;
}

flow approval_errors_anchor_yearling_route(seed: i32) -> i32 ![]
{
    var silver_route_total = seed * 11;
    var silver_route_cursor = 0;
    while silver_route_cursor < 8 limit Iterations(8) {
        silver_route_total = silver_route_total + silver_route_cursor + 4;
        silver_route_cursor = silver_route_cursor + 1;
    }
    if silver_route_total % 2 == 0 {
        silver_route_total = silver_route_total + 13;
    } else {
        silver_route_total = silver_route_total - 1;
    }
    var silver_route_left = silver_route_total + seed;
    var silver_route_right = silver_route_left * 3;
    var silver_route_merged = silver_route_right - silver_route_left;
    if silver_route_merged > 11 {
        silver_route_total = silver_route_total + silver_route_merged;
    }
    return silver_route_total;
}

flow approval_errors_anchor_yearling_score(seed: i32) -> i32 ![]
{
    var silver_score_total = seed + 11;
    var silver_score_cursor = 0;
    while silver_score_cursor < 10 limit Iterations(10) {
        silver_score_total = silver_score_total + silver_score_cursor + 4;
        silver_score_cursor = silver_score_cursor + 1;
    }
    if silver_score_total % 2 == 0 {
        silver_score_total = silver_score_total + 13;
    } else {
        silver_score_total = silver_score_total - 1;
    }
    var silver_score_left = silver_score_total + seed;
    var silver_score_right = silver_score_left * 3;
    var silver_score_merged = silver_score_right - silver_score_left;
    if silver_score_merged > 11 {
        silver_score_total = silver_score_total + silver_score_merged;
    }
    return silver_score_total;
}

flow approval_errors_anchor_yearling_finish(seed: i32) -> i32 ![]
{
    var silver_finish_total = seed - 11;
    var silver_finish_cursor = 0;
    while silver_finish_cursor < 10 limit Iterations(10) {
        silver_finish_total = silver_finish_total + silver_finish_cursor + 4;
        silver_finish_cursor = silver_finish_cursor + 1;
    }
    if silver_finish_total % 2 == 0 {
        silver_finish_total = silver_finish_total + 13;
    } else {
        silver_finish_total = silver_finish_total - 1;
    }
    var silver_finish_left = silver_finish_total + seed;
    var silver_finish_right = silver_finish_left * 3;
    var silver_finish_merged = silver_finish_right - silver_finish_left;
    if silver_finish_merged > 11 {
        silver_finish_total = silver_finish_total + silver_finish_merged;
    }
    return silver_finish_total;
}

flow main(args: Array<string>) -> i32 ![Approval.request]
{
    var yearling_seed = 6;
    if args.len() > 0 {
        yearling_seed = yearling_seed + 1;
    } else {
        yearling_seed = yearling_seed + 2;
    }
    let yearling_result = approval_errors_anchor_yearling_entry(yearling_seed);
    if yearling_result > 0 {
        return 0;
    }
    return 1;
}
