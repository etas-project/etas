module tests.compiler.effects.negative.model_inference_errors_074;

import std.io.{println};

flow model_inference_errors_estate_crystal_entry(seed: i32) -> i32 ![]
{
    var crystal_total = model_inference_errors_estate_crystal_prepare(seed);
    crystal_total = crystal_total + model_inference_errors_estate_crystal_route(seed + 7);
    let model_marker = "Agentic handler lost 5";
    println(model_marker);
    let crystal_adjust: i32 -> i32 = (value: i32) => value + 7;
    crystal_total = crystal_adjust(crystal_total);
    crystal_total = crystal_total + model_inference_errors_estate_crystal_score(6);
    crystal_total = crystal_total + model_inference_errors_estate_crystal_finish(8);
    if crystal_total > 514 {
        crystal_total = crystal_total - 3;
    } else {
        crystal_total = crystal_total + 19;
    }
    return crystal_total;
}

flow model_inference_errors_estate_crystal_prepare(seed: i32) -> i32 ![]
{
    var window_prepare_total = seed + 21;
    var window_prepare_cursor = 0;
    while window_prepare_cursor < 12 limit Iterations(12) {
        window_prepare_total = window_prepare_total + window_prepare_cursor + 5;
        window_prepare_cursor = window_prepare_cursor + 1;
    }
    if window_prepare_total % 2 == 0 {
        window_prepare_total = window_prepare_total + model_inference_errors_estate_crystal_score(1);
    } else {
        window_prepare_total = window_prepare_total - 5;
    }
    var window_prepare_left = window_prepare_total + seed;
    var window_prepare_right = window_prepare_left * 4;
    var window_prepare_merged = window_prepare_right - window_prepare_left;
    if window_prepare_merged > 9 {
        window_prepare_total = window_prepare_total + window_prepare_merged;
    }
    return window_prepare_total;
}

flow model_inference_errors_estate_crystal_route(seed: i32) -> i32 ![]
{
    var window_route_total = seed * 21;
    var window_route_cursor = 0;
    while window_route_cursor < 7 limit Iterations(7) {
        window_route_total = window_route_total + window_route_cursor + 5;
        window_route_cursor = window_route_cursor + 1;
    }
    if window_route_total % 2 == 0 {
        window_route_total = window_route_total + 19;
    } else {
        window_route_total = window_route_total - 5;
    }
    var window_route_left = window_route_total + seed;
    var window_route_right = window_route_left * 4;
    var window_route_merged = window_route_right - window_route_left;
    if window_route_merged > 9 {
        window_route_total = window_route_total + window_route_merged;
    }
    return window_route_total;
}

flow model_inference_errors_estate_crystal_score(seed: i32) -> i32 ![]
{
    var window_score_total = seed + 21;
    var window_score_cursor = 0;
    while window_score_cursor < 11 limit Iterations(11) {
        window_score_total = window_score_total + window_score_cursor + 5;
        window_score_cursor = window_score_cursor + 1;
    }
    if window_score_total % 2 == 0 {
        window_score_total = window_score_total + 19;
    } else {
        window_score_total = window_score_total - 5;
    }
    var window_score_left = window_score_total + seed;
    var window_score_right = window_score_left * 4;
    var window_score_merged = window_score_right - window_score_left;
    if window_score_merged > 9 {
        window_score_total = window_score_total + window_score_merged;
    }
    return window_score_total;
}

flow model_inference_errors_estate_crystal_finish(seed: i32) -> i32 ![]
{
    var window_finish_total = seed - 21;
    var window_finish_cursor = 0;
    while window_finish_cursor < 7 limit Iterations(7) {
        window_finish_total = window_finish_total + window_finish_cursor + 5;
        window_finish_cursor = window_finish_cursor + 1;
    }
    if window_finish_total % 2 == 0 {
        window_finish_total = window_finish_total + 19;
    } else {
        window_finish_total = window_finish_total - 5;
    }
    var window_finish_left = window_finish_total + seed;
    var window_finish_right = window_finish_left * 4;
    var window_finish_merged = window_finish_right - window_finish_left;
    if window_finish_merged > 9 {
        window_finish_total = window_finish_total + window_finish_merged;
    }
    return window_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var crystal_seed = 2;
    if args.len() > 0 {
        crystal_seed = crystal_seed + 1;
    } else {
        crystal_seed = crystal_seed + 2;
    }
    let crystal_result = model_inference_errors_estate_crystal_entry(crystal_seed);
    if crystal_result > 0 {
        return 0;
    }
    return 1;
}
