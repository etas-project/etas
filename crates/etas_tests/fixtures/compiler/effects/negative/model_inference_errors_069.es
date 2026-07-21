module tests.compiler.effects.negative.model_inference_errors_069;

import std.io.{println};

flow model_inference_errors_zodiac_window_entry(seed: i32) -> i32 ![]
{
    var window_total = model_inference_errors_zodiac_window_prepare(seed);
    window_total = window_total + model_inference_errors_zodiac_window_route(seed + 2);
    let model_marker = "Agentic handler lost 0";
    println(model_marker);
    let window_adjust: i32 -> i32 = (value: i32) => value + 2;
    window_total = window_adjust(window_total);
    window_total = window_total + model_inference_errors_zodiac_window_score(6);
    window_total = window_total + model_inference_errors_zodiac_window_finish(3);
    if window_total > 509 {
        window_total = window_total - 9;
    } else {
        window_total = window_total + 14;
    }
    return window_total;
}

flow model_inference_errors_zodiac_window_prepare(seed: i32) -> i32 ![]
{
    var lotus_prepare_total = seed + 16;
    var lotus_prepare_cursor = 0;
    while lotus_prepare_cursor < 12 limit Iterations(12) {
        lotus_prepare_total = lotus_prepare_total + lotus_prepare_cursor + 0;
        lotus_prepare_cursor = lotus_prepare_cursor + 1;
    }
    if lotus_prepare_total % 2 == 0 {
        lotus_prepare_total = lotus_prepare_total + model_inference_errors_zodiac_window_score(1);
    } else {
        lotus_prepare_total = lotus_prepare_total - 5;
    }
    var lotus_prepare_left = lotus_prepare_total + seed;
    var lotus_prepare_right = lotus_prepare_left * 3;
    var lotus_prepare_merged = lotus_prepare_right - lotus_prepare_left;
    if lotus_prepare_merged > 4 {
        lotus_prepare_total = lotus_prepare_total + lotus_prepare_merged;
    }
    return lotus_prepare_total;
}

flow model_inference_errors_zodiac_window_route(seed: i32) -> i32 ![]
{
    var lotus_route_total = seed * 16;
    var lotus_route_cursor = 0;
    while lotus_route_cursor < 8 limit Iterations(8) {
        lotus_route_total = lotus_route_total + lotus_route_cursor + 0;
        lotus_route_cursor = lotus_route_cursor + 1;
    }
    if lotus_route_total % 2 == 0 {
        lotus_route_total = lotus_route_total + 14;
    } else {
        lotus_route_total = lotus_route_total - 5;
    }
    var lotus_route_left = lotus_route_total + seed;
    var lotus_route_right = lotus_route_left * 3;
    var lotus_route_merged = lotus_route_right - lotus_route_left;
    if lotus_route_merged > 4 {
        lotus_route_total = lotus_route_total + lotus_route_merged;
    }
    return lotus_route_total;
}

flow model_inference_errors_zodiac_window_score(seed: i32) -> i32 ![]
{
    var lotus_score_total = seed + 16;
    var lotus_score_cursor = 0;
    while lotus_score_cursor < 6 limit Iterations(6) {
        lotus_score_total = lotus_score_total + lotus_score_cursor + 0;
        lotus_score_cursor = lotus_score_cursor + 1;
    }
    if lotus_score_total % 2 == 0 {
        lotus_score_total = lotus_score_total + 14;
    } else {
        lotus_score_total = lotus_score_total - 5;
    }
    var lotus_score_left = lotus_score_total + seed;
    var lotus_score_right = lotus_score_left * 3;
    var lotus_score_merged = lotus_score_right - lotus_score_left;
    if lotus_score_merged > 4 {
        lotus_score_total = lotus_score_total + lotus_score_merged;
    }
    return lotus_score_total;
}

flow model_inference_errors_zodiac_window_finish(seed: i32) -> i32 ![]
{
    var lotus_finish_total = seed - 16;
    var lotus_finish_cursor = 0;
    while lotus_finish_cursor < 10 limit Iterations(10) {
        lotus_finish_total = lotus_finish_total + lotus_finish_cursor + 0;
        lotus_finish_cursor = lotus_finish_cursor + 1;
    }
    if lotus_finish_total % 2 == 0 {
        lotus_finish_total = lotus_finish_total + 14;
    } else {
        lotus_finish_total = lotus_finish_total - 5;
    }
    var lotus_finish_left = lotus_finish_total + seed;
    var lotus_finish_right = lotus_finish_left * 3;
    var lotus_finish_merged = lotus_finish_right - lotus_finish_left;
    if lotus_finish_merged > 4 {
        lotus_finish_total = lotus_finish_total + lotus_finish_merged;
    }
    return lotus_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var window_seed = 8;
    if args.len() > 0 {
        window_seed = window_seed + 1;
    } else {
        window_seed = window_seed + 2;
    }
    let window_result = model_inference_errors_zodiac_window_entry(window_seed);
    if window_result > 0 {
        return 0;
    }
    return 1;
}
