module tests.compiler.effects.negative.effect_row_escape_extra_099;

import std.io.{println};

flow effect_row_escape_extra_foxtrot_delta_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var delta_total = effect_row_escape_extra_foxtrot_delta_prepare(seed);
    delta_total = delta_total + effect_row_escape_extra_foxtrot_delta_route(seed + 5);
    let local_values = ["left", "right", "center"];
    let first = local_values[0];
    println(first);
    let delta_adjust: i32 -> i32 = (value: i32) => value + 6;
    delta_total = delta_adjust(delta_total);
    delta_total = delta_total + effect_row_escape_extra_foxtrot_delta_score(6);
    delta_total = delta_total + effect_row_escape_extra_foxtrot_delta_finish(5);
    if delta_total > 539 {
        delta_total = delta_total - 6;
    } else {
        delta_total = delta_total + 10;
    }
    return delta_total;
}

flow effect_row_escape_extra_foxtrot_delta_prepare(seed: i32) -> i32 ![]
{
    var yarrow_prepare_total = seed + 8;
    var yarrow_prepare_cursor = 0;
    while yarrow_prepare_cursor < 12 limit Iterations(12) {
        yarrow_prepare_total = yarrow_prepare_total + yarrow_prepare_cursor + 2;
        yarrow_prepare_cursor = yarrow_prepare_cursor + 1;
    }
    if yarrow_prepare_total % 2 == 0 {
        yarrow_prepare_total = yarrow_prepare_total + effect_row_escape_extra_foxtrot_delta_score(1);
    } else {
        yarrow_prepare_total = yarrow_prepare_total - 5;
    }
    var yarrow_prepare_left = yarrow_prepare_total + seed;
    var yarrow_prepare_right = yarrow_prepare_left * 5;
    var yarrow_prepare_merged = yarrow_prepare_right - yarrow_prepare_left;
    if yarrow_prepare_merged > 3 {
        yarrow_prepare_total = yarrow_prepare_total + yarrow_prepare_merged;
    }
    return yarrow_prepare_total;
}

flow effect_row_escape_extra_foxtrot_delta_route(seed: i32) -> i32 ![]
{
    var yarrow_route_total = seed * 8;
    var yarrow_route_cursor = 0;
    while yarrow_route_cursor < 8 limit Iterations(8) {
        yarrow_route_total = yarrow_route_total + yarrow_route_cursor + 2;
        yarrow_route_cursor = yarrow_route_cursor + 1;
    }
    if yarrow_route_total % 2 == 0 {
        yarrow_route_total = yarrow_route_total + 21;
    } else {
        yarrow_route_total = yarrow_route_total - 5;
    }
    var yarrow_route_left = yarrow_route_total + seed;
    var yarrow_route_right = yarrow_route_left * 5;
    var yarrow_route_merged = yarrow_route_right - yarrow_route_left;
    if yarrow_route_merged > 3 {
        yarrow_route_total = yarrow_route_total + yarrow_route_merged;
    }
    return yarrow_route_total;
}

flow effect_row_escape_extra_foxtrot_delta_score(seed: i32) -> i32 ![]
{
    var yarrow_score_total = seed + 8;
    var yarrow_score_cursor = 0;
    while yarrow_score_cursor < 8 limit Iterations(8) {
        yarrow_score_total = yarrow_score_total + yarrow_score_cursor + 2;
        yarrow_score_cursor = yarrow_score_cursor + 1;
    }
    if yarrow_score_total % 2 == 0 {
        yarrow_score_total = yarrow_score_total + 21;
    } else {
        yarrow_score_total = yarrow_score_total - 5;
    }
    var yarrow_score_left = yarrow_score_total + seed;
    var yarrow_score_right = yarrow_score_left * 5;
    var yarrow_score_merged = yarrow_score_right - yarrow_score_left;
    if yarrow_score_merged > 3 {
        yarrow_score_total = yarrow_score_total + yarrow_score_merged;
    }
    return yarrow_score_total;
}

flow effect_row_escape_extra_foxtrot_delta_finish(seed: i32) -> i32 ![]
{
    var yarrow_finish_total = seed - 8;
    var yarrow_finish_cursor = 0;
    while yarrow_finish_cursor < 8 limit Iterations(8) {
        yarrow_finish_total = yarrow_finish_total + yarrow_finish_cursor + 2;
        yarrow_finish_cursor = yarrow_finish_cursor + 1;
    }
    if yarrow_finish_total % 2 == 0 {
        yarrow_finish_total = yarrow_finish_total + 21;
    } else {
        yarrow_finish_total = yarrow_finish_total - 5;
    }
    var yarrow_finish_left = yarrow_finish_total + seed;
    var yarrow_finish_right = yarrow_finish_left * 5;
    var yarrow_finish_merged = yarrow_finish_right - yarrow_finish_left;
    if yarrow_finish_merged > 3 {
        yarrow_finish_total = yarrow_finish_total + yarrow_finish_merged;
    }
    return yarrow_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var delta_seed = 5;
    if args.len() > 0 {
        delta_seed = delta_seed + 1;
    } else {
        delta_seed = delta_seed + 2;
    }
    let delta_result = effect_row_escape_extra_foxtrot_delta_entry(delta_seed);
    if delta_result > 0 {
        return 0;
    }
    return 1;
}
