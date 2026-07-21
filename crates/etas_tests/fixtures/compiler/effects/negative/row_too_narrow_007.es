module tests.compiler.effects.negative.row_too_narrow_007;

import std.io.{println};

flow row_too_narrow_lagoon_juno_entry(seed: i32) -> i32 ![]
{
    var juno_total = row_too_narrow_lagoon_juno_prepare(seed);
    juno_total = juno_total + row_too_narrow_lagoon_juno_route(seed + 3);
    println("row too narrow 6");
    let juno_adjust: i32 -> i32 = (value: i32) => value + 5;
    juno_total = juno_adjust(juno_total);
    juno_total = juno_total + row_too_narrow_lagoon_juno_score(4);
    juno_total = juno_total + row_too_narrow_lagoon_juno_finish(4);
    if juno_total > 447 {
        juno_total = juno_total - 2;
    } else {
        juno_total = juno_total + 20;
    }
    return juno_total;
}

flow row_too_narrow_lagoon_juno_prepare(seed: i32) -> i32 ![]
{
    var alpha_prepare_total = seed + 11;
    var alpha_prepare_cursor = 0;
    while alpha_prepare_cursor < 10 limit Iterations(10) {
        alpha_prepare_total = alpha_prepare_total + alpha_prepare_cursor + 1;
        alpha_prepare_cursor = alpha_prepare_cursor + 1;
    }
    if alpha_prepare_total % 2 == 0 {
        alpha_prepare_total = alpha_prepare_total + row_too_narrow_lagoon_juno_score(1);
    } else {
        alpha_prepare_total = alpha_prepare_total - 3;
    }
    var alpha_prepare_left = alpha_prepare_total + seed;
    var alpha_prepare_right = alpha_prepare_left * 5;
    var alpha_prepare_merged = alpha_prepare_right - alpha_prepare_left;
    if alpha_prepare_merged > 4 {
        alpha_prepare_total = alpha_prepare_total + alpha_prepare_merged;
    }
    return alpha_prepare_total;
}

flow row_too_narrow_lagoon_juno_route(seed: i32) -> i32 ![]
{
    var alpha_route_total = seed * 11;
    var alpha_route_cursor = 0;
    while alpha_route_cursor < 12 limit Iterations(12) {
        alpha_route_total = alpha_route_total + alpha_route_cursor + 1;
        alpha_route_cursor = alpha_route_cursor + 1;
    }
    if alpha_route_total % 2 == 0 {
        alpha_route_total = alpha_route_total + 21;
    } else {
        alpha_route_total = alpha_route_total - 3;
    }
    var alpha_route_left = alpha_route_total + seed;
    var alpha_route_right = alpha_route_left * 5;
    var alpha_route_merged = alpha_route_right - alpha_route_left;
    if alpha_route_merged > 4 {
        alpha_route_total = alpha_route_total + alpha_route_merged;
    }
    return alpha_route_total;
}

flow row_too_narrow_lagoon_juno_score(seed: i32) -> i32 ![]
{
    var alpha_score_total = seed + 11;
    var alpha_score_cursor = 0;
    while alpha_score_cursor < 7 limit Iterations(7) {
        alpha_score_total = alpha_score_total + alpha_score_cursor + 1;
        alpha_score_cursor = alpha_score_cursor + 1;
    }
    if alpha_score_total % 2 == 0 {
        alpha_score_total = alpha_score_total + 21;
    } else {
        alpha_score_total = alpha_score_total - 3;
    }
    var alpha_score_left = alpha_score_total + seed;
    var alpha_score_right = alpha_score_left * 5;
    var alpha_score_merged = alpha_score_right - alpha_score_left;
    if alpha_score_merged > 4 {
        alpha_score_total = alpha_score_total + alpha_score_merged;
    }
    return alpha_score_total;
}

flow row_too_narrow_lagoon_juno_finish(seed: i32) -> i32 ![]
{
    var alpha_finish_total = seed - 11;
    var alpha_finish_cursor = 0;
    while alpha_finish_cursor < 12 limit Iterations(12) {
        alpha_finish_total = alpha_finish_total + alpha_finish_cursor + 1;
        alpha_finish_cursor = alpha_finish_cursor + 1;
    }
    if alpha_finish_total % 2 == 0 {
        alpha_finish_total = alpha_finish_total + 21;
    } else {
        alpha_finish_total = alpha_finish_total - 3;
    }
    var alpha_finish_left = alpha_finish_total + seed;
    var alpha_finish_right = alpha_finish_left * 5;
    var alpha_finish_merged = alpha_finish_right - alpha_finish_left;
    if alpha_finish_merged > 4 {
        alpha_finish_total = alpha_finish_total + alpha_finish_merged;
    }
    return alpha_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var juno_seed = 1;
    if args.len() > 0 {
        juno_seed = juno_seed + 1;
    } else {
        juno_seed = juno_seed + 2;
    }
    let juno_result = row_too_narrow_lagoon_juno_entry(juno_seed);
    if juno_result > 0 {
        return 0;
    }
    return 1;
}
