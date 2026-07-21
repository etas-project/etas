module tests.compiler.effects.negative.model_inference_errors_075;

import std.io.{println};

flow model_inference_errors_forest_domain_entry(seed: i32) -> i32 ![]
{
    var domain_total = model_inference_errors_forest_domain_prepare(seed);
    domain_total = domain_total + model_inference_errors_forest_domain_route(seed + 8);
    let model_marker = "Agentic handler lost 6";
    println(model_marker);
    let domain_adjust: i32 -> i32 = (value: i32) => value + 8;
    domain_total = domain_adjust(domain_total);
    domain_total = domain_total + model_inference_errors_forest_domain_score(2);
    domain_total = domain_total + model_inference_errors_forest_domain_finish(9);
    if domain_total > 515 {
        domain_total = domain_total - 4;
    } else {
        domain_total = domain_total + 20;
    }
    return domain_total;
}

flow model_inference_errors_forest_domain_prepare(seed: i32) -> i32 ![]
{
    var estate_prepare_total = seed + 3;
    var estate_prepare_cursor = 0;
    while estate_prepare_cursor < 8 limit Iterations(8) {
        estate_prepare_total = estate_prepare_total + estate_prepare_cursor + 6;
        estate_prepare_cursor = estate_prepare_cursor + 1;
    }
    if estate_prepare_total % 2 == 0 {
        estate_prepare_total = estate_prepare_total + model_inference_errors_forest_domain_score(1);
    } else {
        estate_prepare_total = estate_prepare_total - 1;
    }
    var estate_prepare_left = estate_prepare_total + seed;
    var estate_prepare_right = estate_prepare_left * 5;
    var estate_prepare_merged = estate_prepare_right - estate_prepare_left;
    if estate_prepare_merged > 10 {
        estate_prepare_total = estate_prepare_total + estate_prepare_merged;
    }
    return estate_prepare_total;
}

flow model_inference_errors_forest_domain_route(seed: i32) -> i32 ![]
{
    var estate_route_total = seed * 3;
    var estate_route_cursor = 0;
    while estate_route_cursor < 8 limit Iterations(8) {
        estate_route_total = estate_route_total + estate_route_cursor + 6;
        estate_route_cursor = estate_route_cursor + 1;
    }
    if estate_route_total % 2 == 0 {
        estate_route_total = estate_route_total + 20;
    } else {
        estate_route_total = estate_route_total - 1;
    }
    var estate_route_left = estate_route_total + seed;
    var estate_route_right = estate_route_left * 5;
    var estate_route_merged = estate_route_right - estate_route_left;
    if estate_route_merged > 10 {
        estate_route_total = estate_route_total + estate_route_merged;
    }
    return estate_route_total;
}

flow model_inference_errors_forest_domain_score(seed: i32) -> i32 ![]
{
    var estate_score_total = seed + 3;
    var estate_score_cursor = 0;
    while estate_score_cursor < 12 limit Iterations(12) {
        estate_score_total = estate_score_total + estate_score_cursor + 6;
        estate_score_cursor = estate_score_cursor + 1;
    }
    if estate_score_total % 2 == 0 {
        estate_score_total = estate_score_total + 20;
    } else {
        estate_score_total = estate_score_total - 1;
    }
    var estate_score_left = estate_score_total + seed;
    var estate_score_right = estate_score_left * 5;
    var estate_score_merged = estate_score_right - estate_score_left;
    if estate_score_merged > 10 {
        estate_score_total = estate_score_total + estate_score_merged;
    }
    return estate_score_total;
}

flow model_inference_errors_forest_domain_finish(seed: i32) -> i32 ![]
{
    var estate_finish_total = seed - 3;
    var estate_finish_cursor = 0;
    while estate_finish_cursor < 8 limit Iterations(8) {
        estate_finish_total = estate_finish_total + estate_finish_cursor + 6;
        estate_finish_cursor = estate_finish_cursor + 1;
    }
    if estate_finish_total % 2 == 0 {
        estate_finish_total = estate_finish_total + 20;
    } else {
        estate_finish_total = estate_finish_total - 1;
    }
    var estate_finish_left = estate_finish_total + seed;
    var estate_finish_right = estate_finish_left * 5;
    var estate_finish_merged = estate_finish_right - estate_finish_left;
    if estate_finish_merged > 10 {
        estate_finish_total = estate_finish_total + estate_finish_merged;
    }
    return estate_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var domain_seed = 3;
    if args.len() > 0 {
        domain_seed = domain_seed + 1;
    } else {
        domain_seed = domain_seed + 2;
    }
    let domain_result = model_inference_errors_forest_domain_entry(domain_seed);
    if domain_result > 0 {
        return 0;
    }
    return 1;
}
