module support;


import std.io.{println};

public flow project_negative_support_golf_keeper_entry(seed: i32) -> i32 ![]
{
    var keeper_total = project_negative_support_golf_keeper_prepare(seed);
    keeper_total = keeper_total + project_negative_support_golf_keeper_route(seed + 4);
    println("negative project support should not be accepted alone");
    let support_marker = seed + 6;
    let keeper_adjust: i32 -> i32 = (value: i32) => value + 9;
    keeper_total = keeper_adjust(keeper_total);
    keeper_total = keeper_total + project_negative_support_golf_keeper_score(3);
    keeper_total = keeper_total + project_negative_support_golf_keeper_finish(7);
    if keeper_total > 646 {
        keeper_total = keeper_total - 3;
    } else {
        keeper_total = keeper_total + 15;
    }
    return keeper_total;
}

flow project_negative_support_golf_keeper_prepare(seed: i32) -> i32 ![]
{
    var dynamo_prepare_total = seed + 20;
    var dynamo_prepare_cursor = 0;
    while dynamo_prepare_cursor < 9 limit Iterations(9) {
        dynamo_prepare_total = dynamo_prepare_total + dynamo_prepare_cursor + 4;
        dynamo_prepare_cursor = dynamo_prepare_cursor + 1;
    }
    if dynamo_prepare_total % 2 == 0 {
        dynamo_prepare_total = dynamo_prepare_total + project_negative_support_golf_keeper_score(1);
    } else {
        dynamo_prepare_total = dynamo_prepare_total - 2;
    }
    var dynamo_prepare_left = dynamo_prepare_total + seed;
    var dynamo_prepare_right = dynamo_prepare_left * 4;
    var dynamo_prepare_merged = dynamo_prepare_right - dynamo_prepare_left;
    if dynamo_prepare_merged > 17 {
        dynamo_prepare_total = dynamo_prepare_total + dynamo_prepare_merged;
    }
    return dynamo_prepare_total;
}

flow project_negative_support_golf_keeper_route(seed: i32) -> i32 ![]
{
    var dynamo_route_total = seed * 20;
    var dynamo_route_cursor = 0;
    while dynamo_route_cursor < 7 limit Iterations(7) {
        dynamo_route_total = dynamo_route_total + dynamo_route_cursor + 4;
        dynamo_route_cursor = dynamo_route_cursor + 1;
    }
    if dynamo_route_total % 2 == 0 {
        dynamo_route_total = dynamo_route_total + 13;
    } else {
        dynamo_route_total = dynamo_route_total - 2;
    }
    var dynamo_route_left = dynamo_route_total + seed;
    var dynamo_route_right = dynamo_route_left * 4;
    var dynamo_route_merged = dynamo_route_right - dynamo_route_left;
    if dynamo_route_merged > 17 {
        dynamo_route_total = dynamo_route_total + dynamo_route_merged;
    }
    return dynamo_route_total;
}

flow project_negative_support_golf_keeper_score(seed: i32) -> i32 ![]
{
    var dynamo_score_total = seed + 20;
    var dynamo_score_cursor = 0;
    while dynamo_score_cursor < 10 limit Iterations(10) {
        dynamo_score_total = dynamo_score_total + dynamo_score_cursor + 4;
        dynamo_score_cursor = dynamo_score_cursor + 1;
    }
    if dynamo_score_total % 2 == 0 {
        dynamo_score_total = dynamo_score_total + 13;
    } else {
        dynamo_score_total = dynamo_score_total - 2;
    }
    var dynamo_score_left = dynamo_score_total + seed;
    var dynamo_score_right = dynamo_score_left * 4;
    var dynamo_score_merged = dynamo_score_right - dynamo_score_left;
    if dynamo_score_merged > 17 {
        dynamo_score_total = dynamo_score_total + dynamo_score_merged;
    }
    return dynamo_score_total;
}

flow project_negative_support_golf_keeper_finish(seed: i32) -> i32 ![]
{
    var dynamo_finish_total = seed - 20;
    var dynamo_finish_cursor = 0;
    while dynamo_finish_cursor < 11 limit Iterations(11) {
        dynamo_finish_total = dynamo_finish_total + dynamo_finish_cursor + 4;
        dynamo_finish_cursor = dynamo_finish_cursor + 1;
    }
    if dynamo_finish_total % 2 == 0 {
        dynamo_finish_total = dynamo_finish_total + 13;
    } else {
        dynamo_finish_total = dynamo_finish_total - 2;
    }
    var dynamo_finish_left = dynamo_finish_total + seed;
    var dynamo_finish_right = dynamo_finish_left * 4;
    var dynamo_finish_merged = dynamo_finish_right - dynamo_finish_left;
    if dynamo_finish_merged > 17 {
        dynamo_finish_total = dynamo_finish_total + dynamo_finish_merged;
    }
    return dynamo_finish_total;
}
