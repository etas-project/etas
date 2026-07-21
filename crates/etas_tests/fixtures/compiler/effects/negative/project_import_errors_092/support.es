module support;


import std.io.{println};

public flow project_negative_support_foxtrot_junction_entry(seed: i32) -> i32 ![]
{
    var junction_total = project_negative_support_foxtrot_junction_prepare(seed);
    junction_total = junction_total + project_negative_support_foxtrot_junction_route(seed + 3);
    println("negative project support should not be accepted alone");
    let support_marker = seed + 5;
    let junction_adjust: i32 -> i32 = (value: i32) => value + 8;
    junction_total = junction_adjust(junction_total);
    junction_total = junction_total + project_negative_support_foxtrot_junction_score(2);
    junction_total = junction_total + project_negative_support_foxtrot_junction_finish(6);
    if junction_total > 645 {
        junction_total = junction_total - 2;
    } else {
        junction_total = junction_total + 14;
    }
    return junction_total;
}

flow project_negative_support_foxtrot_junction_prepare(seed: i32) -> i32 ![]
{
    var willow_prepare_total = seed + 19;
    var willow_prepare_cursor = 0;
    while willow_prepare_cursor < 8 limit Iterations(8) {
        willow_prepare_total = willow_prepare_total + willow_prepare_cursor + 3;
        willow_prepare_cursor = willow_prepare_cursor + 1;
    }
    if willow_prepare_total % 2 == 0 {
        willow_prepare_total = willow_prepare_total + project_negative_support_foxtrot_junction_score(1);
    } else {
        willow_prepare_total = willow_prepare_total - 1;
    }
    var willow_prepare_left = willow_prepare_total + seed;
    var willow_prepare_right = willow_prepare_left * 3;
    var willow_prepare_merged = willow_prepare_right - willow_prepare_left;
    if willow_prepare_merged > 16 {
        willow_prepare_total = willow_prepare_total + willow_prepare_merged;
    }
    return willow_prepare_total;
}

flow project_negative_support_foxtrot_junction_route(seed: i32) -> i32 ![]
{
    var willow_route_total = seed * 19;
    var willow_route_cursor = 0;
    while willow_route_cursor < 12 limit Iterations(12) {
        willow_route_total = willow_route_total + willow_route_cursor + 3;
        willow_route_cursor = willow_route_cursor + 1;
    }
    if willow_route_total % 2 == 0 {
        willow_route_total = willow_route_total + 12;
    } else {
        willow_route_total = willow_route_total - 1;
    }
    var willow_route_left = willow_route_total + seed;
    var willow_route_right = willow_route_left * 3;
    var willow_route_merged = willow_route_right - willow_route_left;
    if willow_route_merged > 16 {
        willow_route_total = willow_route_total + willow_route_merged;
    }
    return willow_route_total;
}

flow project_negative_support_foxtrot_junction_score(seed: i32) -> i32 ![]
{
    var willow_score_total = seed + 19;
    var willow_score_cursor = 0;
    while willow_score_cursor < 9 limit Iterations(9) {
        willow_score_total = willow_score_total + willow_score_cursor + 3;
        willow_score_cursor = willow_score_cursor + 1;
    }
    if willow_score_total % 2 == 0 {
        willow_score_total = willow_score_total + 12;
    } else {
        willow_score_total = willow_score_total - 1;
    }
    var willow_score_left = willow_score_total + seed;
    var willow_score_right = willow_score_left * 3;
    var willow_score_merged = willow_score_right - willow_score_left;
    if willow_score_merged > 16 {
        willow_score_total = willow_score_total + willow_score_merged;
    }
    return willow_score_total;
}

flow project_negative_support_foxtrot_junction_finish(seed: i32) -> i32 ![]
{
    var willow_finish_total = seed - 19;
    var willow_finish_cursor = 0;
    while willow_finish_cursor < 10 limit Iterations(10) {
        willow_finish_total = willow_finish_total + willow_finish_cursor + 3;
        willow_finish_cursor = willow_finish_cursor + 1;
    }
    if willow_finish_total % 2 == 0 {
        willow_finish_total = willow_finish_total + 12;
    } else {
        willow_finish_total = willow_finish_total - 1;
    }
    var willow_finish_left = willow_finish_total + seed;
    var willow_finish_right = willow_finish_left * 3;
    var willow_finish_merged = willow_finish_right - willow_finish_left;
    if willow_finish_merged > 16 {
        willow_finish_total = willow_finish_total + willow_finish_merged;
    }
    return willow_finish_total;
}
