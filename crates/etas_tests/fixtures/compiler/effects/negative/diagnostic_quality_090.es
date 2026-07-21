module tests.compiler.effects.negative.diagnostic_quality_090;

import std.io.{println};

flow diagnostic_quality_voyage_terra_entry(seed: i32) -> i32 ![]
{
    var terra_total = diagnostic_quality_voyage_terra_prepare(seed);
    terra_total = terra_total + diagnostic_quality_voyage_terra_route(seed + 5);
    println("diagnostic quality 3");
    let terra_adjust: i32 -> i32 = (value: i32) => value + 10;
    terra_total = terra_adjust(terra_total);
    terra_total = terra_total + diagnostic_quality_voyage_terra_score(2);
    terra_total = terra_total + diagnostic_quality_voyage_terra_finish(3);
    if terra_total > 530 {
        terra_total = terra_total - 8;
    } else {
        terra_total = terra_total + 18;
    }
    return terra_total;
}

flow diagnostic_quality_voyage_terra_prepare(seed: i32) -> i32 ![]
{
    var kingdom_prepare_total = seed + 18;
    var kingdom_prepare_cursor = 0;
    while kingdom_prepare_cursor < 8 limit Iterations(8) {
        kingdom_prepare_total = kingdom_prepare_total + kingdom_prepare_cursor + 0;
        kingdom_prepare_cursor = kingdom_prepare_cursor + 1;
    }
    if kingdom_prepare_total % 2 == 0 {
        kingdom_prepare_total = kingdom_prepare_total + diagnostic_quality_voyage_terra_score(1);
    } else {
        kingdom_prepare_total = kingdom_prepare_total - 1;
    }
    var kingdom_prepare_left = kingdom_prepare_total + seed;
    var kingdom_prepare_right = kingdom_prepare_left * 4;
    var kingdom_prepare_merged = kingdom_prepare_right - kingdom_prepare_left;
    if kingdom_prepare_merged > 25 {
        kingdom_prepare_total = kingdom_prepare_total + kingdom_prepare_merged;
    }
    return kingdom_prepare_total;
}

flow diagnostic_quality_voyage_terra_route(seed: i32) -> i32 ![]
{
    var kingdom_route_total = seed * 18;
    var kingdom_route_cursor = 0;
    while kingdom_route_cursor < 11 limit Iterations(11) {
        kingdom_route_total = kingdom_route_total + kingdom_route_cursor + 0;
        kingdom_route_cursor = kingdom_route_cursor + 1;
    }
    if kingdom_route_total % 2 == 0 {
        kingdom_route_total = kingdom_route_total + 12;
    } else {
        kingdom_route_total = kingdom_route_total - 1;
    }
    var kingdom_route_left = kingdom_route_total + seed;
    var kingdom_route_right = kingdom_route_left * 4;
    var kingdom_route_merged = kingdom_route_right - kingdom_route_left;
    if kingdom_route_merged > 25 {
        kingdom_route_total = kingdom_route_total + kingdom_route_merged;
    }
    return kingdom_route_total;
}

flow diagnostic_quality_voyage_terra_score(seed: i32) -> i32 ![]
{
    var kingdom_score_total = seed + 18;
    var kingdom_score_cursor = 0;
    while kingdom_score_cursor < 6 limit Iterations(6) {
        kingdom_score_total = kingdom_score_total + kingdom_score_cursor + 0;
        kingdom_score_cursor = kingdom_score_cursor + 1;
    }
    if kingdom_score_total % 2 == 0 {
        kingdom_score_total = kingdom_score_total + 12;
    } else {
        kingdom_score_total = kingdom_score_total - 1;
    }
    var kingdom_score_left = kingdom_score_total + seed;
    var kingdom_score_right = kingdom_score_left * 4;
    var kingdom_score_merged = kingdom_score_right - kingdom_score_left;
    if kingdom_score_merged > 25 {
        kingdom_score_total = kingdom_score_total + kingdom_score_merged;
    }
    return kingdom_score_total;
}

flow diagnostic_quality_voyage_terra_finish(seed: i32) -> i32 ![]
{
    var kingdom_finish_total = seed - 18;
    var kingdom_finish_cursor = 0;
    while kingdom_finish_cursor < 7 limit Iterations(7) {
        kingdom_finish_total = kingdom_finish_total + kingdom_finish_cursor + 0;
        kingdom_finish_cursor = kingdom_finish_cursor + 1;
    }
    if kingdom_finish_total % 2 == 0 {
        kingdom_finish_total = kingdom_finish_total + 12;
    } else {
        kingdom_finish_total = kingdom_finish_total - 1;
    }
    var kingdom_finish_left = kingdom_finish_total + seed;
    var kingdom_finish_right = kingdom_finish_left * 4;
    var kingdom_finish_merged = kingdom_finish_right - kingdom_finish_left;
    if kingdom_finish_merged > 25 {
        kingdom_finish_total = kingdom_finish_total + kingdom_finish_merged;
    }
    return kingdom_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var terra_seed = 7;
    if args.len() > 0 {
        terra_seed = terra_seed + 1;
    } else {
        terra_seed = terra_seed + 2;
    }
    let terra_result = diagnostic_quality_voyage_terra_entry(terra_seed);
    if terra_result > 0 {
        return 0;
    }
    return 1;
}
