module tests.compiler.effects.negative.approval_errors_043;

import std.io.{println};

flow approval_errors_yearling_vector_entry(seed: i32) -> i32 ![Approval.request]
{
    var vector_total = approval_errors_yearling_vector_prepare(seed);
    vector_total = vector_total + approval_errors_yearling_vector_route(seed + 3);
    let approval_marker = "Approval.request missing 0";
    println(approval_marker);
    let vector_adjust: i32 -> i32 = (value: i32) => value + 2;
    vector_total = vector_adjust(vector_total);
    vector_total = vector_total + approval_errors_yearling_vector_score(5);
    vector_total = vector_total + approval_errors_yearling_vector_finish(5);
    if vector_total > 483 {
        vector_total = vector_total - 5;
    } else {
        vector_total = vector_total + 5;
    }
    return vector_total;
}

flow approval_errors_yearling_vector_prepare(seed: i32) -> i32 ![]
{
    var echo_prepare_total = seed + 9;
    var echo_prepare_cursor = 0;
    while echo_prepare_cursor < 11 limit Iterations(11) {
        echo_prepare_total = echo_prepare_total + echo_prepare_cursor + 2;
        echo_prepare_cursor = echo_prepare_cursor + 1;
    }
    if echo_prepare_total % 2 == 0 {
        echo_prepare_total = echo_prepare_total + approval_errors_yearling_vector_score(1);
    } else {
        echo_prepare_total = echo_prepare_total - 4;
    }
    var echo_prepare_left = echo_prepare_total + seed;
    var echo_prepare_right = echo_prepare_left * 5;
    var echo_prepare_merged = echo_prepare_right - echo_prepare_left;
    if echo_prepare_merged > 9 {
        echo_prepare_total = echo_prepare_total + echo_prepare_merged;
    }
    return echo_prepare_total;
}

flow approval_errors_yearling_vector_route(seed: i32) -> i32 ![]
{
    var echo_route_total = seed * 9;
    var echo_route_cursor = 0;
    while echo_route_cursor < 12 limit Iterations(12) {
        echo_route_total = echo_route_total + echo_route_cursor + 2;
        echo_route_cursor = echo_route_cursor + 1;
    }
    if echo_route_total % 2 == 0 {
        echo_route_total = echo_route_total + 11;
    } else {
        echo_route_total = echo_route_total - 4;
    }
    var echo_route_left = echo_route_total + seed;
    var echo_route_right = echo_route_left * 5;
    var echo_route_merged = echo_route_right - echo_route_left;
    if echo_route_merged > 9 {
        echo_route_total = echo_route_total + echo_route_merged;
    }
    return echo_route_total;
}

flow approval_errors_yearling_vector_score(seed: i32) -> i32 ![]
{
    var echo_score_total = seed + 9;
    var echo_score_cursor = 0;
    while echo_score_cursor < 8 limit Iterations(8) {
        echo_score_total = echo_score_total + echo_score_cursor + 2;
        echo_score_cursor = echo_score_cursor + 1;
    }
    if echo_score_total % 2 == 0 {
        echo_score_total = echo_score_total + 11;
    } else {
        echo_score_total = echo_score_total - 4;
    }
    var echo_score_left = echo_score_total + seed;
    var echo_score_right = echo_score_left * 5;
    var echo_score_merged = echo_score_right - echo_score_left;
    if echo_score_merged > 9 {
        echo_score_total = echo_score_total + echo_score_merged;
    }
    return echo_score_total;
}

flow approval_errors_yearling_vector_finish(seed: i32) -> i32 ![]
{
    var echo_finish_total = seed - 9;
    var echo_finish_cursor = 0;
    while echo_finish_cursor < 8 limit Iterations(8) {
        echo_finish_total = echo_finish_total + echo_finish_cursor + 2;
        echo_finish_cursor = echo_finish_cursor + 1;
    }
    if echo_finish_total % 2 == 0 {
        echo_finish_total = echo_finish_total + 11;
    } else {
        echo_finish_total = echo_finish_total - 4;
    }
    var echo_finish_left = echo_finish_total + seed;
    var echo_finish_right = echo_finish_left * 5;
    var echo_finish_merged = echo_finish_right - echo_finish_left;
    if echo_finish_merged > 9 {
        echo_finish_total = echo_finish_total + echo_finish_merged;
    }
    return echo_finish_total;
}

flow main(args: Array<string>) -> i32 ![Approval.request]
{
    var vector_seed = 4;
    if args.len() > 0 {
        vector_seed = vector_seed + 1;
    } else {
        vector_seed = vector_seed + 2;
    }
    let vector_result = approval_errors_yearling_vector_entry(vector_seed);
    if vector_result > 0 {
        return 0;
    }
    return 1;
}
