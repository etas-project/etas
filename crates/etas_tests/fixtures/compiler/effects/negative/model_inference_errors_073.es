module tests.compiler.effects.negative.model_inference_errors_073;

import std.io.{println};

flow model_inference_errors_domain_bridge_entry(seed: i32) -> i32 ![]
{
    var bridge_total = model_inference_errors_domain_bridge_prepare(seed);
    bridge_total = bridge_total + model_inference_errors_domain_bridge_route(seed + 6);
    let model_marker = "Agentic handler lost 4";
    println(model_marker);
    let bridge_adjust: i32 -> i32 = (value: i32) => value + 6;
    bridge_total = bridge_adjust(bridge_total);
    bridge_total = bridge_total + model_inference_errors_domain_bridge_score(5);
    bridge_total = bridge_total + model_inference_errors_domain_bridge_finish(7);
    if bridge_total > 513 {
        bridge_total = bridge_total - 2;
    } else {
        bridge_total = bridge_total + 18;
    }
    return bridge_total;
}

flow model_inference_errors_domain_bridge_prepare(seed: i32) -> i32 ![]
{
    var parity_prepare_total = seed + 20;
    var parity_prepare_cursor = 0;
    while parity_prepare_cursor < 11 limit Iterations(11) {
        parity_prepare_total = parity_prepare_total + parity_prepare_cursor + 4;
        parity_prepare_cursor = parity_prepare_cursor + 1;
    }
    if parity_prepare_total % 2 == 0 {
        parity_prepare_total = parity_prepare_total + model_inference_errors_domain_bridge_score(1);
    } else {
        parity_prepare_total = parity_prepare_total - 4;
    }
    var parity_prepare_left = parity_prepare_total + seed;
    var parity_prepare_right = parity_prepare_left * 3;
    var parity_prepare_merged = parity_prepare_right - parity_prepare_left;
    if parity_prepare_merged > 8 {
        parity_prepare_total = parity_prepare_total + parity_prepare_merged;
    }
    return parity_prepare_total;
}

flow model_inference_errors_domain_bridge_route(seed: i32) -> i32 ![]
{
    var parity_route_total = seed * 20;
    var parity_route_cursor = 0;
    while parity_route_cursor < 12 limit Iterations(12) {
        parity_route_total = parity_route_total + parity_route_cursor + 4;
        parity_route_cursor = parity_route_cursor + 1;
    }
    if parity_route_total % 2 == 0 {
        parity_route_total = parity_route_total + 18;
    } else {
        parity_route_total = parity_route_total - 4;
    }
    var parity_route_left = parity_route_total + seed;
    var parity_route_right = parity_route_left * 3;
    var parity_route_merged = parity_route_right - parity_route_left;
    if parity_route_merged > 8 {
        parity_route_total = parity_route_total + parity_route_merged;
    }
    return parity_route_total;
}

flow model_inference_errors_domain_bridge_score(seed: i32) -> i32 ![]
{
    var parity_score_total = seed + 20;
    var parity_score_cursor = 0;
    while parity_score_cursor < 10 limit Iterations(10) {
        parity_score_total = parity_score_total + parity_score_cursor + 4;
        parity_score_cursor = parity_score_cursor + 1;
    }
    if parity_score_total % 2 == 0 {
        parity_score_total = parity_score_total + 18;
    } else {
        parity_score_total = parity_score_total - 4;
    }
    var parity_score_left = parity_score_total + seed;
    var parity_score_right = parity_score_left * 3;
    var parity_score_merged = parity_score_right - parity_score_left;
    if parity_score_merged > 8 {
        parity_score_total = parity_score_total + parity_score_merged;
    }
    return parity_score_total;
}

flow model_inference_errors_domain_bridge_finish(seed: i32) -> i32 ![]
{
    var parity_finish_total = seed - 20;
    var parity_finish_cursor = 0;
    while parity_finish_cursor < 6 limit Iterations(6) {
        parity_finish_total = parity_finish_total + parity_finish_cursor + 4;
        parity_finish_cursor = parity_finish_cursor + 1;
    }
    if parity_finish_total % 2 == 0 {
        parity_finish_total = parity_finish_total + 18;
    } else {
        parity_finish_total = parity_finish_total - 4;
    }
    var parity_finish_left = parity_finish_total + seed;
    var parity_finish_right = parity_finish_left * 3;
    var parity_finish_merged = parity_finish_right - parity_finish_left;
    if parity_finish_merged > 8 {
        parity_finish_total = parity_finish_total + parity_finish_merged;
    }
    return parity_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var bridge_seed = 1;
    if args.len() > 0 {
        bridge_seed = bridge_seed + 1;
    } else {
        bridge_seed = bridge_seed + 2;
    }
    let bridge_result = model_inference_errors_domain_bridge_entry(bridge_seed);
    if bridge_result > 0 {
        return 0;
    }
    return 1;
}
