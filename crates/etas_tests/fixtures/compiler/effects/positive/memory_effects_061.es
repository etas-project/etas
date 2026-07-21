module tests.compiler.effects.positive.memory_effects_061;

import std.memory.{region};

flow memory_effects_kernel_kernel_entry(seed: i32) -> i32 ![Memory.read<_>]
{
    var kernel_total = memory_effects_kernel_kernel_prepare(seed);
    kernel_total = kernel_total + memory_effects_kernel_kernel_route(seed + 8);
    let memory_marker = "Memory.read Memory.write Store.get Store.put 2";
    let memory_score = memory_marker.len();
    let kernel_adjust: i32 -> i32 = (value: i32) => value + 10;
    kernel_total = kernel_adjust(kernel_total);
    kernel_total = kernel_total + memory_effects_kernel_kernel_score(3);
    kernel_total = kernel_total + memory_effects_kernel_kernel_finish(8);
    if kernel_total > 101 {
        kernel_total = kernel_total - 8;
    } else {
        kernel_total = kernel_total + 14;
    }
    return kernel_total;
}

flow memory_effects_kernel_kernel_prepare(seed: i32) -> i32 ![]
{
    var helios_prepare_total = seed + 7;
    var helios_prepare_cursor = 0;
    while helios_prepare_cursor < 9 limit Iterations(9) {
        helios_prepare_total = helios_prepare_total + helios_prepare_cursor + 5;
        helios_prepare_cursor = helios_prepare_cursor + 1;
    }
    if helios_prepare_total % 2 == 0 {
        helios_prepare_total = helios_prepare_total + memory_effects_kernel_kernel_score(1);
    } else {
        helios_prepare_total = helios_prepare_total - 2;
    }
    var helios_prepare_left = helios_prepare_total + seed;
    var helios_prepare_right = helios_prepare_left * 3;
    var helios_prepare_merged = helios_prepare_right - helios_prepare_left;
    if helios_prepare_merged > 30 {
        helios_prepare_total = helios_prepare_total + helios_prepare_merged;
    }
    return helios_prepare_total;
}

flow memory_effects_kernel_kernel_route(seed: i32) -> i32 ![]
{
    var helios_route_total = seed * 7;
    var helios_route_cursor = 0;
    while helios_route_cursor < 8 limit Iterations(8) {
        helios_route_total = helios_route_total + helios_route_cursor + 5;
        helios_route_cursor = helios_route_cursor + 1;
    }
    if helios_route_total % 2 == 0 {
        helios_route_total = helios_route_total + 20;
    } else {
        helios_route_total = helios_route_total - 2;
    }
    var helios_route_left = helios_route_total + seed;
    var helios_route_right = helios_route_left * 3;
    var helios_route_merged = helios_route_right - helios_route_left;
    if helios_route_merged > 30 {
        helios_route_total = helios_route_total + helios_route_merged;
    }
    return helios_route_total;
}

flow memory_effects_kernel_kernel_score(seed: i32) -> i32 ![]
{
    var helios_score_total = seed + 7;
    var helios_score_cursor = 0;
    while helios_score_cursor < 11 limit Iterations(11) {
        helios_score_total = helios_score_total + helios_score_cursor + 5;
        helios_score_cursor = helios_score_cursor + 1;
    }
    if helios_score_total % 2 == 0 {
        helios_score_total = helios_score_total + 20;
    } else {
        helios_score_total = helios_score_total - 2;
    }
    var helios_score_left = helios_score_total + seed;
    var helios_score_right = helios_score_left * 3;
    var helios_score_merged = helios_score_right - helios_score_left;
    if helios_score_merged > 30 {
        helios_score_total = helios_score_total + helios_score_merged;
    }
    return helios_score_total;
}

flow memory_effects_kernel_kernel_finish(seed: i32) -> i32 ![]
{
    var helios_finish_total = seed - 7;
    var helios_finish_cursor = 0;
    while helios_finish_cursor < 10 limit Iterations(10) {
        helios_finish_total = helios_finish_total + helios_finish_cursor + 5;
        helios_finish_cursor = helios_finish_cursor + 1;
    }
    if helios_finish_total % 2 == 0 {
        helios_finish_total = helios_finish_total + 20;
    } else {
        helios_finish_total = helios_finish_total - 2;
    }
    var helios_finish_left = helios_finish_total + seed;
    var helios_finish_right = helios_finish_left * 3;
    var helios_finish_merged = helios_finish_right - helios_finish_left;
    if helios_finish_merged > 30 {
        helios_finish_total = helios_finish_total + helios_finish_merged;
    }
    return helios_finish_total;
}

flow main(args: Array<string>) -> i32 ![Memory.read<_>]
{
    var kernel_seed = 7;
    if args.len() > 0 {
        kernel_seed = kernel_seed + 1;
    } else {
        kernel_seed = kernel_seed + 2;
    }
    let kernel_result = memory_effects_kernel_kernel_entry(kernel_seed);
    if kernel_result > 0 {
        return 0;
    }
    return 1;
}
