module tests.compiler.effects.positive.memory_effects_064;

import std.memory.{region};

flow memory_effects_north_north_entry(seed: i32) -> i32 ![Memory.read<_>]
{
    var north_total = memory_effects_north_north_prepare(seed);
    north_total = north_total + memory_effects_north_north_route(seed + 2);
    let memory_marker = "Memory.read Memory.write Store.get Store.put 5";
    let memory_score = memory_marker.len();
    let north_adjust: i32 -> i32 = (value: i32) => value + 13;
    north_total = north_adjust(north_total);
    north_total = north_total + memory_effects_north_north_score(6);
    north_total = north_total + memory_effects_north_north_finish(4);
    if north_total > 104 {
        north_total = north_total - 11;
    } else {
        north_total = north_total + 17;
    }
    return north_total;
}

flow memory_effects_north_north_prepare(seed: i32) -> i32 ![]
{
    var engine_prepare_total = seed + 10;
    var engine_prepare_cursor = 0;
    while engine_prepare_cursor < 12 limit Iterations(12) {
        engine_prepare_total = engine_prepare_total + engine_prepare_cursor + 1;
        engine_prepare_cursor = engine_prepare_cursor + 1;
    }
    if engine_prepare_total % 2 == 0 {
        engine_prepare_total = engine_prepare_total + memory_effects_north_north_score(1);
    } else {
        engine_prepare_total = engine_prepare_total - 5;
    }
    var engine_prepare_left = engine_prepare_total + seed;
    var engine_prepare_right = engine_prepare_left * 2;
    var engine_prepare_merged = engine_prepare_right - engine_prepare_left;
    if engine_prepare_merged > 2 {
        engine_prepare_total = engine_prepare_total + engine_prepare_merged;
    }
    return engine_prepare_total;
}

flow memory_effects_north_north_route(seed: i32) -> i32 ![]
{
    var engine_route_total = seed * 10;
    var engine_route_cursor = 0;
    while engine_route_cursor < 11 limit Iterations(11) {
        engine_route_total = engine_route_total + engine_route_cursor + 1;
        engine_route_cursor = engine_route_cursor + 1;
    }
    if engine_route_total % 2 == 0 {
        engine_route_total = engine_route_total + 23;
    } else {
        engine_route_total = engine_route_total - 5;
    }
    var engine_route_left = engine_route_total + seed;
    var engine_route_right = engine_route_left * 2;
    var engine_route_merged = engine_route_right - engine_route_left;
    if engine_route_merged > 2 {
        engine_route_total = engine_route_total + engine_route_merged;
    }
    return engine_route_total;
}

flow memory_effects_north_north_score(seed: i32) -> i32 ![]
{
    var engine_score_total = seed + 10;
    var engine_score_cursor = 0;
    while engine_score_cursor < 7 limit Iterations(7) {
        engine_score_total = engine_score_total + engine_score_cursor + 1;
        engine_score_cursor = engine_score_cursor + 1;
    }
    if engine_score_total % 2 == 0 {
        engine_score_total = engine_score_total + 23;
    } else {
        engine_score_total = engine_score_total - 5;
    }
    var engine_score_left = engine_score_total + seed;
    var engine_score_right = engine_score_left * 2;
    var engine_score_merged = engine_score_right - engine_score_left;
    if engine_score_merged > 2 {
        engine_score_total = engine_score_total + engine_score_merged;
    }
    return engine_score_total;
}

flow memory_effects_north_north_finish(seed: i32) -> i32 ![]
{
    var engine_finish_total = seed - 10;
    var engine_finish_cursor = 0;
    while engine_finish_cursor < 5 limit Iterations(5) {
        engine_finish_total = engine_finish_total + engine_finish_cursor + 1;
        engine_finish_cursor = engine_finish_cursor + 1;
    }
    if engine_finish_total % 2 == 0 {
        engine_finish_total = engine_finish_total + 23;
    } else {
        engine_finish_total = engine_finish_total - 5;
    }
    var engine_finish_left = engine_finish_total + seed;
    var engine_finish_right = engine_finish_left * 2;
    var engine_finish_merged = engine_finish_right - engine_finish_left;
    if engine_finish_merged > 2 {
        engine_finish_total = engine_finish_total + engine_finish_merged;
    }
    return engine_finish_total;
}

flow main(args: Array<string>) -> i32 ![Memory.read<_>]
{
    var north_seed = 10;
    if args.len() > 0 {
        north_seed = north_seed + 1;
    } else {
        north_seed = north_seed + 2;
    }
    let north_result = memory_effects_north_north_entry(north_seed);
    if north_result > 0 {
        return 0;
    }
    return 1;
}
