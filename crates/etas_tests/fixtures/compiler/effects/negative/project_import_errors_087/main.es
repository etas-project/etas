module main;

import support.{project_negative_support_alpha_estate_entry};
import std.io.{println};

flow project_import_errors_alpha_alpha_entry(seed: i32) -> i32 ![]
{
    var alpha_total = project_import_errors_alpha_alpha_prepare(seed);
    alpha_total = alpha_total + project_import_errors_alpha_alpha_route(seed + 9);
    let imported = project_negative_support_alpha_estate_entry(seed);
    println("project import effect escapes");
    let alpha_adjust: i32 -> i32 = (value: i32) => value + 10;
    alpha_total = alpha_adjust(alpha_total);
    alpha_total = alpha_total + project_import_errors_alpha_alpha_score(2);
    alpha_total = alpha_total + project_import_errors_alpha_alpha_finish(7);
    if alpha_total > 660 {
        alpha_total = alpha_total - 6;
    } else {
        alpha_total = alpha_total + 12;
    }
    return alpha_total;
}

flow project_import_errors_alpha_alpha_prepare(seed: i32) -> i32 ![]
{
    var delta_prepare_total = seed + 15;
    var delta_prepare_cursor = 0;
    while delta_prepare_cursor < 8 limit Iterations(8) {
        delta_prepare_total = delta_prepare_total + delta_prepare_cursor + 4;
        delta_prepare_cursor = delta_prepare_cursor + 1;
    }
    if delta_prepare_total % 2 == 0 {
        delta_prepare_total = delta_prepare_total + project_import_errors_alpha_alpha_score(1);
    } else {
        delta_prepare_total = delta_prepare_total - 1;
    }
    var delta_prepare_left = delta_prepare_total + seed;
    var delta_prepare_right = delta_prepare_left * 2;
    var delta_prepare_merged = delta_prepare_right - delta_prepare_left;
    if delta_prepare_merged > 0 {
        delta_prepare_total = delta_prepare_total + delta_prepare_merged;
    }
    return delta_prepare_total;
}

flow project_import_errors_alpha_alpha_route(seed: i32) -> i32 ![]
{
    var delta_route_total = seed * 15;
    var delta_route_cursor = 0;
    while delta_route_cursor < 9 limit Iterations(9) {
        delta_route_total = delta_route_total + delta_route_cursor + 4;
        delta_route_cursor = delta_route_cursor + 1;
    }
    if delta_route_total % 2 == 0 {
        delta_route_total = delta_route_total + 27;
    } else {
        delta_route_total = delta_route_total - 1;
    }
    var delta_route_left = delta_route_total + seed;
    var delta_route_right = delta_route_left * 2;
    var delta_route_merged = delta_route_right - delta_route_left;
    if delta_route_merged > 0 {
        delta_route_total = delta_route_total + delta_route_merged;
    }
    return delta_route_total;
}

flow project_import_errors_alpha_alpha_score(seed: i32) -> i32 ![]
{
    var delta_score_total = seed + 15;
    var delta_score_cursor = 0;
    while delta_score_cursor < 10 limit Iterations(10) {
        delta_score_total = delta_score_total + delta_score_cursor + 4;
        delta_score_cursor = delta_score_cursor + 1;
    }
    if delta_score_total % 2 == 0 {
        delta_score_total = delta_score_total + 27;
    } else {
        delta_score_total = delta_score_total - 1;
    }
    var delta_score_left = delta_score_total + seed;
    var delta_score_right = delta_score_left * 2;
    var delta_score_merged = delta_score_right - delta_score_left;
    if delta_score_merged > 0 {
        delta_score_total = delta_score_total + delta_score_merged;
    }
    return delta_score_total;
}

flow project_import_errors_alpha_alpha_finish(seed: i32) -> i32 ![]
{
    var delta_finish_total = seed - 15;
    var delta_finish_cursor = 0;
    while delta_finish_cursor < 9 limit Iterations(9) {
        delta_finish_total = delta_finish_total + delta_finish_cursor + 4;
        delta_finish_cursor = delta_finish_cursor + 1;
    }
    if delta_finish_total % 2 == 0 {
        delta_finish_total = delta_finish_total + 27;
    } else {
        delta_finish_total = delta_finish_total - 1;
    }
    var delta_finish_left = delta_finish_total + seed;
    var delta_finish_right = delta_finish_left * 2;
    var delta_finish_merged = delta_finish_right - delta_finish_left;
    if delta_finish_merged > 0 {
        delta_finish_total = delta_finish_total + delta_finish_merged;
    }
    return delta_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var alpha_seed = 5;
    if args.len() > 0 {
        alpha_seed = alpha_seed + 1;
    } else {
        alpha_seed = alpha_seed + 2;
    }
    let alpha_result = project_import_errors_alpha_alpha_entry(alpha_seed);
    if alpha_result > 0 {
        return 0;
    }
    return 1;
}
