module main;

import support.{project_negative_support_echo_isotope_entry};
import std.io.{println};

flow project_import_errors_echo_echo_entry(seed: i32) -> i32 ![]
{
    var echo_total = project_import_errors_echo_echo_prepare(seed);
    echo_total = echo_total + project_import_errors_echo_echo_route(seed + 4);
    let imported = project_negative_support_echo_isotope_entry(seed);
    println("project import effect escapes");
    let echo_adjust: i32 -> i32 = (value: i32) => value + 1;
    echo_total = echo_adjust(echo_total);
    echo_total = echo_total + project_import_errors_echo_echo_score(6);
    echo_total = echo_total + project_import_errors_echo_echo_finish(4);
    if echo_total > 664 {
        echo_total = echo_total - 10;
    } else {
        echo_total = echo_total + 16;
    }
    return echo_total;
}

flow project_import_errors_echo_echo_prepare(seed: i32) -> i32 ![]
{
    var fable_prepare_total = seed + 19;
    var fable_prepare_cursor = 0;
    while fable_prepare_cursor < 12 limit Iterations(12) {
        fable_prepare_total = fable_prepare_total + fable_prepare_cursor + 1;
        fable_prepare_cursor = fable_prepare_cursor + 1;
    }
    if fable_prepare_total % 2 == 0 {
        fable_prepare_total = fable_prepare_total + project_import_errors_echo_echo_score(1);
    } else {
        fable_prepare_total = fable_prepare_total - 5;
    }
    var fable_prepare_left = fable_prepare_total + seed;
    var fable_prepare_right = fable_prepare_left * 2;
    var fable_prepare_merged = fable_prepare_right - fable_prepare_left;
    if fable_prepare_merged > 4 {
        fable_prepare_total = fable_prepare_total + fable_prepare_merged;
    }
    return fable_prepare_total;
}

flow project_import_errors_echo_echo_route(seed: i32) -> i32 ![]
{
    var fable_route_total = seed * 19;
    var fable_route_cursor = 0;
    while fable_route_cursor < 7 limit Iterations(7) {
        fable_route_total = fable_route_total + fable_route_cursor + 1;
        fable_route_cursor = fable_route_cursor + 1;
    }
    if fable_route_total % 2 == 0 {
        fable_route_total = fable_route_total + 8;
    } else {
        fable_route_total = fable_route_total - 5;
    }
    var fable_route_left = fable_route_total + seed;
    var fable_route_right = fable_route_left * 2;
    var fable_route_merged = fable_route_right - fable_route_left;
    if fable_route_merged > 4 {
        fable_route_total = fable_route_total + fable_route_merged;
    }
    return fable_route_total;
}

flow project_import_errors_echo_echo_score(seed: i32) -> i32 ![]
{
    var fable_score_total = seed + 19;
    var fable_score_cursor = 0;
    while fable_score_cursor < 7 limit Iterations(7) {
        fable_score_total = fable_score_total + fable_score_cursor + 1;
        fable_score_cursor = fable_score_cursor + 1;
    }
    if fable_score_total % 2 == 0 {
        fable_score_total = fable_score_total + 8;
    } else {
        fable_score_total = fable_score_total - 5;
    }
    var fable_score_left = fable_score_total + seed;
    var fable_score_right = fable_score_left * 2;
    var fable_score_merged = fable_score_right - fable_score_left;
    if fable_score_merged > 4 {
        fable_score_total = fable_score_total + fable_score_merged;
    }
    return fable_score_total;
}

flow project_import_errors_echo_echo_finish(seed: i32) -> i32 ![]
{
    var fable_finish_total = seed - 19;
    var fable_finish_cursor = 0;
    while fable_finish_cursor < 5 limit Iterations(5) {
        fable_finish_total = fable_finish_total + fable_finish_cursor + 1;
        fable_finish_cursor = fable_finish_cursor + 1;
    }
    if fable_finish_total % 2 == 0 {
        fable_finish_total = fable_finish_total + 8;
    } else {
        fable_finish_total = fable_finish_total - 5;
    }
    var fable_finish_left = fable_finish_total + seed;
    var fable_finish_right = fable_finish_left * 2;
    var fable_finish_merged = fable_finish_right - fable_finish_left;
    if fable_finish_merged > 4 {
        fable_finish_total = fable_finish_total + fable_finish_merged;
    }
    return fable_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var echo_seed = 9;
    if args.len() > 0 {
        echo_seed = echo_seed + 1;
    } else {
        echo_seed = echo_seed + 2;
    }
    let echo_result = project_import_errors_echo_echo_entry(echo_seed);
    if echo_result > 0 {
        return 0;
    }
    return 1;
}
