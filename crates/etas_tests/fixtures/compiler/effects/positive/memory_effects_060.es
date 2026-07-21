module tests.compiler.effects.positive.memory_effects_060;

import std.memory.{region};

flow memory_effects_jade_jade_entry(seed: i32) -> i32 ![Memory.read<_>]
{
    var jade_total = memory_effects_jade_jade_prepare(seed);
    jade_total = jade_total + memory_effects_jade_jade_route(seed + 7);
    let memory_marker = "Memory.read Memory.write Store.get Store.put 1";
    let memory_score = memory_marker.len();
    let jade_adjust: i32 -> i32 = (value: i32) => value + 9;
    jade_total = jade_adjust(jade_total);
    jade_total = jade_total + memory_effects_jade_jade_score(2);
    jade_total = jade_total + memory_effects_jade_jade_finish(7);
    if jade_total > 100 {
        jade_total = jade_total - 7;
    } else {
        jade_total = jade_total + 13;
    }
    return jade_total;
}

flow memory_effects_jade_jade_prepare(seed: i32) -> i32 ![]
{
    var atlas_prepare_total = seed + 6;
    var atlas_prepare_cursor = 0;
    while atlas_prepare_cursor < 8 limit Iterations(8) {
        atlas_prepare_total = atlas_prepare_total + atlas_prepare_cursor + 4;
        atlas_prepare_cursor = atlas_prepare_cursor + 1;
    }
    if atlas_prepare_total % 2 == 0 {
        atlas_prepare_total = atlas_prepare_total + memory_effects_jade_jade_score(1);
    } else {
        atlas_prepare_total = atlas_prepare_total - 1;
    }
    var atlas_prepare_left = atlas_prepare_total + seed;
    var atlas_prepare_right = atlas_prepare_left * 2;
    var atlas_prepare_merged = atlas_prepare_right - atlas_prepare_left;
    if atlas_prepare_merged > 29 {
        atlas_prepare_total = atlas_prepare_total + atlas_prepare_merged;
    }
    return atlas_prepare_total;
}

flow memory_effects_jade_jade_route(seed: i32) -> i32 ![]
{
    var atlas_route_total = seed * 6;
    var atlas_route_cursor = 0;
    while atlas_route_cursor < 7 limit Iterations(7) {
        atlas_route_total = atlas_route_total + atlas_route_cursor + 4;
        atlas_route_cursor = atlas_route_cursor + 1;
    }
    if atlas_route_total % 2 == 0 {
        atlas_route_total = atlas_route_total + 19;
    } else {
        atlas_route_total = atlas_route_total - 1;
    }
    var atlas_route_left = atlas_route_total + seed;
    var atlas_route_right = atlas_route_left * 2;
    var atlas_route_merged = atlas_route_right - atlas_route_left;
    if atlas_route_merged > 29 {
        atlas_route_total = atlas_route_total + atlas_route_merged;
    }
    return atlas_route_total;
}

flow memory_effects_jade_jade_score(seed: i32) -> i32 ![]
{
    var atlas_score_total = seed + 6;
    var atlas_score_cursor = 0;
    while atlas_score_cursor < 10 limit Iterations(10) {
        atlas_score_total = atlas_score_total + atlas_score_cursor + 4;
        atlas_score_cursor = atlas_score_cursor + 1;
    }
    if atlas_score_total % 2 == 0 {
        atlas_score_total = atlas_score_total + 19;
    } else {
        atlas_score_total = atlas_score_total - 1;
    }
    var atlas_score_left = atlas_score_total + seed;
    var atlas_score_right = atlas_score_left * 2;
    var atlas_score_merged = atlas_score_right - atlas_score_left;
    if atlas_score_merged > 29 {
        atlas_score_total = atlas_score_total + atlas_score_merged;
    }
    return atlas_score_total;
}

flow memory_effects_jade_jade_finish(seed: i32) -> i32 ![]
{
    var atlas_finish_total = seed - 6;
    var atlas_finish_cursor = 0;
    while atlas_finish_cursor < 9 limit Iterations(9) {
        atlas_finish_total = atlas_finish_total + atlas_finish_cursor + 4;
        atlas_finish_cursor = atlas_finish_cursor + 1;
    }
    if atlas_finish_total % 2 == 0 {
        atlas_finish_total = atlas_finish_total + 19;
    } else {
        atlas_finish_total = atlas_finish_total - 1;
    }
    var atlas_finish_left = atlas_finish_total + seed;
    var atlas_finish_right = atlas_finish_left * 2;
    var atlas_finish_merged = atlas_finish_right - atlas_finish_left;
    if atlas_finish_merged > 29 {
        atlas_finish_total = atlas_finish_total + atlas_finish_merged;
    }
    return atlas_finish_total;
}

flow main(args: Array<string>) -> i32 ![Memory.read<_>]
{
    var jade_seed = 6;
    if args.len() > 0 {
        jade_seed = jade_seed + 1;
    } else {
        jade_seed = jade_seed + 2;
    }
    let jade_result = memory_effects_jade_jade_entry(jade_seed);
    if jade_result > 0 {
        return 0;
    }
    return 1;
}
