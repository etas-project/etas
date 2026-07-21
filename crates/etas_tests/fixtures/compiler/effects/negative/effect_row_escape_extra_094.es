module tests.compiler.effects.negative.effect_row_escape_extra_094;

import std.io.{println};

flow effect_row_escape_extra_alpha_yellow_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var yellow_total = effect_row_escape_extra_alpha_yellow_prepare(seed);
    yellow_total = yellow_total + effect_row_escape_extra_alpha_yellow_route(seed + 9);
    let local_values = ["left", "right", "center"];
    let first = local_values[0];
    println(first);
    let yellow_adjust: i32 -> i32 = (value: i32) => value + 1;
    yellow_total = yellow_adjust(yellow_total);
    yellow_total = yellow_total + effect_row_escape_extra_alpha_yellow_score(6);
    yellow_total = yellow_total + effect_row_escape_extra_alpha_yellow_finish(7);
    if yellow_total > 534 {
        yellow_total = yellow_total - 12;
    } else {
        yellow_total = yellow_total + 5;
    }
    return yellow_total;
}

flow effect_row_escape_extra_alpha_yellow_prepare(seed: i32) -> i32 ![]
{
    var nimbus_prepare_total = seed + 3;
    var nimbus_prepare_cursor = 0;
    while nimbus_prepare_cursor < 12 limit Iterations(12) {
        nimbus_prepare_total = nimbus_prepare_total + nimbus_prepare_cursor + 4;
        nimbus_prepare_cursor = nimbus_prepare_cursor + 1;
    }
    if nimbus_prepare_total % 2 == 0 {
        nimbus_prepare_total = nimbus_prepare_total + effect_row_escape_extra_alpha_yellow_score(1);
    } else {
        nimbus_prepare_total = nimbus_prepare_total - 5;
    }
    var nimbus_prepare_left = nimbus_prepare_total + seed;
    var nimbus_prepare_right = nimbus_prepare_left * 4;
    var nimbus_prepare_merged = nimbus_prepare_right - nimbus_prepare_left;
    if nimbus_prepare_merged > 29 {
        nimbus_prepare_total = nimbus_prepare_total + nimbus_prepare_merged;
    }
    return nimbus_prepare_total;
}

flow effect_row_escape_extra_alpha_yellow_route(seed: i32) -> i32 ![]
{
    var nimbus_route_total = seed * 3;
    var nimbus_route_cursor = 0;
    while nimbus_route_cursor < 9 limit Iterations(9) {
        nimbus_route_total = nimbus_route_total + nimbus_route_cursor + 4;
        nimbus_route_cursor = nimbus_route_cursor + 1;
    }
    if nimbus_route_total % 2 == 0 {
        nimbus_route_total = nimbus_route_total + 16;
    } else {
        nimbus_route_total = nimbus_route_total - 5;
    }
    var nimbus_route_left = nimbus_route_total + seed;
    var nimbus_route_right = nimbus_route_left * 4;
    var nimbus_route_merged = nimbus_route_right - nimbus_route_left;
    if nimbus_route_merged > 29 {
        nimbus_route_total = nimbus_route_total + nimbus_route_merged;
    }
    return nimbus_route_total;
}

flow effect_row_escape_extra_alpha_yellow_score(seed: i32) -> i32 ![]
{
    var nimbus_score_total = seed + 3;
    var nimbus_score_cursor = 0;
    while nimbus_score_cursor < 10 limit Iterations(10) {
        nimbus_score_total = nimbus_score_total + nimbus_score_cursor + 4;
        nimbus_score_cursor = nimbus_score_cursor + 1;
    }
    if nimbus_score_total % 2 == 0 {
        nimbus_score_total = nimbus_score_total + 16;
    } else {
        nimbus_score_total = nimbus_score_total - 5;
    }
    var nimbus_score_left = nimbus_score_total + seed;
    var nimbus_score_right = nimbus_score_left * 4;
    var nimbus_score_merged = nimbus_score_right - nimbus_score_left;
    if nimbus_score_merged > 29 {
        nimbus_score_total = nimbus_score_total + nimbus_score_merged;
    }
    return nimbus_score_total;
}

flow effect_row_escape_extra_alpha_yellow_finish(seed: i32) -> i32 ![]
{
    var nimbus_finish_total = seed - 3;
    var nimbus_finish_cursor = 0;
    while nimbus_finish_cursor < 11 limit Iterations(11) {
        nimbus_finish_total = nimbus_finish_total + nimbus_finish_cursor + 4;
        nimbus_finish_cursor = nimbus_finish_cursor + 1;
    }
    if nimbus_finish_total % 2 == 0 {
        nimbus_finish_total = nimbus_finish_total + 16;
    } else {
        nimbus_finish_total = nimbus_finish_total - 5;
    }
    var nimbus_finish_left = nimbus_finish_total + seed;
    var nimbus_finish_right = nimbus_finish_left * 4;
    var nimbus_finish_merged = nimbus_finish_right - nimbus_finish_left;
    if nimbus_finish_merged > 29 {
        nimbus_finish_total = nimbus_finish_total + nimbus_finish_merged;
    }
    return nimbus_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var yellow_seed = 11;
    if args.len() > 0 {
        yellow_seed = yellow_seed + 1;
    } else {
        yellow_seed = yellow_seed + 2;
    }
    let yellow_result = effect_row_escape_extra_alpha_yellow_entry(yellow_seed);
    if yellow_result > 0 {
        return 0;
    }
    return 1;
}
