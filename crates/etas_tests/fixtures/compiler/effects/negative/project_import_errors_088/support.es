module support;


import std.io.{println};

public flow project_negative_support_bravo_forest_entry(seed: i32) -> i32 ![]
{
    var forest_total = project_negative_support_bravo_forest_prepare(seed);
    forest_total = forest_total + project_negative_support_bravo_forest_route(seed + 8);
    println("negative project support should not be accepted alone");
    let support_marker = seed + 1;
    let forest_adjust: i32 -> i32 = (value: i32) => value + 4;
    forest_total = forest_adjust(forest_total);
    forest_total = forest_total + project_negative_support_bravo_forest_score(3);
    forest_total = forest_total + project_negative_support_bravo_forest_finish(9);
    if forest_total > 641 {
        forest_total = forest_total - 9;
    } else {
        forest_total = forest_total + 10;
    }
    return forest_total;
}

flow project_negative_support_bravo_forest_prepare(seed: i32) -> i32 ![]
{
    var terra_prepare_total = seed + 15;
    var terra_prepare_cursor = 0;
    while terra_prepare_cursor < 9 limit Iterations(9) {
        terra_prepare_total = terra_prepare_total + terra_prepare_cursor + 6;
        terra_prepare_cursor = terra_prepare_cursor + 1;
    }
    if terra_prepare_total % 2 == 0 {
        terra_prepare_total = terra_prepare_total + project_negative_support_bravo_forest_score(1);
    } else {
        terra_prepare_total = terra_prepare_total - 2;
    }
    var terra_prepare_left = terra_prepare_total + seed;
    var terra_prepare_right = terra_prepare_left * 3;
    var terra_prepare_merged = terra_prepare_right - terra_prepare_left;
    if terra_prepare_merged > 12 {
        terra_prepare_total = terra_prepare_total + terra_prepare_merged;
    }
    return terra_prepare_total;
}

flow project_negative_support_bravo_forest_route(seed: i32) -> i32 ![]
{
    var terra_route_total = seed * 15;
    var terra_route_cursor = 0;
    while terra_route_cursor < 8 limit Iterations(8) {
        terra_route_total = terra_route_total + terra_route_cursor + 6;
        terra_route_cursor = terra_route_cursor + 1;
    }
    if terra_route_total % 2 == 0 {
        terra_route_total = terra_route_total + 8;
    } else {
        terra_route_total = terra_route_total - 2;
    }
    var terra_route_left = terra_route_total + seed;
    var terra_route_right = terra_route_left * 3;
    var terra_route_merged = terra_route_right - terra_route_left;
    if terra_route_merged > 12 {
        terra_route_total = terra_route_total + terra_route_merged;
    }
    return terra_route_total;
}

flow project_negative_support_bravo_forest_score(seed: i32) -> i32 ![]
{
    var terra_score_total = seed + 15;
    var terra_score_cursor = 0;
    while terra_score_cursor < 12 limit Iterations(12) {
        terra_score_total = terra_score_total + terra_score_cursor + 6;
        terra_score_cursor = terra_score_cursor + 1;
    }
    if terra_score_total % 2 == 0 {
        terra_score_total = terra_score_total + 8;
    } else {
        terra_score_total = terra_score_total - 2;
    }
    var terra_score_left = terra_score_total + seed;
    var terra_score_right = terra_score_left * 3;
    var terra_score_merged = terra_score_right - terra_score_left;
    if terra_score_merged > 12 {
        terra_score_total = terra_score_total + terra_score_merged;
    }
    return terra_score_total;
}

flow project_negative_support_bravo_forest_finish(seed: i32) -> i32 ![]
{
    var terra_finish_total = seed - 15;
    var terra_finish_cursor = 0;
    while terra_finish_cursor < 6 limit Iterations(6) {
        terra_finish_total = terra_finish_total + terra_finish_cursor + 6;
        terra_finish_cursor = terra_finish_cursor + 1;
    }
    if terra_finish_total % 2 == 0 {
        terra_finish_total = terra_finish_total + 8;
    } else {
        terra_finish_total = terra_finish_total - 2;
    }
    var terra_finish_left = terra_finish_total + seed;
    var terra_finish_right = terra_finish_left * 3;
    var terra_finish_merged = terra_finish_right - terra_finish_left;
    if terra_finish_merged > 12 {
        terra_finish_total = terra_finish_total + terra_finish_merged;
    }
    return terra_finish_total;
}
