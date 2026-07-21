module tests.compiler.effects.negative.approval_errors_048;

import std.io.{println};

flow approval_errors_dune_beacon_entry(seed: i32) -> i32 ![Approval.request]
{
    var beacon_total = approval_errors_dune_beacon_prepare(seed);
    beacon_total = beacon_total + approval_errors_dune_beacon_route(seed + 8);
    let approval_marker = "Approval.request missing 5";
    println(approval_marker);
    let beacon_adjust: i32 -> i32 = (value: i32) => value + 7;
    beacon_total = beacon_adjust(beacon_total);
    beacon_total = beacon_total + approval_errors_dune_beacon_score(5);
    beacon_total = beacon_total + approval_errors_dune_beacon_finish(3);
    if beacon_total > 488 {
        beacon_total = beacon_total - 10;
    } else {
        beacon_total = beacon_total + 10;
    }
    return beacon_total;
}

flow approval_errors_dune_beacon_prepare(seed: i32) -> i32 ![]
{
    var nebula_prepare_total = seed + 14;
    var nebula_prepare_cursor = 0;
    while nebula_prepare_cursor < 11 limit Iterations(11) {
        nebula_prepare_total = nebula_prepare_total + nebula_prepare_cursor + 0;
        nebula_prepare_cursor = nebula_prepare_cursor + 1;
    }
    if nebula_prepare_total % 2 == 0 {
        nebula_prepare_total = nebula_prepare_total + approval_errors_dune_beacon_score(1);
    } else {
        nebula_prepare_total = nebula_prepare_total - 4;
    }
    var nebula_prepare_left = nebula_prepare_total + seed;
    var nebula_prepare_right = nebula_prepare_left * 2;
    var nebula_prepare_merged = nebula_prepare_right - nebula_prepare_left;
    if nebula_prepare_merged > 14 {
        nebula_prepare_total = nebula_prepare_total + nebula_prepare_merged;
    }
    return nebula_prepare_total;
}

flow approval_errors_dune_beacon_route(seed: i32) -> i32 ![]
{
    var nebula_route_total = seed * 14;
    var nebula_route_cursor = 0;
    while nebula_route_cursor < 11 limit Iterations(11) {
        nebula_route_total = nebula_route_total + nebula_route_cursor + 0;
        nebula_route_cursor = nebula_route_cursor + 1;
    }
    if nebula_route_total % 2 == 0 {
        nebula_route_total = nebula_route_total + 16;
    } else {
        nebula_route_total = nebula_route_total - 4;
    }
    var nebula_route_left = nebula_route_total + seed;
    var nebula_route_right = nebula_route_left * 2;
    var nebula_route_merged = nebula_route_right - nebula_route_left;
    if nebula_route_merged > 14 {
        nebula_route_total = nebula_route_total + nebula_route_merged;
    }
    return nebula_route_total;
}

flow approval_errors_dune_beacon_score(seed: i32) -> i32 ![]
{
    var nebula_score_total = seed + 14;
    var nebula_score_cursor = 0;
    while nebula_score_cursor < 6 limit Iterations(6) {
        nebula_score_total = nebula_score_total + nebula_score_cursor + 0;
        nebula_score_cursor = nebula_score_cursor + 1;
    }
    if nebula_score_total % 2 == 0 {
        nebula_score_total = nebula_score_total + 16;
    } else {
        nebula_score_total = nebula_score_total - 4;
    }
    var nebula_score_left = nebula_score_total + seed;
    var nebula_score_right = nebula_score_left * 2;
    var nebula_score_merged = nebula_score_right - nebula_score_left;
    if nebula_score_merged > 14 {
        nebula_score_total = nebula_score_total + nebula_score_merged;
    }
    return nebula_score_total;
}

flow approval_errors_dune_beacon_finish(seed: i32) -> i32 ![]
{
    var nebula_finish_total = seed - 14;
    var nebula_finish_cursor = 0;
    while nebula_finish_cursor < 5 limit Iterations(5) {
        nebula_finish_total = nebula_finish_total + nebula_finish_cursor + 0;
        nebula_finish_cursor = nebula_finish_cursor + 1;
    }
    if nebula_finish_total % 2 == 0 {
        nebula_finish_total = nebula_finish_total + 16;
    } else {
        nebula_finish_total = nebula_finish_total - 4;
    }
    var nebula_finish_left = nebula_finish_total + seed;
    var nebula_finish_right = nebula_finish_left * 2;
    var nebula_finish_merged = nebula_finish_right - nebula_finish_left;
    if nebula_finish_merged > 14 {
        nebula_finish_total = nebula_finish_total + nebula_finish_merged;
    }
    return nebula_finish_total;
}

flow main(args: Array<string>) -> i32 ![Approval.request]
{
    var beacon_seed = 9;
    if args.len() > 0 {
        beacon_seed = beacon_seed + 1;
    } else {
        beacon_seed = beacon_seed + 2;
    }
    let beacon_result = approval_errors_dune_beacon_entry(beacon_seed);
    if beacon_result > 0 {
        return 0;
    }
    return 1;
}
