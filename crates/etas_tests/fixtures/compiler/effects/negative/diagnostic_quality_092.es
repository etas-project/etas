module tests.compiler.effects.negative.diagnostic_quality_092;

import std.io.{println};

flow diagnostic_quality_yellow_voyage_entry(seed: i32) -> i32 ![]
{
    var voyage_total = diagnostic_quality_yellow_voyage_prepare(seed);
    voyage_total = voyage_total + diagnostic_quality_yellow_voyage_route(seed + 7);
    println("diagnostic quality 5");
    let voyage_adjust: i32 -> i32 = (value: i32) => value + 12;
    voyage_total = voyage_adjust(voyage_total);
    voyage_total = voyage_total + diagnostic_quality_yellow_voyage_score(4);
    voyage_total = voyage_total + diagnostic_quality_yellow_voyage_finish(5);
    if voyage_total > 532 {
        voyage_total = voyage_total - 10;
    } else {
        voyage_total = voyage_total + 20;
    }
    return voyage_total;
}

flow diagnostic_quality_yellow_voyage_prepare(seed: i32) -> i32 ![]
{
    var zodiac_prepare_total = seed + 20;
    var zodiac_prepare_cursor = 0;
    while zodiac_prepare_cursor < 10 limit Iterations(10) {
        zodiac_prepare_total = zodiac_prepare_total + zodiac_prepare_cursor + 2;
        zodiac_prepare_cursor = zodiac_prepare_cursor + 1;
    }
    if zodiac_prepare_total % 2 == 0 {
        zodiac_prepare_total = zodiac_prepare_total + diagnostic_quality_yellow_voyage_score(1);
    } else {
        zodiac_prepare_total = zodiac_prepare_total - 3;
    }
    var zodiac_prepare_left = zodiac_prepare_total + seed;
    var zodiac_prepare_right = zodiac_prepare_left * 2;
    var zodiac_prepare_merged = zodiac_prepare_right - zodiac_prepare_left;
    if zodiac_prepare_merged > 27 {
        zodiac_prepare_total = zodiac_prepare_total + zodiac_prepare_merged;
    }
    return zodiac_prepare_total;
}

flow diagnostic_quality_yellow_voyage_route(seed: i32) -> i32 ![]
{
    var zodiac_route_total = seed * 20;
    var zodiac_route_cursor = 0;
    while zodiac_route_cursor < 7 limit Iterations(7) {
        zodiac_route_total = zodiac_route_total + zodiac_route_cursor + 2;
        zodiac_route_cursor = zodiac_route_cursor + 1;
    }
    if zodiac_route_total % 2 == 0 {
        zodiac_route_total = zodiac_route_total + 14;
    } else {
        zodiac_route_total = zodiac_route_total - 3;
    }
    var zodiac_route_left = zodiac_route_total + seed;
    var zodiac_route_right = zodiac_route_left * 2;
    var zodiac_route_merged = zodiac_route_right - zodiac_route_left;
    if zodiac_route_merged > 27 {
        zodiac_route_total = zodiac_route_total + zodiac_route_merged;
    }
    return zodiac_route_total;
}

flow diagnostic_quality_yellow_voyage_score(seed: i32) -> i32 ![]
{
    var zodiac_score_total = seed + 20;
    var zodiac_score_cursor = 0;
    while zodiac_score_cursor < 8 limit Iterations(8) {
        zodiac_score_total = zodiac_score_total + zodiac_score_cursor + 2;
        zodiac_score_cursor = zodiac_score_cursor + 1;
    }
    if zodiac_score_total % 2 == 0 {
        zodiac_score_total = zodiac_score_total + 14;
    } else {
        zodiac_score_total = zodiac_score_total - 3;
    }
    var zodiac_score_left = zodiac_score_total + seed;
    var zodiac_score_right = zodiac_score_left * 2;
    var zodiac_score_merged = zodiac_score_right - zodiac_score_left;
    if zodiac_score_merged > 27 {
        zodiac_score_total = zodiac_score_total + zodiac_score_merged;
    }
    return zodiac_score_total;
}

flow diagnostic_quality_yellow_voyage_finish(seed: i32) -> i32 ![]
{
    var zodiac_finish_total = seed - 20;
    var zodiac_finish_cursor = 0;
    while zodiac_finish_cursor < 9 limit Iterations(9) {
        zodiac_finish_total = zodiac_finish_total + zodiac_finish_cursor + 2;
        zodiac_finish_cursor = zodiac_finish_cursor + 1;
    }
    if zodiac_finish_total % 2 == 0 {
        zodiac_finish_total = zodiac_finish_total + 14;
    } else {
        zodiac_finish_total = zodiac_finish_total - 3;
    }
    var zodiac_finish_left = zodiac_finish_total + seed;
    var zodiac_finish_right = zodiac_finish_left * 2;
    var zodiac_finish_merged = zodiac_finish_right - zodiac_finish_left;
    if zodiac_finish_merged > 27 {
        zodiac_finish_total = zodiac_finish_total + zodiac_finish_merged;
    }
    return zodiac_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var voyage_seed = 9;
    if args.len() > 0 {
        voyage_seed = voyage_seed + 1;
    } else {
        voyage_seed = voyage_seed + 2;
    }
    let voyage_result = diagnostic_quality_yellow_voyage_entry(voyage_seed);
    if voyage_result > 0 {
        return 0;
    }
    return 1;
}
