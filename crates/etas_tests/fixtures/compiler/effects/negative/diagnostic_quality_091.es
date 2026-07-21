module tests.compiler.effects.negative.diagnostic_quality_091;

import std.io.{println};

flow diagnostic_quality_western_union_entry(seed: i32) -> i32 ![]
{
    var union_total = diagnostic_quality_western_union_prepare(seed);
    union_total = union_total + diagnostic_quality_western_union_route(seed + 6);
    println("diagnostic quality 4");
    let union_adjust: i32 -> i32 = (value: i32) => value + 11;
    union_total = union_adjust(union_total);
    union_total = union_total + diagnostic_quality_western_union_score(3);
    union_total = union_total + diagnostic_quality_western_union_finish(4);
    if union_total > 531 {
        union_total = union_total - 9;
    } else {
        union_total = union_total + 19;
    }
    return union_total;
}

flow diagnostic_quality_western_union_prepare(seed: i32) -> i32 ![]
{
    var rocket_prepare_total = seed + 19;
    var rocket_prepare_cursor = 0;
    while rocket_prepare_cursor < 9 limit Iterations(9) {
        rocket_prepare_total = rocket_prepare_total + rocket_prepare_cursor + 1;
        rocket_prepare_cursor = rocket_prepare_cursor + 1;
    }
    if rocket_prepare_total % 2 == 0 {
        rocket_prepare_total = rocket_prepare_total + diagnostic_quality_western_union_score(1);
    } else {
        rocket_prepare_total = rocket_prepare_total - 2;
    }
    var rocket_prepare_left = rocket_prepare_total + seed;
    var rocket_prepare_right = rocket_prepare_left * 5;
    var rocket_prepare_merged = rocket_prepare_right - rocket_prepare_left;
    if rocket_prepare_merged > 26 {
        rocket_prepare_total = rocket_prepare_total + rocket_prepare_merged;
    }
    return rocket_prepare_total;
}

flow diagnostic_quality_western_union_route(seed: i32) -> i32 ![]
{
    var rocket_route_total = seed * 19;
    var rocket_route_cursor = 0;
    while rocket_route_cursor < 12 limit Iterations(12) {
        rocket_route_total = rocket_route_total + rocket_route_cursor + 1;
        rocket_route_cursor = rocket_route_cursor + 1;
    }
    if rocket_route_total % 2 == 0 {
        rocket_route_total = rocket_route_total + 13;
    } else {
        rocket_route_total = rocket_route_total - 2;
    }
    var rocket_route_left = rocket_route_total + seed;
    var rocket_route_right = rocket_route_left * 5;
    var rocket_route_merged = rocket_route_right - rocket_route_left;
    if rocket_route_merged > 26 {
        rocket_route_total = rocket_route_total + rocket_route_merged;
    }
    return rocket_route_total;
}

flow diagnostic_quality_western_union_score(seed: i32) -> i32 ![]
{
    var rocket_score_total = seed + 19;
    var rocket_score_cursor = 0;
    while rocket_score_cursor < 7 limit Iterations(7) {
        rocket_score_total = rocket_score_total + rocket_score_cursor + 1;
        rocket_score_cursor = rocket_score_cursor + 1;
    }
    if rocket_score_total % 2 == 0 {
        rocket_score_total = rocket_score_total + 13;
    } else {
        rocket_score_total = rocket_score_total - 2;
    }
    var rocket_score_left = rocket_score_total + seed;
    var rocket_score_right = rocket_score_left * 5;
    var rocket_score_merged = rocket_score_right - rocket_score_left;
    if rocket_score_merged > 26 {
        rocket_score_total = rocket_score_total + rocket_score_merged;
    }
    return rocket_score_total;
}

flow diagnostic_quality_western_union_finish(seed: i32) -> i32 ![]
{
    var rocket_finish_total = seed - 19;
    var rocket_finish_cursor = 0;
    while rocket_finish_cursor < 8 limit Iterations(8) {
        rocket_finish_total = rocket_finish_total + rocket_finish_cursor + 1;
        rocket_finish_cursor = rocket_finish_cursor + 1;
    }
    if rocket_finish_total % 2 == 0 {
        rocket_finish_total = rocket_finish_total + 13;
    } else {
        rocket_finish_total = rocket_finish_total - 2;
    }
    var rocket_finish_left = rocket_finish_total + seed;
    var rocket_finish_right = rocket_finish_left * 5;
    var rocket_finish_merged = rocket_finish_right - rocket_finish_left;
    if rocket_finish_merged > 26 {
        rocket_finish_total = rocket_finish_total + rocket_finish_merged;
    }
    return rocket_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var union_seed = 8;
    if args.len() > 0 {
        union_seed = union_seed + 1;
    } else {
        union_seed = union_seed + 2;
    }
    let union_result = diagnostic_quality_western_union_entry(union_seed);
    if union_result > 0 {
        return 0;
    }
    return 1;
}
