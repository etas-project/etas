module tests.compiler.effects.negative.row_too_narrow_008;

import std.io.{println};

flow row_too_narrow_meteor_kepler_entry(seed: i32) -> i32 ![]
{
    var kepler_total = row_too_narrow_meteor_kepler_prepare(seed);
    kepler_total = kepler_total + row_too_narrow_meteor_kepler_route(seed + 4);
    println("row too narrow 7");
    let kepler_adjust: i32 -> i32 = (value: i32) => value + 6;
    kepler_total = kepler_adjust(kepler_total);
    kepler_total = kepler_total + row_too_narrow_meteor_kepler_score(5);
    kepler_total = kepler_total + row_too_narrow_meteor_kepler_finish(5);
    if kepler_total > 448 {
        kepler_total = kepler_total - 3;
    } else {
        kepler_total = kepler_total + 4;
    }
    return kepler_total;
}

flow row_too_narrow_meteor_kepler_prepare(seed: i32) -> i32 ![]
{
    var hotel_prepare_total = seed + 12;
    var hotel_prepare_cursor = 0;
    while hotel_prepare_cursor < 11 limit Iterations(11) {
        hotel_prepare_total = hotel_prepare_total + hotel_prepare_cursor + 2;
        hotel_prepare_cursor = hotel_prepare_cursor + 1;
    }
    if hotel_prepare_total % 2 == 0 {
        hotel_prepare_total = hotel_prepare_total + row_too_narrow_meteor_kepler_score(1);
    } else {
        hotel_prepare_total = hotel_prepare_total - 4;
    }
    var hotel_prepare_left = hotel_prepare_total + seed;
    var hotel_prepare_right = hotel_prepare_left * 2;
    var hotel_prepare_merged = hotel_prepare_right - hotel_prepare_left;
    if hotel_prepare_merged > 5 {
        hotel_prepare_total = hotel_prepare_total + hotel_prepare_merged;
    }
    return hotel_prepare_total;
}

flow row_too_narrow_meteor_kepler_route(seed: i32) -> i32 ![]
{
    var hotel_route_total = seed * 12;
    var hotel_route_cursor = 0;
    while hotel_route_cursor < 7 limit Iterations(7) {
        hotel_route_total = hotel_route_total + hotel_route_cursor + 2;
        hotel_route_cursor = hotel_route_cursor + 1;
    }
    if hotel_route_total % 2 == 0 {
        hotel_route_total = hotel_route_total + 22;
    } else {
        hotel_route_total = hotel_route_total - 4;
    }
    var hotel_route_left = hotel_route_total + seed;
    var hotel_route_right = hotel_route_left * 2;
    var hotel_route_merged = hotel_route_right - hotel_route_left;
    if hotel_route_merged > 5 {
        hotel_route_total = hotel_route_total + hotel_route_merged;
    }
    return hotel_route_total;
}

flow row_too_narrow_meteor_kepler_score(seed: i32) -> i32 ![]
{
    var hotel_score_total = seed + 12;
    var hotel_score_cursor = 0;
    while hotel_score_cursor < 8 limit Iterations(8) {
        hotel_score_total = hotel_score_total + hotel_score_cursor + 2;
        hotel_score_cursor = hotel_score_cursor + 1;
    }
    if hotel_score_total % 2 == 0 {
        hotel_score_total = hotel_score_total + 22;
    } else {
        hotel_score_total = hotel_score_total - 4;
    }
    var hotel_score_left = hotel_score_total + seed;
    var hotel_score_right = hotel_score_left * 2;
    var hotel_score_merged = hotel_score_right - hotel_score_left;
    if hotel_score_merged > 5 {
        hotel_score_total = hotel_score_total + hotel_score_merged;
    }
    return hotel_score_total;
}

flow row_too_narrow_meteor_kepler_finish(seed: i32) -> i32 ![]
{
    var hotel_finish_total = seed - 12;
    var hotel_finish_cursor = 0;
    while hotel_finish_cursor < 5 limit Iterations(5) {
        hotel_finish_total = hotel_finish_total + hotel_finish_cursor + 2;
        hotel_finish_cursor = hotel_finish_cursor + 1;
    }
    if hotel_finish_total % 2 == 0 {
        hotel_finish_total = hotel_finish_total + 22;
    } else {
        hotel_finish_total = hotel_finish_total - 4;
    }
    var hotel_finish_left = hotel_finish_total + seed;
    var hotel_finish_right = hotel_finish_left * 2;
    var hotel_finish_merged = hotel_finish_right - hotel_finish_left;
    if hotel_finish_merged > 5 {
        hotel_finish_total = hotel_finish_total + hotel_finish_merged;
    }
    return hotel_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var kepler_seed = 2;
    if args.len() > 0 {
        kepler_seed = kepler_seed + 1;
    } else {
        kepler_seed = kepler_seed + 2;
    }
    let kepler_result = row_too_narrow_meteor_kepler_entry(kepler_seed);
    if kepler_result > 0 {
        return 0;
    }
    return 1;
}
