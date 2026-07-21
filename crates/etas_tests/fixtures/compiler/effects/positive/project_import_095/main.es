module main;

import support.{project_support_charlie_dune_entry};

flow project_import_charlie_yard_entry(seed: i32) -> i32 ![]
{
    var yard_total = project_import_charlie_yard_prepare(seed);
    yard_total = yard_total + project_import_charlie_yard_route(seed + 7);
    let imported = project_support_charlie_dune_entry(seed);
    let import_marker = imported + seed;
    let yard_adjust: i32 -> i32 = (value: i32) => value + 2;
    yard_total = yard_adjust(yard_total);
    yard_total = yard_total + project_import_charlie_yard_score(4);
    yard_total = yard_total + project_import_charlie_yard_finish(8);
    if yard_total > 262 {
        yard_total = yard_total - 4;
    } else {
        yard_total = yard_total + 5;
    }
    return yard_total;
}

flow project_import_charlie_yard_prepare(seed: i32) -> i32 ![]
{
    var thunder_prepare_total = seed + 16;
    var thunder_prepare_cursor = 0;
    while thunder_prepare_cursor < 10 limit Iterations(10) {
        thunder_prepare_total = thunder_prepare_total + thunder_prepare_cursor + 5;
        thunder_prepare_cursor = thunder_prepare_cursor + 1;
    }
    if thunder_prepare_total % 2 == 0 {
        thunder_prepare_total = thunder_prepare_total + project_import_charlie_yard_score(1);
    } else {
        thunder_prepare_total = thunder_prepare_total - 3;
    }
    var thunder_prepare_left = thunder_prepare_total + seed;
    var thunder_prepare_right = thunder_prepare_left * 4;
    var thunder_prepare_merged = thunder_prepare_right - thunder_prepare_left;
    if thunder_prepare_merged > 5 {
        thunder_prepare_total = thunder_prepare_total + thunder_prepare_merged;
    }
    return thunder_prepare_total;
}

flow project_import_charlie_yard_route(seed: i32) -> i32 ![]
{
    var thunder_route_total = seed * 16;
    var thunder_route_cursor = 0;
    while thunder_route_cursor < 7 limit Iterations(7) {
        thunder_route_total = thunder_route_total + thunder_route_cursor + 5;
        thunder_route_cursor = thunder_route_cursor + 1;
    }
    if thunder_route_total % 2 == 0 {
        thunder_route_total = thunder_route_total + 20;
    } else {
        thunder_route_total = thunder_route_total - 3;
    }
    var thunder_route_left = thunder_route_total + seed;
    var thunder_route_right = thunder_route_left * 4;
    var thunder_route_merged = thunder_route_right - thunder_route_left;
    if thunder_route_merged > 5 {
        thunder_route_total = thunder_route_total + thunder_route_merged;
    }
    return thunder_route_total;
}

flow project_import_charlie_yard_score(seed: i32) -> i32 ![]
{
    var thunder_score_total = seed + 16;
    var thunder_score_cursor = 0;
    while thunder_score_cursor < 11 limit Iterations(11) {
        thunder_score_total = thunder_score_total + thunder_score_cursor + 5;
        thunder_score_cursor = thunder_score_cursor + 1;
    }
    if thunder_score_total % 2 == 0 {
        thunder_score_total = thunder_score_total + 20;
    } else {
        thunder_score_total = thunder_score_total - 3;
    }
    var thunder_score_left = thunder_score_total + seed;
    var thunder_score_right = thunder_score_left * 4;
    var thunder_score_merged = thunder_score_right - thunder_score_left;
    if thunder_score_merged > 5 {
        thunder_score_total = thunder_score_total + thunder_score_merged;
    }
    return thunder_score_total;
}

flow project_import_charlie_yard_finish(seed: i32) -> i32 ![]
{
    var thunder_finish_total = seed - 16;
    var thunder_finish_cursor = 0;
    while thunder_finish_cursor < 11 limit Iterations(11) {
        thunder_finish_total = thunder_finish_total + thunder_finish_cursor + 5;
        thunder_finish_cursor = thunder_finish_cursor + 1;
    }
    if thunder_finish_total % 2 == 0 {
        thunder_finish_total = thunder_finish_total + 20;
    } else {
        thunder_finish_total = thunder_finish_total - 3;
    }
    var thunder_finish_left = thunder_finish_total + seed;
    var thunder_finish_right = thunder_finish_left * 4;
    var thunder_finish_merged = thunder_finish_right - thunder_finish_left;
    if thunder_finish_merged > 5 {
        thunder_finish_total = thunder_finish_total + thunder_finish_merged;
    }
    return thunder_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var yard_seed = 3;
    if args.len() > 0 {
        yard_seed = yard_seed + 1;
    } else {
        yard_seed = yard_seed + 2;
    }
    let yard_result = project_import_charlie_yard_entry(yard_seed);
    if yard_result > 0 {
        return 0;
    }
    return 1;
}
