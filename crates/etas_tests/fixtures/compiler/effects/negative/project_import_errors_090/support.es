module support;


import std.io.{println};

public flow project_negative_support_delta_horizon_entry(seed: i32) -> i32 ![]
{
    var horizon_total = project_negative_support_delta_horizon_prepare(seed);
    horizon_total = horizon_total + project_negative_support_delta_horizon_route(seed + 1);
    println("negative project support should not be accepted alone");
    let support_marker = seed + 3;
    let horizon_adjust: i32 -> i32 = (value: i32) => value + 6;
    horizon_total = horizon_adjust(horizon_total);
    horizon_total = horizon_total + project_negative_support_delta_horizon_score(5);
    horizon_total = horizon_total + project_negative_support_delta_horizon_finish(4);
    if horizon_total > 643 {
        horizon_total = horizon_total - 11;
    } else {
        horizon_total = horizon_total + 12;
    }
    return horizon_total;
}

flow project_negative_support_delta_horizon_prepare(seed: i32) -> i32 ![]
{
    var india_prepare_total = seed + 17;
    var india_prepare_cursor = 0;
    while india_prepare_cursor < 11 limit Iterations(11) {
        india_prepare_total = india_prepare_total + india_prepare_cursor + 1;
        india_prepare_cursor = india_prepare_cursor + 1;
    }
    if india_prepare_total % 2 == 0 {
        india_prepare_total = india_prepare_total + project_negative_support_delta_horizon_score(1);
    } else {
        india_prepare_total = india_prepare_total - 4;
    }
    var india_prepare_left = india_prepare_total + seed;
    var india_prepare_right = india_prepare_left * 5;
    var india_prepare_merged = india_prepare_right - india_prepare_left;
    if india_prepare_merged > 14 {
        india_prepare_total = india_prepare_total + india_prepare_merged;
    }
    return india_prepare_total;
}

flow project_negative_support_delta_horizon_route(seed: i32) -> i32 ![]
{
    var india_route_total = seed * 17;
    var india_route_cursor = 0;
    while india_route_cursor < 10 limit Iterations(10) {
        india_route_total = india_route_total + india_route_cursor + 1;
        india_route_cursor = india_route_cursor + 1;
    }
    if india_route_total % 2 == 0 {
        india_route_total = india_route_total + 10;
    } else {
        india_route_total = india_route_total - 4;
    }
    var india_route_left = india_route_total + seed;
    var india_route_right = india_route_left * 5;
    var india_route_merged = india_route_right - india_route_left;
    if india_route_merged > 14 {
        india_route_total = india_route_total + india_route_merged;
    }
    return india_route_total;
}

flow project_negative_support_delta_horizon_score(seed: i32) -> i32 ![]
{
    var india_score_total = seed + 17;
    var india_score_cursor = 0;
    while india_score_cursor < 7 limit Iterations(7) {
        india_score_total = india_score_total + india_score_cursor + 1;
        india_score_cursor = india_score_cursor + 1;
    }
    if india_score_total % 2 == 0 {
        india_score_total = india_score_total + 10;
    } else {
        india_score_total = india_score_total - 4;
    }
    var india_score_left = india_score_total + seed;
    var india_score_right = india_score_left * 5;
    var india_score_merged = india_score_right - india_score_left;
    if india_score_merged > 14 {
        india_score_total = india_score_total + india_score_merged;
    }
    return india_score_total;
}

flow project_negative_support_delta_horizon_finish(seed: i32) -> i32 ![]
{
    var india_finish_total = seed - 17;
    var india_finish_cursor = 0;
    while india_finish_cursor < 8 limit Iterations(8) {
        india_finish_total = india_finish_total + india_finish_cursor + 1;
        india_finish_cursor = india_finish_cursor + 1;
    }
    if india_finish_total % 2 == 0 {
        india_finish_total = india_finish_total + 10;
    } else {
        india_finish_total = india_finish_total - 4;
    }
    var india_finish_left = india_finish_total + seed;
    var india_finish_right = india_finish_left * 5;
    var india_finish_merged = india_finish_right - india_finish_left;
    if india_finish_merged > 14 {
        india_finish_total = india_finish_total + india_finish_merged;
    }
    return india_finish_total;
}
