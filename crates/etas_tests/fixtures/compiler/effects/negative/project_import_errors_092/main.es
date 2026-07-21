module main;

import support.{project_negative_support_foxtrot_junction_entry};
import std.io.{println};

flow project_import_errors_foxtrot_foxtrot_entry(seed: i32) -> i32 ![]
{
    var foxtrot_total = project_import_errors_foxtrot_foxtrot_prepare(seed);
    foxtrot_total = foxtrot_total + project_import_errors_foxtrot_foxtrot_route(seed + 5);
    let imported = project_negative_support_foxtrot_junction_entry(seed);
    println("project import effect escapes");
    let foxtrot_adjust: i32 -> i32 = (value: i32) => value + 2;
    foxtrot_total = foxtrot_adjust(foxtrot_total);
    foxtrot_total = foxtrot_total + project_import_errors_foxtrot_foxtrot_score(2);
    foxtrot_total = foxtrot_total + project_import_errors_foxtrot_foxtrot_finish(5);
    if foxtrot_total > 665 {
        foxtrot_total = foxtrot_total - 11;
    } else {
        foxtrot_total = foxtrot_total + 17;
    }
    return foxtrot_total;
}

flow project_import_errors_foxtrot_foxtrot_prepare(seed: i32) -> i32 ![]
{
    var meteor_prepare_total = seed + 20;
    var meteor_prepare_cursor = 0;
    while meteor_prepare_cursor < 8 limit Iterations(8) {
        meteor_prepare_total = meteor_prepare_total + meteor_prepare_cursor + 2;
        meteor_prepare_cursor = meteor_prepare_cursor + 1;
    }
    if meteor_prepare_total % 2 == 0 {
        meteor_prepare_total = meteor_prepare_total + project_import_errors_foxtrot_foxtrot_score(1);
    } else {
        meteor_prepare_total = meteor_prepare_total - 1;
    }
    var meteor_prepare_left = meteor_prepare_total + seed;
    var meteor_prepare_right = meteor_prepare_left * 3;
    var meteor_prepare_merged = meteor_prepare_right - meteor_prepare_left;
    if meteor_prepare_merged > 5 {
        meteor_prepare_total = meteor_prepare_total + meteor_prepare_merged;
    }
    return meteor_prepare_total;
}

flow project_import_errors_foxtrot_foxtrot_route(seed: i32) -> i32 ![]
{
    var meteor_route_total = seed * 20;
    var meteor_route_cursor = 0;
    while meteor_route_cursor < 8 limit Iterations(8) {
        meteor_route_total = meteor_route_total + meteor_route_cursor + 2;
        meteor_route_cursor = meteor_route_cursor + 1;
    }
    if meteor_route_total % 2 == 0 {
        meteor_route_total = meteor_route_total + 9;
    } else {
        meteor_route_total = meteor_route_total - 1;
    }
    var meteor_route_left = meteor_route_total + seed;
    var meteor_route_right = meteor_route_left * 3;
    var meteor_route_merged = meteor_route_right - meteor_route_left;
    if meteor_route_merged > 5 {
        meteor_route_total = meteor_route_total + meteor_route_merged;
    }
    return meteor_route_total;
}

flow project_import_errors_foxtrot_foxtrot_score(seed: i32) -> i32 ![]
{
    var meteor_score_total = seed + 20;
    var meteor_score_cursor = 0;
    while meteor_score_cursor < 8 limit Iterations(8) {
        meteor_score_total = meteor_score_total + meteor_score_cursor + 2;
        meteor_score_cursor = meteor_score_cursor + 1;
    }
    if meteor_score_total % 2 == 0 {
        meteor_score_total = meteor_score_total + 9;
    } else {
        meteor_score_total = meteor_score_total - 1;
    }
    var meteor_score_left = meteor_score_total + seed;
    var meteor_score_right = meteor_score_left * 3;
    var meteor_score_merged = meteor_score_right - meteor_score_left;
    if meteor_score_merged > 5 {
        meteor_score_total = meteor_score_total + meteor_score_merged;
    }
    return meteor_score_total;
}

flow project_import_errors_foxtrot_foxtrot_finish(seed: i32) -> i32 ![]
{
    var meteor_finish_total = seed - 20;
    var meteor_finish_cursor = 0;
    while meteor_finish_cursor < 6 limit Iterations(6) {
        meteor_finish_total = meteor_finish_total + meteor_finish_cursor + 2;
        meteor_finish_cursor = meteor_finish_cursor + 1;
    }
    if meteor_finish_total % 2 == 0 {
        meteor_finish_total = meteor_finish_total + 9;
    } else {
        meteor_finish_total = meteor_finish_total - 1;
    }
    var meteor_finish_left = meteor_finish_total + seed;
    var meteor_finish_right = meteor_finish_left * 3;
    var meteor_finish_merged = meteor_finish_right - meteor_finish_left;
    if meteor_finish_merged > 5 {
        meteor_finish_total = meteor_finish_total + meteor_finish_merged;
    }
    return meteor_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var foxtrot_seed = 10;
    if args.len() > 0 {
        foxtrot_seed = foxtrot_seed + 1;
    } else {
        foxtrot_seed = foxtrot_seed + 2;
    }
    let foxtrot_result = project_import_errors_foxtrot_foxtrot_entry(foxtrot_seed);
    if foxtrot_result > 0 {
        return 0;
    }
    return 1;
}
