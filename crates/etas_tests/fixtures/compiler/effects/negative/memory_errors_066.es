module tests.compiler.effects.negative.memory_errors_066;

import std.io.{println};

flow memory_errors_valley_temple_entry(seed: i32) -> i32 ![Memory.write<_>]
{
    var temple_total = memory_errors_valley_temple_prepare(seed);
    temple_total = temple_total + memory_errors_valley_temple_route(seed + 8);
    let memory_marker = "Memory.write ProjectMemory.Other 5";
    println(memory_marker);
    let temple_adjust: i32 -> i32 = (value: i32) => value + 12;
    temple_total = temple_adjust(temple_total);
    temple_total = temple_total + memory_errors_valley_temple_score(3);
    temple_total = temple_total + memory_errors_valley_temple_finish(7);
    if temple_total > 506 {
        temple_total = temple_total - 6;
    } else {
        temple_total = temple_total + 11;
    }
    return temple_total;
}

flow memory_errors_valley_temple_prepare(seed: i32) -> i32 ![]
{
    var prairie_prepare_total = seed + 13;
    var prairie_prepare_cursor = 0;
    while prairie_prepare_cursor < 9 limit Iterations(9) {
        prairie_prepare_total = prairie_prepare_total + prairie_prepare_cursor + 4;
        prairie_prepare_cursor = prairie_prepare_cursor + 1;
    }
    if prairie_prepare_total % 2 == 0 {
        prairie_prepare_total = prairie_prepare_total + memory_errors_valley_temple_score(1);
    } else {
        prairie_prepare_total = prairie_prepare_total - 2;
    }
    var prairie_prepare_left = prairie_prepare_total + seed;
    var prairie_prepare_right = prairie_prepare_left * 4;
    var prairie_prepare_merged = prairie_prepare_right - prairie_prepare_left;
    if prairie_prepare_merged > 1 {
        prairie_prepare_total = prairie_prepare_total + prairie_prepare_merged;
    }
    return prairie_prepare_total;
}

flow memory_errors_valley_temple_route(seed: i32) -> i32 ![]
{
    var prairie_route_total = seed * 13;
    var prairie_route_cursor = 0;
    while prairie_route_cursor < 11 limit Iterations(11) {
        prairie_route_total = prairie_route_total + prairie_route_cursor + 4;
        prairie_route_cursor = prairie_route_cursor + 1;
    }
    if prairie_route_total % 2 == 0 {
        prairie_route_total = prairie_route_total + 11;
    } else {
        prairie_route_total = prairie_route_total - 2;
    }
    var prairie_route_left = prairie_route_total + seed;
    var prairie_route_right = prairie_route_left * 4;
    var prairie_route_merged = prairie_route_right - prairie_route_left;
    if prairie_route_merged > 1 {
        prairie_route_total = prairie_route_total + prairie_route_merged;
    }
    return prairie_route_total;
}

flow memory_errors_valley_temple_score(seed: i32) -> i32 ![]
{
    var prairie_score_total = seed + 13;
    var prairie_score_cursor = 0;
    while prairie_score_cursor < 10 limit Iterations(10) {
        prairie_score_total = prairie_score_total + prairie_score_cursor + 4;
        prairie_score_cursor = prairie_score_cursor + 1;
    }
    if prairie_score_total % 2 == 0 {
        prairie_score_total = prairie_score_total + 11;
    } else {
        prairie_score_total = prairie_score_total - 2;
    }
    var prairie_score_left = prairie_score_total + seed;
    var prairie_score_right = prairie_score_left * 4;
    var prairie_score_merged = prairie_score_right - prairie_score_left;
    if prairie_score_merged > 1 {
        prairie_score_total = prairie_score_total + prairie_score_merged;
    }
    return prairie_score_total;
}

flow memory_errors_valley_temple_finish(seed: i32) -> i32 ![]
{
    var prairie_finish_total = seed - 13;
    var prairie_finish_cursor = 0;
    while prairie_finish_cursor < 7 limit Iterations(7) {
        prairie_finish_total = prairie_finish_total + prairie_finish_cursor + 4;
        prairie_finish_cursor = prairie_finish_cursor + 1;
    }
    if prairie_finish_total % 2 == 0 {
        prairie_finish_total = prairie_finish_total + 11;
    } else {
        prairie_finish_total = prairie_finish_total - 2;
    }
    var prairie_finish_left = prairie_finish_total + seed;
    var prairie_finish_right = prairie_finish_left * 4;
    var prairie_finish_merged = prairie_finish_right - prairie_finish_left;
    if prairie_finish_merged > 1 {
        prairie_finish_total = prairie_finish_total + prairie_finish_merged;
    }
    return prairie_finish_total;
}

flow main(args: Array<string>) -> i32 ![Memory.write<_>]
{
    var temple_seed = 5;
    if args.len() > 0 {
        temple_seed = temple_seed + 1;
    } else {
        temple_seed = temple_seed + 2;
    }
    let temple_result = memory_errors_valley_temple_entry(temple_seed);
    if temple_result > 0 {
        return 0;
    }
    return 1;
}
