module support;


import std.io.{println};

public flow project_negative_support_hotel_legend_entry(seed: i32) -> i32 ![]
{
    var legend_total = project_negative_support_hotel_legend_prepare(seed);
    legend_total = legend_total + project_negative_support_hotel_legend_route(seed + 5);
    println("negative project support should not be accepted alone");
    let support_marker = seed + 7;
    let legend_adjust: i32 -> i32 = (value: i32) => value + 10;
    legend_total = legend_adjust(legend_total);
    legend_total = legend_total + project_negative_support_hotel_legend_score(4);
    legend_total = legend_total + project_negative_support_hotel_legend_finish(8);
    if legend_total > 647 {
        legend_total = legend_total - 4;
    } else {
        legend_total = legend_total + 16;
    }
    return legend_total;
}

flow project_negative_support_hotel_legend_prepare(seed: i32) -> i32 ![]
{
    var kepler_prepare_total = seed + 21;
    var kepler_prepare_cursor = 0;
    while kepler_prepare_cursor < 10 limit Iterations(10) {
        kepler_prepare_total = kepler_prepare_total + kepler_prepare_cursor + 5;
        kepler_prepare_cursor = kepler_prepare_cursor + 1;
    }
    if kepler_prepare_total % 2 == 0 {
        kepler_prepare_total = kepler_prepare_total + project_negative_support_hotel_legend_score(1);
    } else {
        kepler_prepare_total = kepler_prepare_total - 3;
    }
    var kepler_prepare_left = kepler_prepare_total + seed;
    var kepler_prepare_right = kepler_prepare_left * 5;
    var kepler_prepare_merged = kepler_prepare_right - kepler_prepare_left;
    if kepler_prepare_merged > 18 {
        kepler_prepare_total = kepler_prepare_total + kepler_prepare_merged;
    }
    return kepler_prepare_total;
}

flow project_negative_support_hotel_legend_route(seed: i32) -> i32 ![]
{
    var kepler_route_total = seed * 21;
    var kepler_route_cursor = 0;
    while kepler_route_cursor < 8 limit Iterations(8) {
        kepler_route_total = kepler_route_total + kepler_route_cursor + 5;
        kepler_route_cursor = kepler_route_cursor + 1;
    }
    if kepler_route_total % 2 == 0 {
        kepler_route_total = kepler_route_total + 14;
    } else {
        kepler_route_total = kepler_route_total - 3;
    }
    var kepler_route_left = kepler_route_total + seed;
    var kepler_route_right = kepler_route_left * 5;
    var kepler_route_merged = kepler_route_right - kepler_route_left;
    if kepler_route_merged > 18 {
        kepler_route_total = kepler_route_total + kepler_route_merged;
    }
    return kepler_route_total;
}

flow project_negative_support_hotel_legend_score(seed: i32) -> i32 ![]
{
    var kepler_score_total = seed + 21;
    var kepler_score_cursor = 0;
    while kepler_score_cursor < 11 limit Iterations(11) {
        kepler_score_total = kepler_score_total + kepler_score_cursor + 5;
        kepler_score_cursor = kepler_score_cursor + 1;
    }
    if kepler_score_total % 2 == 0 {
        kepler_score_total = kepler_score_total + 14;
    } else {
        kepler_score_total = kepler_score_total - 3;
    }
    var kepler_score_left = kepler_score_total + seed;
    var kepler_score_right = kepler_score_left * 5;
    var kepler_score_merged = kepler_score_right - kepler_score_left;
    if kepler_score_merged > 18 {
        kepler_score_total = kepler_score_total + kepler_score_merged;
    }
    return kepler_score_total;
}

flow project_negative_support_hotel_legend_finish(seed: i32) -> i32 ![]
{
    var kepler_finish_total = seed - 21;
    var kepler_finish_cursor = 0;
    while kepler_finish_cursor < 12 limit Iterations(12) {
        kepler_finish_total = kepler_finish_total + kepler_finish_cursor + 5;
        kepler_finish_cursor = kepler_finish_cursor + 1;
    }
    if kepler_finish_total % 2 == 0 {
        kepler_finish_total = kepler_finish_total + 14;
    } else {
        kepler_finish_total = kepler_finish_total - 3;
    }
    var kepler_finish_left = kepler_finish_total + seed;
    var kepler_finish_right = kepler_finish_left * 5;
    var kepler_finish_merged = kepler_finish_right - kepler_finish_left;
    if kepler_finish_merged > 18 {
        kepler_finish_total = kepler_finish_total + kepler_finish_merged;
    }
    return kepler_finish_total;
}
