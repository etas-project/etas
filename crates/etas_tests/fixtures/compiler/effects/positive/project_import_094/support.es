module support;


public flow project_support_bravo_cascade_entry(seed: i32) -> i32 ![]
{
    var cascade_total = project_support_bravo_cascade_prepare(seed);
    cascade_total = cascade_total + project_support_bravo_cascade_route(seed + 4);
    let support_marker = seed + 1;
    let cascade_adjust: i32 -> i32 = (value: i32) => value + 7;
    cascade_total = cascade_adjust(cascade_total);
    cascade_total = cascade_total + project_support_bravo_cascade_score(3);
    cascade_total = cascade_total + project_support_bravo_cascade_finish(8);
    if cascade_total > 241 {
        cascade_total = cascade_total - 5;
    } else {
        cascade_total = cascade_total + 18;
    }
    return cascade_total;
}

flow project_support_bravo_cascade_prepare(seed: i32) -> i32 ![]
{
    var umbra_prepare_total = seed + 14;
    var umbra_prepare_cursor = 0;
    while umbra_prepare_cursor < 9 limit Iterations(9) {
        umbra_prepare_total = umbra_prepare_total + umbra_prepare_cursor + 5;
        umbra_prepare_cursor = umbra_prepare_cursor + 1;
    }
    if umbra_prepare_total % 2 == 0 {
        umbra_prepare_total = umbra_prepare_total + project_support_bravo_cascade_score(1);
    } else {
        umbra_prepare_total = umbra_prepare_total - 2;
    }
    var umbra_prepare_left = umbra_prepare_total + seed;
    var umbra_prepare_right = umbra_prepare_left * 3;
    var umbra_prepare_merged = umbra_prepare_right - umbra_prepare_left;
    if umbra_prepare_merged > 15 {
        umbra_prepare_total = umbra_prepare_total + umbra_prepare_merged;
    }
    return umbra_prepare_total;
}

flow project_support_bravo_cascade_route(seed: i32) -> i32 ![]
{
    var umbra_route_total = seed * 14;
    var umbra_route_cursor = 0;
    while umbra_route_cursor < 10 limit Iterations(10) {
        umbra_route_total = umbra_route_total + umbra_route_cursor + 5;
        umbra_route_cursor = umbra_route_cursor + 1;
    }
    if umbra_route_total % 2 == 0 {
        umbra_route_total = umbra_route_total + 22;
    } else {
        umbra_route_total = umbra_route_total - 2;
    }
    var umbra_route_left = umbra_route_total + seed;
    var umbra_route_right = umbra_route_left * 3;
    var umbra_route_merged = umbra_route_right - umbra_route_left;
    if umbra_route_merged > 15 {
        umbra_route_total = umbra_route_total + umbra_route_merged;
    }
    return umbra_route_total;
}

flow project_support_bravo_cascade_score(seed: i32) -> i32 ![]
{
    var umbra_score_total = seed + 14;
    var umbra_score_cursor = 0;
    while umbra_score_cursor < 11 limit Iterations(11) {
        umbra_score_total = umbra_score_total + umbra_score_cursor + 5;
        umbra_score_cursor = umbra_score_cursor + 1;
    }
    if umbra_score_total % 2 == 0 {
        umbra_score_total = umbra_score_total + 22;
    } else {
        umbra_score_total = umbra_score_total - 2;
    }
    var umbra_score_left = umbra_score_total + seed;
    var umbra_score_right = umbra_score_left * 3;
    var umbra_score_merged = umbra_score_right - umbra_score_left;
    if umbra_score_merged > 15 {
        umbra_score_total = umbra_score_total + umbra_score_merged;
    }
    return umbra_score_total;
}

flow project_support_bravo_cascade_finish(seed: i32) -> i32 ![]
{
    var umbra_finish_total = seed - 14;
    var umbra_finish_cursor = 0;
    while umbra_finish_cursor < 6 limit Iterations(6) {
        umbra_finish_total = umbra_finish_total + umbra_finish_cursor + 5;
        umbra_finish_cursor = umbra_finish_cursor + 1;
    }
    if umbra_finish_total % 2 == 0 {
        umbra_finish_total = umbra_finish_total + 22;
    } else {
        umbra_finish_total = umbra_finish_total - 2;
    }
    var umbra_finish_left = umbra_finish_total + seed;
    var umbra_finish_right = umbra_finish_left * 3;
    var umbra_finish_merged = umbra_finish_right - umbra_finish_left;
    if umbra_finish_merged > 15 {
        umbra_finish_total = umbra_finish_total + umbra_finish_merged;
    }
    return umbra_finish_total;
}
