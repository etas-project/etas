module tests.compiler.effects.negative.memory_errors_062;

import std.io.{println};

flow memory_errors_rocket_parity_entry(seed: i32) -> i32 ![Memory.write<_>]
{
    var parity_total = memory_errors_rocket_parity_prepare(seed);
    parity_total = parity_total + memory_errors_rocket_parity_route(seed + 4);
    let memory_marker = "Memory.write ProjectMemory.Other 1";
    println(memory_marker);
    let parity_adjust: i32 -> i32 = (value: i32) => value + 8;
    parity_total = parity_adjust(parity_total);
    parity_total = parity_total + memory_errors_rocket_parity_score(4);
    parity_total = parity_total + memory_errors_rocket_parity_finish(3);
    if parity_total > 502 {
        parity_total = parity_total - 2;
    } else {
        parity_total = parity_total + 7;
    }
    return parity_total;
}

flow memory_errors_rocket_parity_prepare(seed: i32) -> i32 ![]
{
    var nectar_prepare_total = seed + 9;
    var nectar_prepare_cursor = 0;
    while nectar_prepare_cursor < 10 limit Iterations(10) {
        nectar_prepare_total = nectar_prepare_total + nectar_prepare_cursor + 0;
        nectar_prepare_cursor = nectar_prepare_cursor + 1;
    }
    if nectar_prepare_total % 2 == 0 {
        nectar_prepare_total = nectar_prepare_total + memory_errors_rocket_parity_score(1);
    } else {
        nectar_prepare_total = nectar_prepare_total - 3;
    }
    var nectar_prepare_left = nectar_prepare_total + seed;
    var nectar_prepare_right = nectar_prepare_left * 4;
    var nectar_prepare_merged = nectar_prepare_right - nectar_prepare_left;
    if nectar_prepare_merged > 28 {
        nectar_prepare_total = nectar_prepare_total + nectar_prepare_merged;
    }
    return nectar_prepare_total;
}

flow memory_errors_rocket_parity_route(seed: i32) -> i32 ![]
{
    var nectar_route_total = seed * 9;
    var nectar_route_cursor = 0;
    while nectar_route_cursor < 7 limit Iterations(7) {
        nectar_route_total = nectar_route_total + nectar_route_cursor + 0;
        nectar_route_cursor = nectar_route_cursor + 1;
    }
    if nectar_route_total % 2 == 0 {
        nectar_route_total = nectar_route_total + 7;
    } else {
        nectar_route_total = nectar_route_total - 3;
    }
    var nectar_route_left = nectar_route_total + seed;
    var nectar_route_right = nectar_route_left * 4;
    var nectar_route_merged = nectar_route_right - nectar_route_left;
    if nectar_route_merged > 28 {
        nectar_route_total = nectar_route_total + nectar_route_merged;
    }
    return nectar_route_total;
}

flow memory_errors_rocket_parity_score(seed: i32) -> i32 ![]
{
    var nectar_score_total = seed + 9;
    var nectar_score_cursor = 0;
    while nectar_score_cursor < 6 limit Iterations(6) {
        nectar_score_total = nectar_score_total + nectar_score_cursor + 0;
        nectar_score_cursor = nectar_score_cursor + 1;
    }
    if nectar_score_total % 2 == 0 {
        nectar_score_total = nectar_score_total + 7;
    } else {
        nectar_score_total = nectar_score_total - 3;
    }
    var nectar_score_left = nectar_score_total + seed;
    var nectar_score_right = nectar_score_left * 4;
    var nectar_score_merged = nectar_score_right - nectar_score_left;
    if nectar_score_merged > 28 {
        nectar_score_total = nectar_score_total + nectar_score_merged;
    }
    return nectar_score_total;
}

flow memory_errors_rocket_parity_finish(seed: i32) -> i32 ![]
{
    var nectar_finish_total = seed - 9;
    var nectar_finish_cursor = 0;
    while nectar_finish_cursor < 11 limit Iterations(11) {
        nectar_finish_total = nectar_finish_total + nectar_finish_cursor + 0;
        nectar_finish_cursor = nectar_finish_cursor + 1;
    }
    if nectar_finish_total % 2 == 0 {
        nectar_finish_total = nectar_finish_total + 7;
    } else {
        nectar_finish_total = nectar_finish_total - 3;
    }
    var nectar_finish_left = nectar_finish_total + seed;
    var nectar_finish_right = nectar_finish_left * 4;
    var nectar_finish_merged = nectar_finish_right - nectar_finish_left;
    if nectar_finish_merged > 28 {
        nectar_finish_total = nectar_finish_total + nectar_finish_merged;
    }
    return nectar_finish_total;
}

flow main(args: Array<string>) -> i32 ![Memory.write<_>]
{
    var parity_seed = 1;
    if args.len() > 0 {
        parity_seed = parity_seed + 1;
    } else {
        parity_seed = parity_seed + 2;
    }
    let parity_result = memory_errors_rocket_parity_entry(parity_seed);
    if parity_result > 0 {
        return 0;
    }
    return 1;
}
