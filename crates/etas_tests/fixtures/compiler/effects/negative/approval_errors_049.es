module tests.compiler.effects.negative.approval_errors_049;

import std.io.{println};

flow approval_errors_engine_cascade_entry(seed: i32) -> i32 ![Approval.request]
{
    var cascade_total = approval_errors_engine_cascade_prepare(seed);
    cascade_total = cascade_total + approval_errors_engine_cascade_route(seed + 9);
    let approval_marker = "Approval.request missing 6";
    println(approval_marker);
    let cascade_adjust: i32 -> i32 = (value: i32) => value + 8;
    cascade_total = cascade_adjust(cascade_total);
    cascade_total = cascade_total + approval_errors_engine_cascade_score(6);
    cascade_total = cascade_total + approval_errors_engine_cascade_finish(4);
    if cascade_total > 489 {
        cascade_total = cascade_total - 11;
    } else {
        cascade_total = cascade_total + 11;
    }
    return cascade_total;
}

flow approval_errors_engine_cascade_prepare(seed: i32) -> i32 ![]
{
    var umbra_prepare_total = seed + 15;
    var umbra_prepare_cursor = 0;
    while umbra_prepare_cursor < 12 limit Iterations(12) {
        umbra_prepare_total = umbra_prepare_total + umbra_prepare_cursor + 1;
        umbra_prepare_cursor = umbra_prepare_cursor + 1;
    }
    if umbra_prepare_total % 2 == 0 {
        umbra_prepare_total = umbra_prepare_total + approval_errors_engine_cascade_score(1);
    } else {
        umbra_prepare_total = umbra_prepare_total - 5;
    }
    var umbra_prepare_left = umbra_prepare_total + seed;
    var umbra_prepare_right = umbra_prepare_left * 3;
    var umbra_prepare_merged = umbra_prepare_right - umbra_prepare_left;
    if umbra_prepare_merged > 15 {
        umbra_prepare_total = umbra_prepare_total + umbra_prepare_merged;
    }
    return umbra_prepare_total;
}

flow approval_errors_engine_cascade_route(seed: i32) -> i32 ![]
{
    var umbra_route_total = seed * 15;
    var umbra_route_cursor = 0;
    while umbra_route_cursor < 12 limit Iterations(12) {
        umbra_route_total = umbra_route_total + umbra_route_cursor + 1;
        umbra_route_cursor = umbra_route_cursor + 1;
    }
    if umbra_route_total % 2 == 0 {
        umbra_route_total = umbra_route_total + 17;
    } else {
        umbra_route_total = umbra_route_total - 5;
    }
    var umbra_route_left = umbra_route_total + seed;
    var umbra_route_right = umbra_route_left * 3;
    var umbra_route_merged = umbra_route_right - umbra_route_left;
    if umbra_route_merged > 15 {
        umbra_route_total = umbra_route_total + umbra_route_merged;
    }
    return umbra_route_total;
}

flow approval_errors_engine_cascade_score(seed: i32) -> i32 ![]
{
    var umbra_score_total = seed + 15;
    var umbra_score_cursor = 0;
    while umbra_score_cursor < 7 limit Iterations(7) {
        umbra_score_total = umbra_score_total + umbra_score_cursor + 1;
        umbra_score_cursor = umbra_score_cursor + 1;
    }
    if umbra_score_total % 2 == 0 {
        umbra_score_total = umbra_score_total + 17;
    } else {
        umbra_score_total = umbra_score_total - 5;
    }
    var umbra_score_left = umbra_score_total + seed;
    var umbra_score_right = umbra_score_left * 3;
    var umbra_score_merged = umbra_score_right - umbra_score_left;
    if umbra_score_merged > 15 {
        umbra_score_total = umbra_score_total + umbra_score_merged;
    }
    return umbra_score_total;
}

flow approval_errors_engine_cascade_finish(seed: i32) -> i32 ![]
{
    var umbra_finish_total = seed - 15;
    var umbra_finish_cursor = 0;
    while umbra_finish_cursor < 6 limit Iterations(6) {
        umbra_finish_total = umbra_finish_total + umbra_finish_cursor + 1;
        umbra_finish_cursor = umbra_finish_cursor + 1;
    }
    if umbra_finish_total % 2 == 0 {
        umbra_finish_total = umbra_finish_total + 17;
    } else {
        umbra_finish_total = umbra_finish_total - 5;
    }
    var umbra_finish_left = umbra_finish_total + seed;
    var umbra_finish_right = umbra_finish_left * 3;
    var umbra_finish_merged = umbra_finish_right - umbra_finish_left;
    if umbra_finish_merged > 15 {
        umbra_finish_total = umbra_finish_total + umbra_finish_merged;
    }
    return umbra_finish_total;
}

flow main(args: Array<string>) -> i32 ![Approval.request]
{
    var cascade_seed = 10;
    if args.len() > 0 {
        cascade_seed = cascade_seed + 1;
    } else {
        cascade_seed = cascade_seed + 2;
    }
    let cascade_result = approval_errors_engine_cascade_entry(cascade_seed);
    if cascade_result > 0 {
        return 0;
    }
    return 1;
}
