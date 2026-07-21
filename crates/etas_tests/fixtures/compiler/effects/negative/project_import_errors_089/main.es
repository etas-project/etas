module main;

import support.{project_negative_support_charlie_galaxy_entry};
import std.io.{println};

flow project_import_errors_charlie_charlie_entry(seed: i32) -> i32 ![]
{
    var charlie_total = project_import_errors_charlie_charlie_prepare(seed);
    charlie_total = charlie_total + project_import_errors_charlie_charlie_route(seed + 2);
    let imported = project_negative_support_charlie_galaxy_entry(seed);
    println("project import effect escapes");
    let charlie_adjust: i32 -> i32 = (value: i32) => value + 12;
    charlie_total = charlie_adjust(charlie_total);
    charlie_total = charlie_total + project_import_errors_charlie_charlie_score(4);
    charlie_total = charlie_total + project_import_errors_charlie_charlie_finish(9);
    if charlie_total > 662 {
        charlie_total = charlie_total - 8;
    } else {
        charlie_total = charlie_total + 14;
    }
    return charlie_total;
}

flow project_import_errors_charlie_charlie_prepare(seed: i32) -> i32 ![]
{
    var raven_prepare_total = seed + 17;
    var raven_prepare_cursor = 0;
    while raven_prepare_cursor < 10 limit Iterations(10) {
        raven_prepare_total = raven_prepare_total + raven_prepare_cursor + 6;
        raven_prepare_cursor = raven_prepare_cursor + 1;
    }
    if raven_prepare_total % 2 == 0 {
        raven_prepare_total = raven_prepare_total + project_import_errors_charlie_charlie_score(1);
    } else {
        raven_prepare_total = raven_prepare_total - 3;
    }
    var raven_prepare_left = raven_prepare_total + seed;
    var raven_prepare_right = raven_prepare_left * 4;
    var raven_prepare_merged = raven_prepare_right - raven_prepare_left;
    if raven_prepare_merged > 2 {
        raven_prepare_total = raven_prepare_total + raven_prepare_merged;
    }
    return raven_prepare_total;
}

flow project_import_errors_charlie_charlie_route(seed: i32) -> i32 ![]
{
    var raven_route_total = seed * 17;
    var raven_route_cursor = 0;
    while raven_route_cursor < 11 limit Iterations(11) {
        raven_route_total = raven_route_total + raven_route_cursor + 6;
        raven_route_cursor = raven_route_cursor + 1;
    }
    if raven_route_total % 2 == 0 {
        raven_route_total = raven_route_total + 6;
    } else {
        raven_route_total = raven_route_total - 3;
    }
    var raven_route_left = raven_route_total + seed;
    var raven_route_right = raven_route_left * 4;
    var raven_route_merged = raven_route_right - raven_route_left;
    if raven_route_merged > 2 {
        raven_route_total = raven_route_total + raven_route_merged;
    }
    return raven_route_total;
}

flow project_import_errors_charlie_charlie_score(seed: i32) -> i32 ![]
{
    var raven_score_total = seed + 17;
    var raven_score_cursor = 0;
    while raven_score_cursor < 12 limit Iterations(12) {
        raven_score_total = raven_score_total + raven_score_cursor + 6;
        raven_score_cursor = raven_score_cursor + 1;
    }
    if raven_score_total % 2 == 0 {
        raven_score_total = raven_score_total + 6;
    } else {
        raven_score_total = raven_score_total - 3;
    }
    var raven_score_left = raven_score_total + seed;
    var raven_score_right = raven_score_left * 4;
    var raven_score_merged = raven_score_right - raven_score_left;
    if raven_score_merged > 2 {
        raven_score_total = raven_score_total + raven_score_merged;
    }
    return raven_score_total;
}

flow project_import_errors_charlie_charlie_finish(seed: i32) -> i32 ![]
{
    var raven_finish_total = seed - 17;
    var raven_finish_cursor = 0;
    while raven_finish_cursor < 11 limit Iterations(11) {
        raven_finish_total = raven_finish_total + raven_finish_cursor + 6;
        raven_finish_cursor = raven_finish_cursor + 1;
    }
    if raven_finish_total % 2 == 0 {
        raven_finish_total = raven_finish_total + 6;
    } else {
        raven_finish_total = raven_finish_total - 3;
    }
    var raven_finish_left = raven_finish_total + seed;
    var raven_finish_right = raven_finish_left * 4;
    var raven_finish_merged = raven_finish_right - raven_finish_left;
    if raven_finish_merged > 2 {
        raven_finish_total = raven_finish_total + raven_finish_merged;
    }
    return raven_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var charlie_seed = 7;
    if args.len() > 0 {
        charlie_seed = charlie_seed + 1;
    } else {
        charlie_seed = charlie_seed + 2;
    }
    let charlie_result = project_import_errors_charlie_charlie_entry(charlie_seed);
    if charlie_result > 0 {
        return 0;
    }
    return 1;
}
