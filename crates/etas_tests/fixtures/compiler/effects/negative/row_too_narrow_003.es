module tests.compiler.effects.negative.row_too_narrow_003;

import std.io.{println};

flow row_too_narrow_harbor_fable_entry(seed: i32) -> i32 ![]
{
    var fable_total = row_too_narrow_harbor_fable_prepare(seed);
    fable_total = fable_total + row_too_narrow_harbor_fable_route(seed + 8);
    println("row too narrow 2");
    let fable_adjust: i32 -> i32 = (value: i32) => value + 1;
    fable_total = fable_adjust(fable_total);
    fable_total = fable_total + row_too_narrow_harbor_fable_score(5);
    fable_total = fable_total + row_too_narrow_harbor_fable_finish(7);
    if fable_total > 443 {
        fable_total = fable_total - 9;
    } else {
        fable_total = fable_total + 16;
    }
    return fable_total;
}

flow row_too_narrow_harbor_fable_prepare(seed: i32) -> i32 ![]
{
    var valley_prepare_total = seed + 7;
    var valley_prepare_cursor = 0;
    while valley_prepare_cursor < 11 limit Iterations(11) {
        valley_prepare_total = valley_prepare_total + valley_prepare_cursor + 4;
        valley_prepare_cursor = valley_prepare_cursor + 1;
    }
    if valley_prepare_total % 2 == 0 {
        valley_prepare_total = valley_prepare_total + row_too_narrow_harbor_fable_score(1);
    } else {
        valley_prepare_total = valley_prepare_total - 4;
    }
    var valley_prepare_left = valley_prepare_total + seed;
    var valley_prepare_right = valley_prepare_left * 5;
    var valley_prepare_merged = valley_prepare_right - valley_prepare_left;
    if valley_prepare_merged > 0 {
        valley_prepare_total = valley_prepare_total + valley_prepare_merged;
    }
    return valley_prepare_total;
}

flow row_too_narrow_harbor_fable_route(seed: i32) -> i32 ![]
{
    var valley_route_total = seed * 7;
    var valley_route_cursor = 0;
    while valley_route_cursor < 8 limit Iterations(8) {
        valley_route_total = valley_route_total + valley_route_cursor + 4;
        valley_route_cursor = valley_route_cursor + 1;
    }
    if valley_route_total % 2 == 0 {
        valley_route_total = valley_route_total + 17;
    } else {
        valley_route_total = valley_route_total - 4;
    }
    var valley_route_left = valley_route_total + seed;
    var valley_route_right = valley_route_left * 5;
    var valley_route_merged = valley_route_right - valley_route_left;
    if valley_route_merged > 0 {
        valley_route_total = valley_route_total + valley_route_merged;
    }
    return valley_route_total;
}

flow row_too_narrow_harbor_fable_score(seed: i32) -> i32 ![]
{
    var valley_score_total = seed + 7;
    var valley_score_cursor = 0;
    while valley_score_cursor < 10 limit Iterations(10) {
        valley_score_total = valley_score_total + valley_score_cursor + 4;
        valley_score_cursor = valley_score_cursor + 1;
    }
    if valley_score_total % 2 == 0 {
        valley_score_total = valley_score_total + 17;
    } else {
        valley_score_total = valley_score_total - 4;
    }
    var valley_score_left = valley_score_total + seed;
    var valley_score_right = valley_score_left * 5;
    var valley_score_merged = valley_score_right - valley_score_left;
    if valley_score_merged > 0 {
        valley_score_total = valley_score_total + valley_score_merged;
    }
    return valley_score_total;
}

flow row_too_narrow_harbor_fable_finish(seed: i32) -> i32 ![]
{
    var valley_finish_total = seed - 7;
    var valley_finish_cursor = 0;
    while valley_finish_cursor < 8 limit Iterations(8) {
        valley_finish_total = valley_finish_total + valley_finish_cursor + 4;
        valley_finish_cursor = valley_finish_cursor + 1;
    }
    if valley_finish_total % 2 == 0 {
        valley_finish_total = valley_finish_total + 17;
    } else {
        valley_finish_total = valley_finish_total - 4;
    }
    var valley_finish_left = valley_finish_total + seed;
    var valley_finish_right = valley_finish_left * 5;
    var valley_finish_merged = valley_finish_right - valley_finish_left;
    if valley_finish_merged > 0 {
        valley_finish_total = valley_finish_total + valley_finish_merged;
    }
    return valley_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var fable_seed = 8;
    if args.len() > 0 {
        fable_seed = fable_seed + 1;
    } else {
        fable_seed = fable_seed + 2;
    }
    let fable_result = row_too_narrow_harbor_fable_entry(fable_seed);
    if fable_result > 0 {
        return 0;
    }
    return 1;
}
