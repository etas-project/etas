module tests.compiler.effects.negative.model_inference_errors_071;

import std.io.{println};

flow model_inference_errors_bridge_zodiac_entry(seed: i32) -> i32 ![]
{
    var zodiac_total = model_inference_errors_bridge_zodiac_prepare(seed);
    zodiac_total = zodiac_total + model_inference_errors_bridge_zodiac_route(seed + 4);
    let model_marker = "Agentic handler lost 2";
    println(model_marker);
    let zodiac_adjust: i32 -> i32 = (value: i32) => value + 4;
    zodiac_total = zodiac_adjust(zodiac_total);
    zodiac_total = zodiac_total + model_inference_errors_bridge_zodiac_score(3);
    zodiac_total = zodiac_total + model_inference_errors_bridge_zodiac_finish(5);
    if zodiac_total > 511 {
        zodiac_total = zodiac_total - 11;
    } else {
        zodiac_total = zodiac_total + 16;
    }
    return zodiac_total;
}

flow model_inference_errors_bridge_zodiac_prepare(seed: i32) -> i32 ![]
{
    var beacon_prepare_total = seed + 18;
    var beacon_prepare_cursor = 0;
    while beacon_prepare_cursor < 9 limit Iterations(9) {
        beacon_prepare_total = beacon_prepare_total + beacon_prepare_cursor + 2;
        beacon_prepare_cursor = beacon_prepare_cursor + 1;
    }
    if beacon_prepare_total % 2 == 0 {
        beacon_prepare_total = beacon_prepare_total + model_inference_errors_bridge_zodiac_score(1);
    } else {
        beacon_prepare_total = beacon_prepare_total - 2;
    }
    var beacon_prepare_left = beacon_prepare_total + seed;
    var beacon_prepare_right = beacon_prepare_left * 5;
    var beacon_prepare_merged = beacon_prepare_right - beacon_prepare_left;
    if beacon_prepare_merged > 6 {
        beacon_prepare_total = beacon_prepare_total + beacon_prepare_merged;
    }
    return beacon_prepare_total;
}

flow model_inference_errors_bridge_zodiac_route(seed: i32) -> i32 ![]
{
    var beacon_route_total = seed * 18;
    var beacon_route_cursor = 0;
    while beacon_route_cursor < 10 limit Iterations(10) {
        beacon_route_total = beacon_route_total + beacon_route_cursor + 2;
        beacon_route_cursor = beacon_route_cursor + 1;
    }
    if beacon_route_total % 2 == 0 {
        beacon_route_total = beacon_route_total + 16;
    } else {
        beacon_route_total = beacon_route_total - 2;
    }
    var beacon_route_left = beacon_route_total + seed;
    var beacon_route_right = beacon_route_left * 5;
    var beacon_route_merged = beacon_route_right - beacon_route_left;
    if beacon_route_merged > 6 {
        beacon_route_total = beacon_route_total + beacon_route_merged;
    }
    return beacon_route_total;
}

flow model_inference_errors_bridge_zodiac_score(seed: i32) -> i32 ![]
{
    var beacon_score_total = seed + 18;
    var beacon_score_cursor = 0;
    while beacon_score_cursor < 8 limit Iterations(8) {
        beacon_score_total = beacon_score_total + beacon_score_cursor + 2;
        beacon_score_cursor = beacon_score_cursor + 1;
    }
    if beacon_score_total % 2 == 0 {
        beacon_score_total = beacon_score_total + 16;
    } else {
        beacon_score_total = beacon_score_total - 2;
    }
    var beacon_score_left = beacon_score_total + seed;
    var beacon_score_right = beacon_score_left * 5;
    var beacon_score_merged = beacon_score_right - beacon_score_left;
    if beacon_score_merged > 6 {
        beacon_score_total = beacon_score_total + beacon_score_merged;
    }
    return beacon_score_total;
}

flow model_inference_errors_bridge_zodiac_finish(seed: i32) -> i32 ![]
{
    var beacon_finish_total = seed - 18;
    var beacon_finish_cursor = 0;
    while beacon_finish_cursor < 12 limit Iterations(12) {
        beacon_finish_total = beacon_finish_total + beacon_finish_cursor + 2;
        beacon_finish_cursor = beacon_finish_cursor + 1;
    }
    if beacon_finish_total % 2 == 0 {
        beacon_finish_total = beacon_finish_total + 16;
    } else {
        beacon_finish_total = beacon_finish_total - 2;
    }
    var beacon_finish_left = beacon_finish_total + seed;
    var beacon_finish_right = beacon_finish_left * 5;
    var beacon_finish_merged = beacon_finish_right - beacon_finish_left;
    if beacon_finish_merged > 6 {
        beacon_finish_total = beacon_finish_total + beacon_finish_merged;
    }
    return beacon_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var zodiac_seed = 10;
    if args.len() > 0 {
        zodiac_seed = zodiac_seed + 1;
    } else {
        zodiac_seed = zodiac_seed + 2;
    }
    let zodiac_result = model_inference_errors_bridge_zodiac_entry(zodiac_seed);
    if zodiac_result > 0 {
        return 0;
    }
    return 1;
}
