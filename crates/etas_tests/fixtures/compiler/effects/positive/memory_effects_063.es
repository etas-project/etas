module tests.compiler.effects.positive.memory_effects_063;

import std.memory.{region};

flow memory_effects_matrix_matrix_entry(seed: i32) -> i32 ![Memory.read<_>]
{
    var matrix_total = memory_effects_matrix_matrix_prepare(seed);
    matrix_total = matrix_total + memory_effects_matrix_matrix_route(seed + 1);
    let memory_marker = "Memory.read Memory.write Store.get Store.put 4";
    let memory_score = memory_marker.len();
    let matrix_adjust: i32 -> i32 = (value: i32) => value + 12;
    matrix_total = matrix_adjust(matrix_total);
    matrix_total = matrix_total + memory_effects_matrix_matrix_score(5);
    matrix_total = matrix_total + memory_effects_matrix_matrix_finish(3);
    if matrix_total > 103 {
        matrix_total = matrix_total - 10;
    } else {
        matrix_total = matrix_total + 16;
    }
    return matrix_total;
}

flow memory_effects_matrix_matrix_prepare(seed: i32) -> i32 ![]
{
    var wander_prepare_total = seed + 9;
    var wander_prepare_cursor = 0;
    while wander_prepare_cursor < 11 limit Iterations(11) {
        wander_prepare_total = wander_prepare_total + wander_prepare_cursor + 0;
        wander_prepare_cursor = wander_prepare_cursor + 1;
    }
    if wander_prepare_total % 2 == 0 {
        wander_prepare_total = wander_prepare_total + memory_effects_matrix_matrix_score(1);
    } else {
        wander_prepare_total = wander_prepare_total - 4;
    }
    var wander_prepare_left = wander_prepare_total + seed;
    var wander_prepare_right = wander_prepare_left * 5;
    var wander_prepare_merged = wander_prepare_right - wander_prepare_left;
    if wander_prepare_merged > 1 {
        wander_prepare_total = wander_prepare_total + wander_prepare_merged;
    }
    return wander_prepare_total;
}

flow memory_effects_matrix_matrix_route(seed: i32) -> i32 ![]
{
    var wander_route_total = seed * 9;
    var wander_route_cursor = 0;
    while wander_route_cursor < 10 limit Iterations(10) {
        wander_route_total = wander_route_total + wander_route_cursor + 0;
        wander_route_cursor = wander_route_cursor + 1;
    }
    if wander_route_total % 2 == 0 {
        wander_route_total = wander_route_total + 22;
    } else {
        wander_route_total = wander_route_total - 4;
    }
    var wander_route_left = wander_route_total + seed;
    var wander_route_right = wander_route_left * 5;
    var wander_route_merged = wander_route_right - wander_route_left;
    if wander_route_merged > 1 {
        wander_route_total = wander_route_total + wander_route_merged;
    }
    return wander_route_total;
}

flow memory_effects_matrix_matrix_score(seed: i32) -> i32 ![]
{
    var wander_score_total = seed + 9;
    var wander_score_cursor = 0;
    while wander_score_cursor < 6 limit Iterations(6) {
        wander_score_total = wander_score_total + wander_score_cursor + 0;
        wander_score_cursor = wander_score_cursor + 1;
    }
    if wander_score_total % 2 == 0 {
        wander_score_total = wander_score_total + 22;
    } else {
        wander_score_total = wander_score_total - 4;
    }
    var wander_score_left = wander_score_total + seed;
    var wander_score_right = wander_score_left * 5;
    var wander_score_merged = wander_score_right - wander_score_left;
    if wander_score_merged > 1 {
        wander_score_total = wander_score_total + wander_score_merged;
    }
    return wander_score_total;
}

flow memory_effects_matrix_matrix_finish(seed: i32) -> i32 ![]
{
    var wander_finish_total = seed - 9;
    var wander_finish_cursor = 0;
    while wander_finish_cursor < 12 limit Iterations(12) {
        wander_finish_total = wander_finish_total + wander_finish_cursor + 0;
        wander_finish_cursor = wander_finish_cursor + 1;
    }
    if wander_finish_total % 2 == 0 {
        wander_finish_total = wander_finish_total + 22;
    } else {
        wander_finish_total = wander_finish_total - 4;
    }
    var wander_finish_left = wander_finish_total + seed;
    var wander_finish_right = wander_finish_left * 5;
    var wander_finish_merged = wander_finish_right - wander_finish_left;
    if wander_finish_merged > 1 {
        wander_finish_total = wander_finish_total + wander_finish_merged;
    }
    return wander_finish_total;
}

flow main(args: Array<string>) -> i32 ![Memory.read<_>]
{
    var matrix_seed = 9;
    if args.len() > 0 {
        matrix_seed = matrix_seed + 1;
    } else {
        matrix_seed = matrix_seed + 2;
    }
    let matrix_result = memory_effects_matrix_matrix_entry(matrix_seed);
    if matrix_result > 0 {
        return 0;
    }
    return 1;
}
