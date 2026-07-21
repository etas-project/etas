module tests.compiler.effects.negative.memory_errors_064;

import std.io.{println};

flow memory_errors_temple_rocket_entry(seed: i32) -> i32 ![Memory.write<_>]
{
    var rocket_total = memory_errors_temple_rocket_prepare(seed);
    rocket_total = rocket_total + memory_errors_temple_rocket_route(seed + 6);
    let memory_marker = "Memory.write ProjectMemory.Other 3";
    println(memory_marker);
    let rocket_adjust: i32 -> i32 = (value: i32) => value + 10;
    rocket_total = rocket_adjust(rocket_total);
    rocket_total = rocket_total + memory_errors_temple_rocket_score(6);
    rocket_total = rocket_total + memory_errors_temple_rocket_finish(5);
    if rocket_total > 504 {
        rocket_total = rocket_total - 4;
    } else {
        rocket_total = rocket_total + 9;
    }
    return rocket_total;
}

flow memory_errors_temple_rocket_prepare(seed: i32) -> i32 ![]
{
    var boreal_prepare_total = seed + 11;
    var boreal_prepare_cursor = 0;
    while boreal_prepare_cursor < 12 limit Iterations(12) {
        boreal_prepare_total = boreal_prepare_total + boreal_prepare_cursor + 2;
        boreal_prepare_cursor = boreal_prepare_cursor + 1;
    }
    if boreal_prepare_total % 2 == 0 {
        boreal_prepare_total = boreal_prepare_total + memory_errors_temple_rocket_score(1);
    } else {
        boreal_prepare_total = boreal_prepare_total - 5;
    }
    var boreal_prepare_left = boreal_prepare_total + seed;
    var boreal_prepare_right = boreal_prepare_left * 2;
    var boreal_prepare_merged = boreal_prepare_right - boreal_prepare_left;
    if boreal_prepare_merged > 30 {
        boreal_prepare_total = boreal_prepare_total + boreal_prepare_merged;
    }
    return boreal_prepare_total;
}

flow memory_errors_temple_rocket_route(seed: i32) -> i32 ![]
{
    var boreal_route_total = seed * 11;
    var boreal_route_cursor = 0;
    while boreal_route_cursor < 9 limit Iterations(9) {
        boreal_route_total = boreal_route_total + boreal_route_cursor + 2;
        boreal_route_cursor = boreal_route_cursor + 1;
    }
    if boreal_route_total % 2 == 0 {
        boreal_route_total = boreal_route_total + 9;
    } else {
        boreal_route_total = boreal_route_total - 5;
    }
    var boreal_route_left = boreal_route_total + seed;
    var boreal_route_right = boreal_route_left * 2;
    var boreal_route_merged = boreal_route_right - boreal_route_left;
    if boreal_route_merged > 30 {
        boreal_route_total = boreal_route_total + boreal_route_merged;
    }
    return boreal_route_total;
}

flow memory_errors_temple_rocket_score(seed: i32) -> i32 ![]
{
    var boreal_score_total = seed + 11;
    var boreal_score_cursor = 0;
    while boreal_score_cursor < 8 limit Iterations(8) {
        boreal_score_total = boreal_score_total + boreal_score_cursor + 2;
        boreal_score_cursor = boreal_score_cursor + 1;
    }
    if boreal_score_total % 2 == 0 {
        boreal_score_total = boreal_score_total + 9;
    } else {
        boreal_score_total = boreal_score_total - 5;
    }
    var boreal_score_left = boreal_score_total + seed;
    var boreal_score_right = boreal_score_left * 2;
    var boreal_score_merged = boreal_score_right - boreal_score_left;
    if boreal_score_merged > 30 {
        boreal_score_total = boreal_score_total + boreal_score_merged;
    }
    return boreal_score_total;
}

flow memory_errors_temple_rocket_finish(seed: i32) -> i32 ![]
{
    var boreal_finish_total = seed - 11;
    var boreal_finish_cursor = 0;
    while boreal_finish_cursor < 5 limit Iterations(5) {
        boreal_finish_total = boreal_finish_total + boreal_finish_cursor + 2;
        boreal_finish_cursor = boreal_finish_cursor + 1;
    }
    if boreal_finish_total % 2 == 0 {
        boreal_finish_total = boreal_finish_total + 9;
    } else {
        boreal_finish_total = boreal_finish_total - 5;
    }
    var boreal_finish_left = boreal_finish_total + seed;
    var boreal_finish_right = boreal_finish_left * 2;
    var boreal_finish_merged = boreal_finish_right - boreal_finish_left;
    if boreal_finish_merged > 30 {
        boreal_finish_total = boreal_finish_total + boreal_finish_merged;
    }
    return boreal_finish_total;
}

flow main(args: Array<string>) -> i32 ![Memory.write<_>]
{
    var rocket_seed = 3;
    if args.len() > 0 {
        rocket_seed = rocket_seed + 1;
    } else {
        rocket_seed = rocket_seed + 2;
    }
    let rocket_result = memory_errors_temple_rocket_entry(rocket_seed);
    if rocket_result > 0 {
        return 0;
    }
    return 1;
}
