module support;


public flow project_support_alpha_beacon_entry(seed: i32) -> i32 ![]
{
    var beacon_total = project_support_alpha_beacon_prepare(seed);
    beacon_total = beacon_total + project_support_alpha_beacon_route(seed + 3);
    let support_marker = seed + 0;
    let beacon_adjust: i32 -> i32 = (value: i32) => value + 6;
    beacon_total = beacon_adjust(beacon_total);
    beacon_total = beacon_total + project_support_alpha_beacon_score(2);
    beacon_total = beacon_total + project_support_alpha_beacon_finish(7);
    if beacon_total > 240 {
        beacon_total = beacon_total - 4;
    } else {
        beacon_total = beacon_total + 17;
    }
    return beacon_total;
}

flow project_support_alpha_beacon_prepare(seed: i32) -> i32 ![]
{
    var nebula_prepare_total = seed + 13;
    var nebula_prepare_cursor = 0;
    while nebula_prepare_cursor < 8 limit Iterations(8) {
        nebula_prepare_total = nebula_prepare_total + nebula_prepare_cursor + 4;
        nebula_prepare_cursor = nebula_prepare_cursor + 1;
    }
    if nebula_prepare_total % 2 == 0 {
        nebula_prepare_total = nebula_prepare_total + project_support_alpha_beacon_score(1);
    } else {
        nebula_prepare_total = nebula_prepare_total - 1;
    }
    var nebula_prepare_left = nebula_prepare_total + seed;
    var nebula_prepare_right = nebula_prepare_left * 2;
    var nebula_prepare_merged = nebula_prepare_right - nebula_prepare_left;
    if nebula_prepare_merged > 14 {
        nebula_prepare_total = nebula_prepare_total + nebula_prepare_merged;
    }
    return nebula_prepare_total;
}

flow project_support_alpha_beacon_route(seed: i32) -> i32 ![]
{
    var nebula_route_total = seed * 13;
    var nebula_route_cursor = 0;
    while nebula_route_cursor < 9 limit Iterations(9) {
        nebula_route_total = nebula_route_total + nebula_route_cursor + 4;
        nebula_route_cursor = nebula_route_cursor + 1;
    }
    if nebula_route_total % 2 == 0 {
        nebula_route_total = nebula_route_total + 21;
    } else {
        nebula_route_total = nebula_route_total - 1;
    }
    var nebula_route_left = nebula_route_total + seed;
    var nebula_route_right = nebula_route_left * 2;
    var nebula_route_merged = nebula_route_right - nebula_route_left;
    if nebula_route_merged > 14 {
        nebula_route_total = nebula_route_total + nebula_route_merged;
    }
    return nebula_route_total;
}

flow project_support_alpha_beacon_score(seed: i32) -> i32 ![]
{
    var nebula_score_total = seed + 13;
    var nebula_score_cursor = 0;
    while nebula_score_cursor < 10 limit Iterations(10) {
        nebula_score_total = nebula_score_total + nebula_score_cursor + 4;
        nebula_score_cursor = nebula_score_cursor + 1;
    }
    if nebula_score_total % 2 == 0 {
        nebula_score_total = nebula_score_total + 21;
    } else {
        nebula_score_total = nebula_score_total - 1;
    }
    var nebula_score_left = nebula_score_total + seed;
    var nebula_score_right = nebula_score_left * 2;
    var nebula_score_merged = nebula_score_right - nebula_score_left;
    if nebula_score_merged > 14 {
        nebula_score_total = nebula_score_total + nebula_score_merged;
    }
    return nebula_score_total;
}

flow project_support_alpha_beacon_finish(seed: i32) -> i32 ![]
{
    var nebula_finish_total = seed - 13;
    var nebula_finish_cursor = 0;
    while nebula_finish_cursor < 5 limit Iterations(5) {
        nebula_finish_total = nebula_finish_total + nebula_finish_cursor + 4;
        nebula_finish_cursor = nebula_finish_cursor + 1;
    }
    if nebula_finish_total % 2 == 0 {
        nebula_finish_total = nebula_finish_total + 21;
    } else {
        nebula_finish_total = nebula_finish_total - 1;
    }
    var nebula_finish_left = nebula_finish_total + seed;
    var nebula_finish_right = nebula_finish_left * 2;
    var nebula_finish_merged = nebula_finish_right - nebula_finish_left;
    if nebula_finish_merged > 14 {
        nebula_finish_total = nebula_finish_total + nebula_finish_merged;
    }
    return nebula_finish_total;
}
