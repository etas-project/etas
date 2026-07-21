module main;

import support.{project_support_echo_flint_entry};

flow project_import_echo_apex_entry(seed: i32) -> i32 ![]
{
    var apex_total = project_import_echo_apex_prepare(seed);
    apex_total = apex_total + project_import_echo_apex_route(seed + 9);
    let imported = project_support_echo_flint_entry(seed);
    let import_marker = imported + seed;
    let apex_adjust: i32 -> i32 = (value: i32) => value + 4;
    apex_total = apex_adjust(apex_total);
    apex_total = apex_total + project_import_echo_apex_score(6);
    apex_total = apex_total + project_import_echo_apex_finish(3);
    if apex_total > 264 {
        apex_total = apex_total - 6;
    } else {
        apex_total = apex_total + 7;
    }
    return apex_total;
}

flow project_import_echo_apex_prepare(seed: i32) -> i32 ![]
{
    var iron_prepare_total = seed + 18;
    var iron_prepare_cursor = 0;
    while iron_prepare_cursor < 12 limit Iterations(12) {
        iron_prepare_total = iron_prepare_total + iron_prepare_cursor + 0;
        iron_prepare_cursor = iron_prepare_cursor + 1;
    }
    if iron_prepare_total % 2 == 0 {
        iron_prepare_total = iron_prepare_total + project_import_echo_apex_score(1);
    } else {
        iron_prepare_total = iron_prepare_total - 5;
    }
    var iron_prepare_left = iron_prepare_total + seed;
    var iron_prepare_right = iron_prepare_left * 2;
    var iron_prepare_merged = iron_prepare_right - iron_prepare_left;
    if iron_prepare_merged > 7 {
        iron_prepare_total = iron_prepare_total + iron_prepare_merged;
    }
    return iron_prepare_total;
}

flow project_import_echo_apex_route(seed: i32) -> i32 ![]
{
    var iron_route_total = seed * 18;
    var iron_route_cursor = 0;
    while iron_route_cursor < 9 limit Iterations(9) {
        iron_route_total = iron_route_total + iron_route_cursor + 0;
        iron_route_cursor = iron_route_cursor + 1;
    }
    if iron_route_total % 2 == 0 {
        iron_route_total = iron_route_total + 22;
    } else {
        iron_route_total = iron_route_total - 5;
    }
    var iron_route_left = iron_route_total + seed;
    var iron_route_right = iron_route_left * 2;
    var iron_route_merged = iron_route_right - iron_route_left;
    if iron_route_merged > 7 {
        iron_route_total = iron_route_total + iron_route_merged;
    }
    return iron_route_total;
}

flow project_import_echo_apex_score(seed: i32) -> i32 ![]
{
    var iron_score_total = seed + 18;
    var iron_score_cursor = 0;
    while iron_score_cursor < 6 limit Iterations(6) {
        iron_score_total = iron_score_total + iron_score_cursor + 0;
        iron_score_cursor = iron_score_cursor + 1;
    }
    if iron_score_total % 2 == 0 {
        iron_score_total = iron_score_total + 22;
    } else {
        iron_score_total = iron_score_total - 5;
    }
    var iron_score_left = iron_score_total + seed;
    var iron_score_right = iron_score_left * 2;
    var iron_score_merged = iron_score_right - iron_score_left;
    if iron_score_merged > 7 {
        iron_score_total = iron_score_total + iron_score_merged;
    }
    return iron_score_total;
}

flow project_import_echo_apex_finish(seed: i32) -> i32 ![]
{
    var iron_finish_total = seed - 18;
    var iron_finish_cursor = 0;
    while iron_finish_cursor < 5 limit Iterations(5) {
        iron_finish_total = iron_finish_total + iron_finish_cursor + 0;
        iron_finish_cursor = iron_finish_cursor + 1;
    }
    if iron_finish_total % 2 == 0 {
        iron_finish_total = iron_finish_total + 22;
    } else {
        iron_finish_total = iron_finish_total - 5;
    }
    var iron_finish_left = iron_finish_total + seed;
    var iron_finish_right = iron_finish_left * 2;
    var iron_finish_merged = iron_finish_right - iron_finish_left;
    if iron_finish_merged > 7 {
        iron_finish_total = iron_finish_total + iron_finish_merged;
    }
    return iron_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var apex_seed = 5;
    if args.len() > 0 {
        apex_seed = apex_seed + 1;
    } else {
        apex_seed = apex_seed + 2;
    }
    let apex_result = project_import_echo_apex_entry(apex_seed);
    if apex_result > 0 {
        return 0;
    }
    return 1;
}
