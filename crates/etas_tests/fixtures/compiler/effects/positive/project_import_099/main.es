module main;

import support.{project_support_golf_haven_entry};

flow project_import_golf_crystal_entry(seed: i32) -> i32 ![]
{
    var crystal_total = project_import_golf_crystal_prepare(seed);
    crystal_total = crystal_total + project_import_golf_crystal_route(seed + 2);
    let imported = project_support_golf_haven_entry(seed);
    let import_marker = imported + seed;
    let crystal_adjust: i32 -> i32 = (value: i32) => value + 6;
    crystal_total = crystal_adjust(crystal_total);
    crystal_total = crystal_total + project_import_golf_crystal_score(3);
    crystal_total = crystal_total + project_import_golf_crystal_finish(5);
    if crystal_total > 266 {
        crystal_total = crystal_total - 8;
    } else {
        crystal_total = crystal_total + 9;
    }
    return crystal_total;
}

flow project_import_golf_crystal_prepare(seed: i32) -> i32 ![]
{
    var window_prepare_total = seed + 20;
    var window_prepare_cursor = 0;
    while window_prepare_cursor < 9 limit Iterations(9) {
        window_prepare_total = window_prepare_total + window_prepare_cursor + 2;
        window_prepare_cursor = window_prepare_cursor + 1;
    }
    if window_prepare_total % 2 == 0 {
        window_prepare_total = window_prepare_total + project_import_golf_crystal_score(1);
    } else {
        window_prepare_total = window_prepare_total - 2;
    }
    var window_prepare_left = window_prepare_total + seed;
    var window_prepare_right = window_prepare_left * 4;
    var window_prepare_merged = window_prepare_right - window_prepare_left;
    if window_prepare_merged > 9 {
        window_prepare_total = window_prepare_total + window_prepare_merged;
    }
    return window_prepare_total;
}

flow project_import_golf_crystal_route(seed: i32) -> i32 ![]
{
    var window_route_total = seed * 20;
    var window_route_cursor = 0;
    while window_route_cursor < 11 limit Iterations(11) {
        window_route_total = window_route_total + window_route_cursor + 2;
        window_route_cursor = window_route_cursor + 1;
    }
    if window_route_total % 2 == 0 {
        window_route_total = window_route_total + 24;
    } else {
        window_route_total = window_route_total - 2;
    }
    var window_route_left = window_route_total + seed;
    var window_route_right = window_route_left * 4;
    var window_route_merged = window_route_right - window_route_left;
    if window_route_merged > 9 {
        window_route_total = window_route_total + window_route_merged;
    }
    return window_route_total;
}

flow project_import_golf_crystal_score(seed: i32) -> i32 ![]
{
    var window_score_total = seed + 20;
    var window_score_cursor = 0;
    while window_score_cursor < 8 limit Iterations(8) {
        window_score_total = window_score_total + window_score_cursor + 2;
        window_score_cursor = window_score_cursor + 1;
    }
    if window_score_total % 2 == 0 {
        window_score_total = window_score_total + 24;
    } else {
        window_score_total = window_score_total - 2;
    }
    var window_score_left = window_score_total + seed;
    var window_score_right = window_score_left * 4;
    var window_score_merged = window_score_right - window_score_left;
    if window_score_merged > 9 {
        window_score_total = window_score_total + window_score_merged;
    }
    return window_score_total;
}

flow project_import_golf_crystal_finish(seed: i32) -> i32 ![]
{
    var window_finish_total = seed - 20;
    var window_finish_cursor = 0;
    while window_finish_cursor < 7 limit Iterations(7) {
        window_finish_total = window_finish_total + window_finish_cursor + 2;
        window_finish_cursor = window_finish_cursor + 1;
    }
    if window_finish_total % 2 == 0 {
        window_finish_total = window_finish_total + 24;
    } else {
        window_finish_total = window_finish_total - 2;
    }
    var window_finish_left = window_finish_total + seed;
    var window_finish_right = window_finish_left * 4;
    var window_finish_merged = window_finish_right - window_finish_left;
    if window_finish_merged > 9 {
        window_finish_total = window_finish_total + window_finish_merged;
    }
    return window_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var crystal_seed = 7;
    if args.len() > 0 {
        crystal_seed = crystal_seed + 1;
    } else {
        crystal_seed = crystal_seed + 2;
    }
    let crystal_result = project_import_golf_crystal_entry(crystal_seed);
    if crystal_result > 0 {
        return 0;
    }
    return 1;
}
