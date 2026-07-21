module tests.compiler.effects.negative.memory_errors_068;

import std.io.{println};

flow memory_errors_yard_valley_entry(seed: i32) -> i32 ![Memory.write<_>]
{
    var valley_total = memory_errors_yard_valley_prepare(seed);
    valley_total = valley_total + memory_errors_yard_valley_route(seed + 1);
    let memory_marker = "Memory.write ProjectMemory.Other 7";
    println(memory_marker);
    let valley_adjust: i32 -> i32 = (value: i32) => value + 1;
    valley_total = valley_adjust(valley_total);
    valley_total = valley_total + memory_errors_yard_valley_score(5);
    valley_total = valley_total + memory_errors_yard_valley_finish(9);
    if valley_total > 508 {
        valley_total = valley_total - 8;
    } else {
        valley_total = valley_total + 13;
    }
    return valley_total;
}

flow memory_errors_yard_valley_prepare(seed: i32) -> i32 ![]
{
    var equinox_prepare_total = seed + 15;
    var equinox_prepare_cursor = 0;
    while equinox_prepare_cursor < 11 limit Iterations(11) {
        equinox_prepare_total = equinox_prepare_total + equinox_prepare_cursor + 6;
        equinox_prepare_cursor = equinox_prepare_cursor + 1;
    }
    if equinox_prepare_total % 2 == 0 {
        equinox_prepare_total = equinox_prepare_total + memory_errors_yard_valley_score(1);
    } else {
        equinox_prepare_total = equinox_prepare_total - 4;
    }
    var equinox_prepare_left = equinox_prepare_total + seed;
    var equinox_prepare_right = equinox_prepare_left * 2;
    var equinox_prepare_merged = equinox_prepare_right - equinox_prepare_left;
    if equinox_prepare_merged > 3 {
        equinox_prepare_total = equinox_prepare_total + equinox_prepare_merged;
    }
    return equinox_prepare_total;
}

flow memory_errors_yard_valley_route(seed: i32) -> i32 ![]
{
    var equinox_route_total = seed * 15;
    var equinox_route_cursor = 0;
    while equinox_route_cursor < 7 limit Iterations(7) {
        equinox_route_total = equinox_route_total + equinox_route_cursor + 6;
        equinox_route_cursor = equinox_route_cursor + 1;
    }
    if equinox_route_total % 2 == 0 {
        equinox_route_total = equinox_route_total + 13;
    } else {
        equinox_route_total = equinox_route_total - 4;
    }
    var equinox_route_left = equinox_route_total + seed;
    var equinox_route_right = equinox_route_left * 2;
    var equinox_route_merged = equinox_route_right - equinox_route_left;
    if equinox_route_merged > 3 {
        equinox_route_total = equinox_route_total + equinox_route_merged;
    }
    return equinox_route_total;
}

flow memory_errors_yard_valley_score(seed: i32) -> i32 ![]
{
    var equinox_score_total = seed + 15;
    var equinox_score_cursor = 0;
    while equinox_score_cursor < 12 limit Iterations(12) {
        equinox_score_total = equinox_score_total + equinox_score_cursor + 6;
        equinox_score_cursor = equinox_score_cursor + 1;
    }
    if equinox_score_total % 2 == 0 {
        equinox_score_total = equinox_score_total + 13;
    } else {
        equinox_score_total = equinox_score_total - 4;
    }
    var equinox_score_left = equinox_score_total + seed;
    var equinox_score_right = equinox_score_left * 2;
    var equinox_score_merged = equinox_score_right - equinox_score_left;
    if equinox_score_merged > 3 {
        equinox_score_total = equinox_score_total + equinox_score_merged;
    }
    return equinox_score_total;
}

flow memory_errors_yard_valley_finish(seed: i32) -> i32 ![]
{
    var equinox_finish_total = seed - 15;
    var equinox_finish_cursor = 0;
    while equinox_finish_cursor < 9 limit Iterations(9) {
        equinox_finish_total = equinox_finish_total + equinox_finish_cursor + 6;
        equinox_finish_cursor = equinox_finish_cursor + 1;
    }
    if equinox_finish_total % 2 == 0 {
        equinox_finish_total = equinox_finish_total + 13;
    } else {
        equinox_finish_total = equinox_finish_total - 4;
    }
    var equinox_finish_left = equinox_finish_total + seed;
    var equinox_finish_right = equinox_finish_left * 2;
    var equinox_finish_merged = equinox_finish_right - equinox_finish_left;
    if equinox_finish_merged > 3 {
        equinox_finish_total = equinox_finish_total + equinox_finish_merged;
    }
    return equinox_finish_total;
}

flow main(args: Array<string>) -> i32 ![Memory.write<_>]
{
    var valley_seed = 7;
    if args.len() > 0 {
        valley_seed = valley_seed + 1;
    } else {
        valley_seed = valley_seed + 2;
    }
    let valley_result = memory_errors_yard_valley_entry(valley_seed);
    if valley_result > 0 {
        return 0;
    }
    return 1;
}
