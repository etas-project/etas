module tests.compiler.effects.positive.memory_effects_065;

import std.memory.{region};

flow memory_effects_orbit_orbit_entry(seed: i32) -> i32 ![Memory.read<_>]
{
    var orbit_total = memory_effects_orbit_orbit_prepare(seed);
    orbit_total = orbit_total + memory_effects_orbit_orbit_route(seed + 3);
    let memory_marker = "Memory.read Memory.write Store.get Store.put 6";
    let memory_score = memory_marker.len();
    let orbit_adjust: i32 -> i32 = (value: i32) => value + 1;
    orbit_total = orbit_adjust(orbit_total);
    orbit_total = orbit_total + memory_effects_orbit_orbit_score(2);
    orbit_total = orbit_total + memory_effects_orbit_orbit_finish(5);
    if orbit_total > 105 {
        orbit_total = orbit_total - 12;
    } else {
        orbit_total = orbit_total + 18;
    }
    return orbit_total;
}

flow memory_effects_orbit_orbit_prepare(seed: i32) -> i32 ![]
{
    var lantern_prepare_total = seed + 11;
    var lantern_prepare_cursor = 0;
    while lantern_prepare_cursor < 8 limit Iterations(8) {
        lantern_prepare_total = lantern_prepare_total + lantern_prepare_cursor + 2;
        lantern_prepare_cursor = lantern_prepare_cursor + 1;
    }
    if lantern_prepare_total % 2 == 0 {
        lantern_prepare_total = lantern_prepare_total + memory_effects_orbit_orbit_score(1);
    } else {
        lantern_prepare_total = lantern_prepare_total - 1;
    }
    var lantern_prepare_left = lantern_prepare_total + seed;
    var lantern_prepare_right = lantern_prepare_left * 3;
    var lantern_prepare_merged = lantern_prepare_right - lantern_prepare_left;
    if lantern_prepare_merged > 3 {
        lantern_prepare_total = lantern_prepare_total + lantern_prepare_merged;
    }
    return lantern_prepare_total;
}

flow memory_effects_orbit_orbit_route(seed: i32) -> i32 ![]
{
    var lantern_route_total = seed * 11;
    var lantern_route_cursor = 0;
    while lantern_route_cursor < 12 limit Iterations(12) {
        lantern_route_total = lantern_route_total + lantern_route_cursor + 2;
        lantern_route_cursor = lantern_route_cursor + 1;
    }
    if lantern_route_total % 2 == 0 {
        lantern_route_total = lantern_route_total + 24;
    } else {
        lantern_route_total = lantern_route_total - 1;
    }
    var lantern_route_left = lantern_route_total + seed;
    var lantern_route_right = lantern_route_left * 3;
    var lantern_route_merged = lantern_route_right - lantern_route_left;
    if lantern_route_merged > 3 {
        lantern_route_total = lantern_route_total + lantern_route_merged;
    }
    return lantern_route_total;
}

flow memory_effects_orbit_orbit_score(seed: i32) -> i32 ![]
{
    var lantern_score_total = seed + 11;
    var lantern_score_cursor = 0;
    while lantern_score_cursor < 8 limit Iterations(8) {
        lantern_score_total = lantern_score_total + lantern_score_cursor + 2;
        lantern_score_cursor = lantern_score_cursor + 1;
    }
    if lantern_score_total % 2 == 0 {
        lantern_score_total = lantern_score_total + 24;
    } else {
        lantern_score_total = lantern_score_total - 1;
    }
    var lantern_score_left = lantern_score_total + seed;
    var lantern_score_right = lantern_score_left * 3;
    var lantern_score_merged = lantern_score_right - lantern_score_left;
    if lantern_score_merged > 3 {
        lantern_score_total = lantern_score_total + lantern_score_merged;
    }
    return lantern_score_total;
}

flow memory_effects_orbit_orbit_finish(seed: i32) -> i32 ![]
{
    var lantern_finish_total = seed - 11;
    var lantern_finish_cursor = 0;
    while lantern_finish_cursor < 6 limit Iterations(6) {
        lantern_finish_total = lantern_finish_total + lantern_finish_cursor + 2;
        lantern_finish_cursor = lantern_finish_cursor + 1;
    }
    if lantern_finish_total % 2 == 0 {
        lantern_finish_total = lantern_finish_total + 24;
    } else {
        lantern_finish_total = lantern_finish_total - 1;
    }
    var lantern_finish_left = lantern_finish_total + seed;
    var lantern_finish_right = lantern_finish_left * 3;
    var lantern_finish_merged = lantern_finish_right - lantern_finish_left;
    if lantern_finish_merged > 3 {
        lantern_finish_total = lantern_finish_total + lantern_finish_merged;
    }
    return lantern_finish_total;
}

flow main(args: Array<string>) -> i32 ![Memory.read<_>]
{
    var orbit_seed = 11;
    if args.len() > 0 {
        orbit_seed = orbit_seed + 1;
    } else {
        orbit_seed = orbit_seed + 2;
    }
    let orbit_result = memory_effects_orbit_orbit_entry(orbit_seed);
    if orbit_result > 0 {
        return 0;
    }
    return 1;
}
