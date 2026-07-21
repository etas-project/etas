module tests.compiler.effects.positive.memory_effects_066;

import std.memory.{region};

flow memory_effects_pulse_pulse_entry(seed: i32) -> i32 ![Memory.read<_>]
{
    var pulse_total = memory_effects_pulse_pulse_prepare(seed);
    pulse_total = pulse_total + memory_effects_pulse_pulse_route(seed + 4);
    let memory_marker = "Memory.read Memory.write Store.get Store.put 7";
    let memory_score = memory_marker.len();
    let pulse_adjust: i32 -> i32 = (value: i32) => value + 2;
    pulse_total = pulse_adjust(pulse_total);
    pulse_total = pulse_total + memory_effects_pulse_pulse_score(3);
    pulse_total = pulse_total + memory_effects_pulse_pulse_finish(6);
    if pulse_total > 106 {
        pulse_total = pulse_total - 2;
    } else {
        pulse_total = pulse_total + 19;
    }
    return pulse_total;
}

flow memory_effects_pulse_pulse_prepare(seed: i32) -> i32 ![]
{
    var saffron_prepare_total = seed + 12;
    var saffron_prepare_cursor = 0;
    while saffron_prepare_cursor < 9 limit Iterations(9) {
        saffron_prepare_total = saffron_prepare_total + saffron_prepare_cursor + 3;
        saffron_prepare_cursor = saffron_prepare_cursor + 1;
    }
    if saffron_prepare_total % 2 == 0 {
        saffron_prepare_total = saffron_prepare_total + memory_effects_pulse_pulse_score(1);
    } else {
        saffron_prepare_total = saffron_prepare_total - 2;
    }
    var saffron_prepare_left = saffron_prepare_total + seed;
    var saffron_prepare_right = saffron_prepare_left * 4;
    var saffron_prepare_merged = saffron_prepare_right - saffron_prepare_left;
    if saffron_prepare_merged > 4 {
        saffron_prepare_total = saffron_prepare_total + saffron_prepare_merged;
    }
    return saffron_prepare_total;
}

flow memory_effects_pulse_pulse_route(seed: i32) -> i32 ![]
{
    var saffron_route_total = seed * 12;
    var saffron_route_cursor = 0;
    while saffron_route_cursor < 7 limit Iterations(7) {
        saffron_route_total = saffron_route_total + saffron_route_cursor + 3;
        saffron_route_cursor = saffron_route_cursor + 1;
    }
    if saffron_route_total % 2 == 0 {
        saffron_route_total = saffron_route_total + 25;
    } else {
        saffron_route_total = saffron_route_total - 2;
    }
    var saffron_route_left = saffron_route_total + seed;
    var saffron_route_right = saffron_route_left * 4;
    var saffron_route_merged = saffron_route_right - saffron_route_left;
    if saffron_route_merged > 4 {
        saffron_route_total = saffron_route_total + saffron_route_merged;
    }
    return saffron_route_total;
}

flow memory_effects_pulse_pulse_score(seed: i32) -> i32 ![]
{
    var saffron_score_total = seed + 12;
    var saffron_score_cursor = 0;
    while saffron_score_cursor < 9 limit Iterations(9) {
        saffron_score_total = saffron_score_total + saffron_score_cursor + 3;
        saffron_score_cursor = saffron_score_cursor + 1;
    }
    if saffron_score_total % 2 == 0 {
        saffron_score_total = saffron_score_total + 25;
    } else {
        saffron_score_total = saffron_score_total - 2;
    }
    var saffron_score_left = saffron_score_total + seed;
    var saffron_score_right = saffron_score_left * 4;
    var saffron_score_merged = saffron_score_right - saffron_score_left;
    if saffron_score_merged > 4 {
        saffron_score_total = saffron_score_total + saffron_score_merged;
    }
    return saffron_score_total;
}

flow memory_effects_pulse_pulse_finish(seed: i32) -> i32 ![]
{
    var saffron_finish_total = seed - 12;
    var saffron_finish_cursor = 0;
    while saffron_finish_cursor < 7 limit Iterations(7) {
        saffron_finish_total = saffron_finish_total + saffron_finish_cursor + 3;
        saffron_finish_cursor = saffron_finish_cursor + 1;
    }
    if saffron_finish_total % 2 == 0 {
        saffron_finish_total = saffron_finish_total + 25;
    } else {
        saffron_finish_total = saffron_finish_total - 2;
    }
    var saffron_finish_left = saffron_finish_total + seed;
    var saffron_finish_right = saffron_finish_left * 4;
    var saffron_finish_merged = saffron_finish_right - saffron_finish_left;
    if saffron_finish_merged > 4 {
        saffron_finish_total = saffron_finish_total + saffron_finish_merged;
    }
    return saffron_finish_total;
}

flow main(args: Array<string>) -> i32 ![Memory.read<_>]
{
    var pulse_seed = 1;
    if args.len() > 0 {
        pulse_seed = pulse_seed + 1;
    } else {
        pulse_seed = pulse_seed + 2;
    }
    let pulse_result = memory_effects_pulse_pulse_entry(pulse_seed);
    if pulse_result > 0 {
        return 0;
    }
    return 1;
}
