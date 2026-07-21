module tests.compiler.effects.negative.model_inference_errors_076;

import std.io.{println};

flow model_inference_errors_galaxy_estate_entry(seed: i32) -> i32 ![]
{
    var estate_total = model_inference_errors_galaxy_estate_prepare(seed);
    estate_total = estate_total + model_inference_errors_galaxy_estate_route(seed + 9);
    let model_marker = "Agentic handler lost 7";
    println(model_marker);
    let estate_adjust: i32 -> i32 = (value: i32) => value + 9;
    estate_total = estate_adjust(estate_total);
    estate_total = estate_total + model_inference_errors_galaxy_estate_score(3);
    estate_total = estate_total + model_inference_errors_galaxy_estate_finish(3);
    if estate_total > 516 {
        estate_total = estate_total - 5;
    } else {
        estate_total = estate_total + 4;
    }
    return estate_total;
}

flow model_inference_errors_galaxy_estate_prepare(seed: i32) -> i32 ![]
{
    var legend_prepare_total = seed + 4;
    var legend_prepare_cursor = 0;
    while legend_prepare_cursor < 9 limit Iterations(9) {
        legend_prepare_total = legend_prepare_total + legend_prepare_cursor + 0;
        legend_prepare_cursor = legend_prepare_cursor + 1;
    }
    if legend_prepare_total % 2 == 0 {
        legend_prepare_total = legend_prepare_total + model_inference_errors_galaxy_estate_score(1);
    } else {
        legend_prepare_total = legend_prepare_total - 2;
    }
    var legend_prepare_left = legend_prepare_total + seed;
    var legend_prepare_right = legend_prepare_left * 2;
    var legend_prepare_merged = legend_prepare_right - legend_prepare_left;
    if legend_prepare_merged > 11 {
        legend_prepare_total = legend_prepare_total + legend_prepare_merged;
    }
    return legend_prepare_total;
}

flow model_inference_errors_galaxy_estate_route(seed: i32) -> i32 ![]
{
    var legend_route_total = seed * 4;
    var legend_route_cursor = 0;
    while legend_route_cursor < 9 limit Iterations(9) {
        legend_route_total = legend_route_total + legend_route_cursor + 0;
        legend_route_cursor = legend_route_cursor + 1;
    }
    if legend_route_total % 2 == 0 {
        legend_route_total = legend_route_total + 21;
    } else {
        legend_route_total = legend_route_total - 2;
    }
    var legend_route_left = legend_route_total + seed;
    var legend_route_right = legend_route_left * 2;
    var legend_route_merged = legend_route_right - legend_route_left;
    if legend_route_merged > 11 {
        legend_route_total = legend_route_total + legend_route_merged;
    }
    return legend_route_total;
}

flow model_inference_errors_galaxy_estate_score(seed: i32) -> i32 ![]
{
    var legend_score_total = seed + 4;
    var legend_score_cursor = 0;
    while legend_score_cursor < 6 limit Iterations(6) {
        legend_score_total = legend_score_total + legend_score_cursor + 0;
        legend_score_cursor = legend_score_cursor + 1;
    }
    if legend_score_total % 2 == 0 {
        legend_score_total = legend_score_total + 21;
    } else {
        legend_score_total = legend_score_total - 2;
    }
    var legend_score_left = legend_score_total + seed;
    var legend_score_right = legend_score_left * 2;
    var legend_score_merged = legend_score_right - legend_score_left;
    if legend_score_merged > 11 {
        legend_score_total = legend_score_total + legend_score_merged;
    }
    return legend_score_total;
}

flow model_inference_errors_galaxy_estate_finish(seed: i32) -> i32 ![]
{
    var legend_finish_total = seed - 4;
    var legend_finish_cursor = 0;
    while legend_finish_cursor < 9 limit Iterations(9) {
        legend_finish_total = legend_finish_total + legend_finish_cursor + 0;
        legend_finish_cursor = legend_finish_cursor + 1;
    }
    if legend_finish_total % 2 == 0 {
        legend_finish_total = legend_finish_total + 21;
    } else {
        legend_finish_total = legend_finish_total - 2;
    }
    var legend_finish_left = legend_finish_total + seed;
    var legend_finish_right = legend_finish_left * 2;
    var legend_finish_merged = legend_finish_right - legend_finish_left;
    if legend_finish_merged > 11 {
        legend_finish_total = legend_finish_total + legend_finish_merged;
    }
    return legend_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var estate_seed = 4;
    if args.len() > 0 {
        estate_seed = estate_seed + 1;
    } else {
        estate_seed = estate_seed + 2;
    }
    let estate_result = model_inference_errors_galaxy_estate_entry(estate_seed);
    if estate_result > 0 {
        return 0;
    }
    return 1;
}
