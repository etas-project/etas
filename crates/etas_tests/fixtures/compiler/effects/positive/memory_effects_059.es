module tests.compiler.effects.positive.memory_effects_059;

import std.memory.{region};

flow memory_effects_islet_islet_entry(seed: i32) -> i32 ![Memory.read<_>]
{
    var islet_total = memory_effects_islet_islet_prepare(seed);
    islet_total = islet_total + memory_effects_islet_islet_route(seed + 6);
    let memory_marker = "Memory.read Memory.write Store.get Store.put 0";
    let memory_score = memory_marker.len();
    let islet_adjust: i32 -> i32 = (value: i32) => value + 8;
    islet_total = islet_adjust(islet_total);
    islet_total = islet_total + memory_effects_islet_islet_score(6);
    islet_total = islet_total + memory_effects_islet_islet_finish(6);
    if islet_total > 99 {
        islet_total = islet_total - 6;
    } else {
        islet_total = islet_total + 12;
    }
    return islet_total;
}

flow memory_effects_islet_islet_prepare(seed: i32) -> i32 ![]
{
    var summit_prepare_total = seed + 5;
    var summit_prepare_cursor = 0;
    while summit_prepare_cursor < 12 limit Iterations(12) {
        summit_prepare_total = summit_prepare_total + summit_prepare_cursor + 3;
        summit_prepare_cursor = summit_prepare_cursor + 1;
    }
    if summit_prepare_total % 2 == 0 {
        summit_prepare_total = summit_prepare_total + memory_effects_islet_islet_score(1);
    } else {
        summit_prepare_total = summit_prepare_total - 5;
    }
    var summit_prepare_left = summit_prepare_total + seed;
    var summit_prepare_right = summit_prepare_left * 5;
    var summit_prepare_merged = summit_prepare_right - summit_prepare_left;
    if summit_prepare_merged > 28 {
        summit_prepare_total = summit_prepare_total + summit_prepare_merged;
    }
    return summit_prepare_total;
}

flow memory_effects_islet_islet_route(seed: i32) -> i32 ![]
{
    var summit_route_total = seed * 5;
    var summit_route_cursor = 0;
    while summit_route_cursor < 12 limit Iterations(12) {
        summit_route_total = summit_route_total + summit_route_cursor + 3;
        summit_route_cursor = summit_route_cursor + 1;
    }
    if summit_route_total % 2 == 0 {
        summit_route_total = summit_route_total + 18;
    } else {
        summit_route_total = summit_route_total - 5;
    }
    var summit_route_left = summit_route_total + seed;
    var summit_route_right = summit_route_left * 5;
    var summit_route_merged = summit_route_right - summit_route_left;
    if summit_route_merged > 28 {
        summit_route_total = summit_route_total + summit_route_merged;
    }
    return summit_route_total;
}

flow memory_effects_islet_islet_score(seed: i32) -> i32 ![]
{
    var summit_score_total = seed + 5;
    var summit_score_cursor = 0;
    while summit_score_cursor < 9 limit Iterations(9) {
        summit_score_total = summit_score_total + summit_score_cursor + 3;
        summit_score_cursor = summit_score_cursor + 1;
    }
    if summit_score_total % 2 == 0 {
        summit_score_total = summit_score_total + 18;
    } else {
        summit_score_total = summit_score_total - 5;
    }
    var summit_score_left = summit_score_total + seed;
    var summit_score_right = summit_score_left * 5;
    var summit_score_merged = summit_score_right - summit_score_left;
    if summit_score_merged > 28 {
        summit_score_total = summit_score_total + summit_score_merged;
    }
    return summit_score_total;
}

flow memory_effects_islet_islet_finish(seed: i32) -> i32 ![]
{
    var summit_finish_total = seed - 5;
    var summit_finish_cursor = 0;
    while summit_finish_cursor < 8 limit Iterations(8) {
        summit_finish_total = summit_finish_total + summit_finish_cursor + 3;
        summit_finish_cursor = summit_finish_cursor + 1;
    }
    if summit_finish_total % 2 == 0 {
        summit_finish_total = summit_finish_total + 18;
    } else {
        summit_finish_total = summit_finish_total - 5;
    }
    var summit_finish_left = summit_finish_total + seed;
    var summit_finish_right = summit_finish_left * 5;
    var summit_finish_merged = summit_finish_right - summit_finish_left;
    if summit_finish_merged > 28 {
        summit_finish_total = summit_finish_total + summit_finish_merged;
    }
    return summit_finish_total;
}

flow main(args: Array<string>) -> i32 ![Memory.read<_>]
{
    var islet_seed = 5;
    if args.len() > 0 {
        islet_seed = islet_seed + 1;
    } else {
        islet_seed = islet_seed + 2;
    }
    let islet_result = memory_effects_islet_islet_entry(islet_seed);
    if islet_result > 0 {
        return 0;
    }
    return 1;
}
