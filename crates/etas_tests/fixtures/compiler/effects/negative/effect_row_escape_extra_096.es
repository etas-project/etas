module tests.compiler.effects.negative.effect_row_escape_extra_096;

import std.io.{println};

flow effect_row_escape_extra_charlie_alpha_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var alpha_total = effect_row_escape_extra_charlie_alpha_prepare(seed);
    alpha_total = alpha_total + effect_row_escape_extra_charlie_alpha_route(seed + 2);
    let local_values = ["left", "right", "center"];
    let first = local_values[0];
    println(first);
    let alpha_adjust: i32 -> i32 = (value: i32) => value + 3;
    alpha_total = alpha_adjust(alpha_total);
    alpha_total = alpha_total + effect_row_escape_extra_charlie_alpha_score(3);
    alpha_total = alpha_total + effect_row_escape_extra_charlie_alpha_finish(9);
    if alpha_total > 536 {
        alpha_total = alpha_total - 3;
    } else {
        alpha_total = alpha_total + 7;
    }
    return alpha_total;
}

flow effect_row_escape_extra_charlie_alpha_prepare(seed: i32) -> i32 ![]
{
    var delta_prepare_total = seed + 5;
    var delta_prepare_cursor = 0;
    while delta_prepare_cursor < 9 limit Iterations(9) {
        delta_prepare_total = delta_prepare_total + delta_prepare_cursor + 6;
        delta_prepare_cursor = delta_prepare_cursor + 1;
    }
    if delta_prepare_total % 2 == 0 {
        delta_prepare_total = delta_prepare_total + effect_row_escape_extra_charlie_alpha_score(1);
    } else {
        delta_prepare_total = delta_prepare_total - 2;
    }
    var delta_prepare_left = delta_prepare_total + seed;
    var delta_prepare_right = delta_prepare_left * 2;
    var delta_prepare_merged = delta_prepare_right - delta_prepare_left;
    if delta_prepare_merged > 0 {
        delta_prepare_total = delta_prepare_total + delta_prepare_merged;
    }
    return delta_prepare_total;
}

flow effect_row_escape_extra_charlie_alpha_route(seed: i32) -> i32 ![]
{
    var delta_route_total = seed * 5;
    var delta_route_cursor = 0;
    while delta_route_cursor < 11 limit Iterations(11) {
        delta_route_total = delta_route_total + delta_route_cursor + 6;
        delta_route_cursor = delta_route_cursor + 1;
    }
    if delta_route_total % 2 == 0 {
        delta_route_total = delta_route_total + 18;
    } else {
        delta_route_total = delta_route_total - 2;
    }
    var delta_route_left = delta_route_total + seed;
    var delta_route_right = delta_route_left * 2;
    var delta_route_merged = delta_route_right - delta_route_left;
    if delta_route_merged > 0 {
        delta_route_total = delta_route_total + delta_route_merged;
    }
    return delta_route_total;
}

flow effect_row_escape_extra_charlie_alpha_score(seed: i32) -> i32 ![]
{
    var delta_score_total = seed + 5;
    var delta_score_cursor = 0;
    while delta_score_cursor < 12 limit Iterations(12) {
        delta_score_total = delta_score_total + delta_score_cursor + 6;
        delta_score_cursor = delta_score_cursor + 1;
    }
    if delta_score_total % 2 == 0 {
        delta_score_total = delta_score_total + 18;
    } else {
        delta_score_total = delta_score_total - 2;
    }
    var delta_score_left = delta_score_total + seed;
    var delta_score_right = delta_score_left * 2;
    var delta_score_merged = delta_score_right - delta_score_left;
    if delta_score_merged > 0 {
        delta_score_total = delta_score_total + delta_score_merged;
    }
    return delta_score_total;
}

flow effect_row_escape_extra_charlie_alpha_finish(seed: i32) -> i32 ![]
{
    var delta_finish_total = seed - 5;
    var delta_finish_cursor = 0;
    while delta_finish_cursor < 5 limit Iterations(5) {
        delta_finish_total = delta_finish_total + delta_finish_cursor + 6;
        delta_finish_cursor = delta_finish_cursor + 1;
    }
    if delta_finish_total % 2 == 0 {
        delta_finish_total = delta_finish_total + 18;
    } else {
        delta_finish_total = delta_finish_total - 2;
    }
    var delta_finish_left = delta_finish_total + seed;
    var delta_finish_right = delta_finish_left * 2;
    var delta_finish_merged = delta_finish_right - delta_finish_left;
    if delta_finish_merged > 0 {
        delta_finish_total = delta_finish_total + delta_finish_merged;
    }
    return delta_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var alpha_seed = 2;
    if args.len() > 0 {
        alpha_seed = alpha_seed + 1;
    } else {
        alpha_seed = alpha_seed + 2;
    }
    let alpha_result = effect_row_escape_extra_charlie_alpha_entry(alpha_seed);
    if alpha_result > 0 {
        return 0;
    }
    return 1;
}
