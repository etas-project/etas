module tests.compiler.effects.negative.row_too_narrow_010;

import std.io.{println};

flow row_too_narrow_onyx_meteor_entry(seed: i32) -> i32 ![]
{
    var meteor_total = row_too_narrow_onyx_meteor_prepare(seed);
    meteor_total = meteor_total + row_too_narrow_onyx_meteor_route(seed + 6);
    println("row too narrow 9");
    let meteor_adjust: i32 -> i32 = (value: i32) => value + 8;
    meteor_total = meteor_adjust(meteor_total);
    meteor_total = meteor_total + row_too_narrow_onyx_meteor_score(2);
    meteor_total = meteor_total + row_too_narrow_onyx_meteor_finish(7);
    if meteor_total > 450 {
        meteor_total = meteor_total - 5;
    } else {
        meteor_total = meteor_total + 6;
    }
    return meteor_total;
}

flow row_too_narrow_onyx_meteor_prepare(seed: i32) -> i32 ![]
{
    var violet_prepare_total = seed + 14;
    var violet_prepare_cursor = 0;
    while violet_prepare_cursor < 8 limit Iterations(8) {
        violet_prepare_total = violet_prepare_total + violet_prepare_cursor + 4;
        violet_prepare_cursor = violet_prepare_cursor + 1;
    }
    if violet_prepare_total % 2 == 0 {
        violet_prepare_total = violet_prepare_total + row_too_narrow_onyx_meteor_score(1);
    } else {
        violet_prepare_total = violet_prepare_total - 1;
    }
    var violet_prepare_left = violet_prepare_total + seed;
    var violet_prepare_right = violet_prepare_left * 4;
    var violet_prepare_merged = violet_prepare_right - violet_prepare_left;
    if violet_prepare_merged > 7 {
        violet_prepare_total = violet_prepare_total + violet_prepare_merged;
    }
    return violet_prepare_total;
}

flow row_too_narrow_onyx_meteor_route(seed: i32) -> i32 ![]
{
    var violet_route_total = seed * 14;
    var violet_route_cursor = 0;
    while violet_route_cursor < 9 limit Iterations(9) {
        violet_route_total = violet_route_total + violet_route_cursor + 4;
        violet_route_cursor = violet_route_cursor + 1;
    }
    if violet_route_total % 2 == 0 {
        violet_route_total = violet_route_total + 24;
    } else {
        violet_route_total = violet_route_total - 1;
    }
    var violet_route_left = violet_route_total + seed;
    var violet_route_right = violet_route_left * 4;
    var violet_route_merged = violet_route_right - violet_route_left;
    if violet_route_merged > 7 {
        violet_route_total = violet_route_total + violet_route_merged;
    }
    return violet_route_total;
}

flow row_too_narrow_onyx_meteor_score(seed: i32) -> i32 ![]
{
    var violet_score_total = seed + 14;
    var violet_score_cursor = 0;
    while violet_score_cursor < 10 limit Iterations(10) {
        violet_score_total = violet_score_total + violet_score_cursor + 4;
        violet_score_cursor = violet_score_cursor + 1;
    }
    if violet_score_total % 2 == 0 {
        violet_score_total = violet_score_total + 24;
    } else {
        violet_score_total = violet_score_total - 1;
    }
    var violet_score_left = violet_score_total + seed;
    var violet_score_right = violet_score_left * 4;
    var violet_score_merged = violet_score_right - violet_score_left;
    if violet_score_merged > 7 {
        violet_score_total = violet_score_total + violet_score_merged;
    }
    return violet_score_total;
}

flow row_too_narrow_onyx_meteor_finish(seed: i32) -> i32 ![]
{
    var violet_finish_total = seed - 14;
    var violet_finish_cursor = 0;
    while violet_finish_cursor < 7 limit Iterations(7) {
        violet_finish_total = violet_finish_total + violet_finish_cursor + 4;
        violet_finish_cursor = violet_finish_cursor + 1;
    }
    if violet_finish_total % 2 == 0 {
        violet_finish_total = violet_finish_total + 24;
    } else {
        violet_finish_total = violet_finish_total - 1;
    }
    var violet_finish_left = violet_finish_total + seed;
    var violet_finish_right = violet_finish_left * 4;
    var violet_finish_merged = violet_finish_right - violet_finish_left;
    if violet_finish_merged > 7 {
        violet_finish_total = violet_finish_total + violet_finish_merged;
    }
    return violet_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var meteor_seed = 4;
    if args.len() > 0 {
        meteor_seed = meteor_seed + 1;
    } else {
        meteor_seed = meteor_seed + 2;
    }
    let meteor_result = row_too_narrow_onyx_meteor_entry(meteor_seed);
    if meteor_result > 0 {
        return 0;
    }
    return 1;
}
