module main;

import support.{project_negative_support_hotel_legend_entry};
import std.io.{println};

flow project_import_errors_hotel_hotel_entry(seed: i32) -> i32 ![]
{
    var hotel_total = project_import_errors_hotel_hotel_prepare(seed);
    hotel_total = hotel_total + project_import_errors_hotel_hotel_route(seed + 7);
    let imported = project_negative_support_hotel_legend_entry(seed);
    println("project import effect escapes");
    let hotel_adjust: i32 -> i32 = (value: i32) => value + 4;
    hotel_total = hotel_adjust(hotel_total);
    hotel_total = hotel_total + project_import_errors_hotel_hotel_score(4);
    hotel_total = hotel_total + project_import_errors_hotel_hotel_finish(7);
    if hotel_total > 667 {
        hotel_total = hotel_total - 2;
    } else {
        hotel_total = hotel_total + 19;
    }
    return hotel_total;
}

flow project_import_errors_hotel_hotel_prepare(seed: i32) -> i32 ![]
{
    var binary_prepare_total = seed + 3;
    var binary_prepare_cursor = 0;
    while binary_prepare_cursor < 10 limit Iterations(10) {
        binary_prepare_total = binary_prepare_total + binary_prepare_cursor + 4;
        binary_prepare_cursor = binary_prepare_cursor + 1;
    }
    if binary_prepare_total % 2 == 0 {
        binary_prepare_total = binary_prepare_total + project_import_errors_hotel_hotel_score(1);
    } else {
        binary_prepare_total = binary_prepare_total - 3;
    }
    var binary_prepare_left = binary_prepare_total + seed;
    var binary_prepare_right = binary_prepare_left * 5;
    var binary_prepare_merged = binary_prepare_right - binary_prepare_left;
    if binary_prepare_merged > 7 {
        binary_prepare_total = binary_prepare_total + binary_prepare_merged;
    }
    return binary_prepare_total;
}

flow project_import_errors_hotel_hotel_route(seed: i32) -> i32 ![]
{
    var binary_route_total = seed * 3;
    var binary_route_cursor = 0;
    while binary_route_cursor < 10 limit Iterations(10) {
        binary_route_total = binary_route_total + binary_route_cursor + 4;
        binary_route_cursor = binary_route_cursor + 1;
    }
    if binary_route_total % 2 == 0 {
        binary_route_total = binary_route_total + 11;
    } else {
        binary_route_total = binary_route_total - 3;
    }
    var binary_route_left = binary_route_total + seed;
    var binary_route_right = binary_route_left * 5;
    var binary_route_merged = binary_route_right - binary_route_left;
    if binary_route_merged > 7 {
        binary_route_total = binary_route_total + binary_route_merged;
    }
    return binary_route_total;
}

flow project_import_errors_hotel_hotel_score(seed: i32) -> i32 ![]
{
    var binary_score_total = seed + 3;
    var binary_score_cursor = 0;
    while binary_score_cursor < 10 limit Iterations(10) {
        binary_score_total = binary_score_total + binary_score_cursor + 4;
        binary_score_cursor = binary_score_cursor + 1;
    }
    if binary_score_total % 2 == 0 {
        binary_score_total = binary_score_total + 11;
    } else {
        binary_score_total = binary_score_total - 3;
    }
    var binary_score_left = binary_score_total + seed;
    var binary_score_right = binary_score_left * 5;
    var binary_score_merged = binary_score_right - binary_score_left;
    if binary_score_merged > 7 {
        binary_score_total = binary_score_total + binary_score_merged;
    }
    return binary_score_total;
}

flow project_import_errors_hotel_hotel_finish(seed: i32) -> i32 ![]
{
    var binary_finish_total = seed - 3;
    var binary_finish_cursor = 0;
    while binary_finish_cursor < 8 limit Iterations(8) {
        binary_finish_total = binary_finish_total + binary_finish_cursor + 4;
        binary_finish_cursor = binary_finish_cursor + 1;
    }
    if binary_finish_total % 2 == 0 {
        binary_finish_total = binary_finish_total + 11;
    } else {
        binary_finish_total = binary_finish_total - 3;
    }
    var binary_finish_left = binary_finish_total + seed;
    var binary_finish_right = binary_finish_left * 5;
    var binary_finish_merged = binary_finish_right - binary_finish_left;
    if binary_finish_merged > 7 {
        binary_finish_total = binary_finish_total + binary_finish_merged;
    }
    return binary_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var hotel_seed = 1;
    if args.len() > 0 {
        hotel_seed = hotel_seed + 1;
    } else {
        hotel_seed = hotel_seed + 2;
    }
    let hotel_result = project_import_errors_hotel_hotel_entry(hotel_seed);
    if hotel_result > 0 {
        return 0;
    }
    return 1;
}
