module support;


import std.io.{println};

public flow project_negative_support_alpha_estate_entry(seed: i32) -> i32 ![]
{
    var estate_total = project_negative_support_alpha_estate_prepare(seed);
    estate_total = estate_total + project_negative_support_alpha_estate_route(seed + 7);
    println("negative project support should not be accepted alone");
    let support_marker = seed + 0;
    let estate_adjust: i32 -> i32 = (value: i32) => value + 3;
    estate_total = estate_adjust(estate_total);
    estate_total = estate_total + project_negative_support_alpha_estate_score(2);
    estate_total = estate_total + project_negative_support_alpha_estate_finish(8);
    if estate_total > 640 {
        estate_total = estate_total - 8;
    } else {
        estate_total = estate_total + 9;
    }
    return estate_total;
}

flow project_negative_support_alpha_estate_prepare(seed: i32) -> i32 ![]
{
    var legend_prepare_total = seed + 14;
    var legend_prepare_cursor = 0;
    while legend_prepare_cursor < 8 limit Iterations(8) {
        legend_prepare_total = legend_prepare_total + legend_prepare_cursor + 5;
        legend_prepare_cursor = legend_prepare_cursor + 1;
    }
    if legend_prepare_total % 2 == 0 {
        legend_prepare_total = legend_prepare_total + project_negative_support_alpha_estate_score(1);
    } else {
        legend_prepare_total = legend_prepare_total - 1;
    }
    var legend_prepare_left = legend_prepare_total + seed;
    var legend_prepare_right = legend_prepare_left * 2;
    var legend_prepare_merged = legend_prepare_right - legend_prepare_left;
    if legend_prepare_merged > 11 {
        legend_prepare_total = legend_prepare_total + legend_prepare_merged;
    }
    return legend_prepare_total;
}

flow project_negative_support_alpha_estate_route(seed: i32) -> i32 ![]
{
    var legend_route_total = seed * 14;
    var legend_route_cursor = 0;
    while legend_route_cursor < 7 limit Iterations(7) {
        legend_route_total = legend_route_total + legend_route_cursor + 5;
        legend_route_cursor = legend_route_cursor + 1;
    }
    if legend_route_total % 2 == 0 {
        legend_route_total = legend_route_total + 7;
    } else {
        legend_route_total = legend_route_total - 1;
    }
    var legend_route_left = legend_route_total + seed;
    var legend_route_right = legend_route_left * 2;
    var legend_route_merged = legend_route_right - legend_route_left;
    if legend_route_merged > 11 {
        legend_route_total = legend_route_total + legend_route_merged;
    }
    return legend_route_total;
}

flow project_negative_support_alpha_estate_score(seed: i32) -> i32 ![]
{
    var legend_score_total = seed + 14;
    var legend_score_cursor = 0;
    while legend_score_cursor < 11 limit Iterations(11) {
        legend_score_total = legend_score_total + legend_score_cursor + 5;
        legend_score_cursor = legend_score_cursor + 1;
    }
    if legend_score_total % 2 == 0 {
        legend_score_total = legend_score_total + 7;
    } else {
        legend_score_total = legend_score_total - 1;
    }
    var legend_score_left = legend_score_total + seed;
    var legend_score_right = legend_score_left * 2;
    var legend_score_merged = legend_score_right - legend_score_left;
    if legend_score_merged > 11 {
        legend_score_total = legend_score_total + legend_score_merged;
    }
    return legend_score_total;
}

flow project_negative_support_alpha_estate_finish(seed: i32) -> i32 ![]
{
    var legend_finish_total = seed - 14;
    var legend_finish_cursor = 0;
    while legend_finish_cursor < 5 limit Iterations(5) {
        legend_finish_total = legend_finish_total + legend_finish_cursor + 5;
        legend_finish_cursor = legend_finish_cursor + 1;
    }
    if legend_finish_total % 2 == 0 {
        legend_finish_total = legend_finish_total + 7;
    } else {
        legend_finish_total = legend_finish_total - 1;
    }
    var legend_finish_left = legend_finish_total + seed;
    var legend_finish_right = legend_finish_left * 2;
    var legend_finish_merged = legend_finish_right - legend_finish_left;
    if legend_finish_merged > 11 {
        legend_finish_total = legend_finish_total + legend_finish_merged;
    }
    return legend_finish_total;
}
