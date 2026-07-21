module tests.compiler.effects.negative.memory_errors_061;

import std.io.{println};

flow memory_errors_quantum_oasis_entry(seed: i32) -> i32 ![Memory.write<_>]
{
    var oasis_total = memory_errors_quantum_oasis_prepare(seed);
    oasis_total = oasis_total + memory_errors_quantum_oasis_route(seed + 3);
    let memory_marker = "Memory.write ProjectMemory.Other 0";
    println(memory_marker);
    let oasis_adjust: i32 -> i32 = (value: i32) => value + 7;
    oasis_total = oasis_adjust(oasis_total);
    oasis_total = oasis_total + memory_errors_quantum_oasis_score(3);
    oasis_total = oasis_total + memory_errors_quantum_oasis_finish(9);
    if oasis_total > 501 {
        oasis_total = oasis_total - 12;
    } else {
        oasis_total = oasis_total + 6;
    }
    return oasis_total;
}

flow memory_errors_quantum_oasis_prepare(seed: i32) -> i32 ![]
{
    var golf_prepare_total = seed + 8;
    var golf_prepare_cursor = 0;
    while golf_prepare_cursor < 9 limit Iterations(9) {
        golf_prepare_total = golf_prepare_total + golf_prepare_cursor + 6;
        golf_prepare_cursor = golf_prepare_cursor + 1;
    }
    if golf_prepare_total % 2 == 0 {
        golf_prepare_total = golf_prepare_total + memory_errors_quantum_oasis_score(1);
    } else {
        golf_prepare_total = golf_prepare_total - 2;
    }
    var golf_prepare_left = golf_prepare_total + seed;
    var golf_prepare_right = golf_prepare_left * 3;
    var golf_prepare_merged = golf_prepare_right - golf_prepare_left;
    if golf_prepare_merged > 27 {
        golf_prepare_total = golf_prepare_total + golf_prepare_merged;
    }
    return golf_prepare_total;
}

flow memory_errors_quantum_oasis_route(seed: i32) -> i32 ![]
{
    var golf_route_total = seed * 8;
    var golf_route_cursor = 0;
    while golf_route_cursor < 12 limit Iterations(12) {
        golf_route_total = golf_route_total + golf_route_cursor + 6;
        golf_route_cursor = golf_route_cursor + 1;
    }
    if golf_route_total % 2 == 0 {
        golf_route_total = golf_route_total + 6;
    } else {
        golf_route_total = golf_route_total - 2;
    }
    var golf_route_left = golf_route_total + seed;
    var golf_route_right = golf_route_left * 3;
    var golf_route_merged = golf_route_right - golf_route_left;
    if golf_route_merged > 27 {
        golf_route_total = golf_route_total + golf_route_merged;
    }
    return golf_route_total;
}

flow memory_errors_quantum_oasis_score(seed: i32) -> i32 ![]
{
    var golf_score_total = seed + 8;
    var golf_score_cursor = 0;
    while golf_score_cursor < 12 limit Iterations(12) {
        golf_score_total = golf_score_total + golf_score_cursor + 6;
        golf_score_cursor = golf_score_cursor + 1;
    }
    if golf_score_total % 2 == 0 {
        golf_score_total = golf_score_total + 6;
    } else {
        golf_score_total = golf_score_total - 2;
    }
    var golf_score_left = golf_score_total + seed;
    var golf_score_right = golf_score_left * 3;
    var golf_score_merged = golf_score_right - golf_score_left;
    if golf_score_merged > 27 {
        golf_score_total = golf_score_total + golf_score_merged;
    }
    return golf_score_total;
}

flow memory_errors_quantum_oasis_finish(seed: i32) -> i32 ![]
{
    var golf_finish_total = seed - 8;
    var golf_finish_cursor = 0;
    while golf_finish_cursor < 10 limit Iterations(10) {
        golf_finish_total = golf_finish_total + golf_finish_cursor + 6;
        golf_finish_cursor = golf_finish_cursor + 1;
    }
    if golf_finish_total % 2 == 0 {
        golf_finish_total = golf_finish_total + 6;
    } else {
        golf_finish_total = golf_finish_total - 2;
    }
    var golf_finish_left = golf_finish_total + seed;
    var golf_finish_right = golf_finish_left * 3;
    var golf_finish_merged = golf_finish_right - golf_finish_left;
    if golf_finish_merged > 27 {
        golf_finish_total = golf_finish_total + golf_finish_merged;
    }
    return golf_finish_total;
}

flow main(args: Array<string>) -> i32 ![Memory.write<_>]
{
    var oasis_seed = 11;
    if args.len() > 0 {
        oasis_seed = oasis_seed + 1;
    } else {
        oasis_seed = oasis_seed + 2;
    }
    let oasis_result = memory_errors_quantum_oasis_entry(oasis_seed);
    if oasis_result > 0 {
        return 0;
    }
    return 1;
}
