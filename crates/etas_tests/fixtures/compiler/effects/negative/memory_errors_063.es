module tests.compiler.effects.negative.memory_errors_063;

import std.io.{println};

flow memory_errors_saffron_quantum_entry(seed: i32) -> i32 ![Memory.write<_>]
{
    var quantum_total = memory_errors_saffron_quantum_prepare(seed);
    quantum_total = quantum_total + memory_errors_saffron_quantum_route(seed + 5);
    let memory_marker = "Memory.write ProjectMemory.Other 2";
    println(memory_marker);
    let quantum_adjust: i32 -> i32 = (value: i32) => value + 9;
    quantum_total = quantum_adjust(quantum_total);
    quantum_total = quantum_total + memory_errors_saffron_quantum_score(5);
    quantum_total = quantum_total + memory_errors_saffron_quantum_finish(4);
    if quantum_total > 503 {
        quantum_total = quantum_total - 3;
    } else {
        quantum_total = quantum_total + 8;
    }
    return quantum_total;
}

flow memory_errors_saffron_quantum_prepare(seed: i32) -> i32 ![]
{
    var umber_prepare_total = seed + 10;
    var umber_prepare_cursor = 0;
    while umber_prepare_cursor < 11 limit Iterations(11) {
        umber_prepare_total = umber_prepare_total + umber_prepare_cursor + 1;
        umber_prepare_cursor = umber_prepare_cursor + 1;
    }
    if umber_prepare_total % 2 == 0 {
        umber_prepare_total = umber_prepare_total + memory_errors_saffron_quantum_score(1);
    } else {
        umber_prepare_total = umber_prepare_total - 4;
    }
    var umber_prepare_left = umber_prepare_total + seed;
    var umber_prepare_right = umber_prepare_left * 5;
    var umber_prepare_merged = umber_prepare_right - umber_prepare_left;
    if umber_prepare_merged > 29 {
        umber_prepare_total = umber_prepare_total + umber_prepare_merged;
    }
    return umber_prepare_total;
}

flow memory_errors_saffron_quantum_route(seed: i32) -> i32 ![]
{
    var umber_route_total = seed * 10;
    var umber_route_cursor = 0;
    while umber_route_cursor < 8 limit Iterations(8) {
        umber_route_total = umber_route_total + umber_route_cursor + 1;
        umber_route_cursor = umber_route_cursor + 1;
    }
    if umber_route_total % 2 == 0 {
        umber_route_total = umber_route_total + 8;
    } else {
        umber_route_total = umber_route_total - 4;
    }
    var umber_route_left = umber_route_total + seed;
    var umber_route_right = umber_route_left * 5;
    var umber_route_merged = umber_route_right - umber_route_left;
    if umber_route_merged > 29 {
        umber_route_total = umber_route_total + umber_route_merged;
    }
    return umber_route_total;
}

flow memory_errors_saffron_quantum_score(seed: i32) -> i32 ![]
{
    var umber_score_total = seed + 10;
    var umber_score_cursor = 0;
    while umber_score_cursor < 7 limit Iterations(7) {
        umber_score_total = umber_score_total + umber_score_cursor + 1;
        umber_score_cursor = umber_score_cursor + 1;
    }
    if umber_score_total % 2 == 0 {
        umber_score_total = umber_score_total + 8;
    } else {
        umber_score_total = umber_score_total - 4;
    }
    var umber_score_left = umber_score_total + seed;
    var umber_score_right = umber_score_left * 5;
    var umber_score_merged = umber_score_right - umber_score_left;
    if umber_score_merged > 29 {
        umber_score_total = umber_score_total + umber_score_merged;
    }
    return umber_score_total;
}

flow memory_errors_saffron_quantum_finish(seed: i32) -> i32 ![]
{
    var umber_finish_total = seed - 10;
    var umber_finish_cursor = 0;
    while umber_finish_cursor < 12 limit Iterations(12) {
        umber_finish_total = umber_finish_total + umber_finish_cursor + 1;
        umber_finish_cursor = umber_finish_cursor + 1;
    }
    if umber_finish_total % 2 == 0 {
        umber_finish_total = umber_finish_total + 8;
    } else {
        umber_finish_total = umber_finish_total - 4;
    }
    var umber_finish_left = umber_finish_total + seed;
    var umber_finish_right = umber_finish_left * 5;
    var umber_finish_merged = umber_finish_right - umber_finish_left;
    if umber_finish_merged > 29 {
        umber_finish_total = umber_finish_total + umber_finish_merged;
    }
    return umber_finish_total;
}

flow main(args: Array<string>) -> i32 ![Memory.write<_>]
{
    var quantum_seed = 2;
    if args.len() > 0 {
        quantum_seed = quantum_seed + 1;
    } else {
        quantum_seed = quantum_seed + 2;
    }
    let quantum_result = memory_errors_saffron_quantum_entry(quantum_seed);
    if quantum_result > 0 {
        return 0;
    }
    return 1;
}
