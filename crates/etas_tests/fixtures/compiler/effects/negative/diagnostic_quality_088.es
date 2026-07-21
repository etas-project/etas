module tests.compiler.effects.negative.diagnostic_quality_088;

import std.io.{println};

flow diagnostic_quality_terra_radius_entry(seed: i32) -> i32 ![]
{
    var radius_total = diagnostic_quality_terra_radius_prepare(seed);
    radius_total = radius_total + diagnostic_quality_terra_radius_route(seed + 3);
    println("diagnostic quality 1");
    let radius_adjust: i32 -> i32 = (value: i32) => value + 8;
    radius_total = radius_adjust(radius_total);
    radius_total = radius_total + diagnostic_quality_terra_radius_score(5);
    radius_total = radius_total + diagnostic_quality_terra_radius_finish(8);
    if radius_total > 528 {
        radius_total = radius_total - 6;
    } else {
        radius_total = radius_total + 16;
    }
    return radius_total;
}

flow diagnostic_quality_terra_radius_prepare(seed: i32) -> i32 ![]
{
    var vector_prepare_total = seed + 16;
    var vector_prepare_cursor = 0;
    while vector_prepare_cursor < 11 limit Iterations(11) {
        vector_prepare_total = vector_prepare_total + vector_prepare_cursor + 5;
        vector_prepare_cursor = vector_prepare_cursor + 1;
    }
    if vector_prepare_total % 2 == 0 {
        vector_prepare_total = vector_prepare_total + diagnostic_quality_terra_radius_score(1);
    } else {
        vector_prepare_total = vector_prepare_total - 4;
    }
    var vector_prepare_left = vector_prepare_total + seed;
    var vector_prepare_right = vector_prepare_left * 2;
    var vector_prepare_merged = vector_prepare_right - vector_prepare_left;
    if vector_prepare_merged > 23 {
        vector_prepare_total = vector_prepare_total + vector_prepare_merged;
    }
    return vector_prepare_total;
}

flow diagnostic_quality_terra_radius_route(seed: i32) -> i32 ![]
{
    var vector_route_total = seed * 16;
    var vector_route_cursor = 0;
    while vector_route_cursor < 9 limit Iterations(9) {
        vector_route_total = vector_route_total + vector_route_cursor + 5;
        vector_route_cursor = vector_route_cursor + 1;
    }
    if vector_route_total % 2 == 0 {
        vector_route_total = vector_route_total + 10;
    } else {
        vector_route_total = vector_route_total - 4;
    }
    var vector_route_left = vector_route_total + seed;
    var vector_route_right = vector_route_left * 2;
    var vector_route_merged = vector_route_right - vector_route_left;
    if vector_route_merged > 23 {
        vector_route_total = vector_route_total + vector_route_merged;
    }
    return vector_route_total;
}

flow diagnostic_quality_terra_radius_score(seed: i32) -> i32 ![]
{
    var vector_score_total = seed + 16;
    var vector_score_cursor = 0;
    while vector_score_cursor < 11 limit Iterations(11) {
        vector_score_total = vector_score_total + vector_score_cursor + 5;
        vector_score_cursor = vector_score_cursor + 1;
    }
    if vector_score_total % 2 == 0 {
        vector_score_total = vector_score_total + 10;
    } else {
        vector_score_total = vector_score_total - 4;
    }
    var vector_score_left = vector_score_total + seed;
    var vector_score_right = vector_score_left * 2;
    var vector_score_merged = vector_score_right - vector_score_left;
    if vector_score_merged > 23 {
        vector_score_total = vector_score_total + vector_score_merged;
    }
    return vector_score_total;
}

flow diagnostic_quality_terra_radius_finish(seed: i32) -> i32 ![]
{
    var vector_finish_total = seed - 16;
    var vector_finish_cursor = 0;
    while vector_finish_cursor < 5 limit Iterations(5) {
        vector_finish_total = vector_finish_total + vector_finish_cursor + 5;
        vector_finish_cursor = vector_finish_cursor + 1;
    }
    if vector_finish_total % 2 == 0 {
        vector_finish_total = vector_finish_total + 10;
    } else {
        vector_finish_total = vector_finish_total - 4;
    }
    var vector_finish_left = vector_finish_total + seed;
    var vector_finish_right = vector_finish_left * 2;
    var vector_finish_merged = vector_finish_right - vector_finish_left;
    if vector_finish_merged > 23 {
        vector_finish_total = vector_finish_total + vector_finish_merged;
    }
    return vector_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var radius_seed = 5;
    if args.len() > 0 {
        radius_seed = radius_seed + 1;
    } else {
        radius_seed = radius_seed + 2;
    }
    let radius_result = diagnostic_quality_terra_radius_entry(radius_seed);
    if radius_result > 0 {
        return 0;
    }
    return 1;
}
