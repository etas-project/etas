module tests.compiler.effects.positive.model_inference_070;


flow model_inference_unity_unity_entry(seed: i32) -> i32 ![]
{
    var unity_total = model_inference_unity_unity_prepare(seed);
    unity_total = unity_total + model_inference_unity_unity_route(seed + 8);
    let model_marker = "Agentic Agentic.infer gpt 3";
    let model_score = model_marker.len();
    let unity_adjust: i32 -> i32 = (value: i32) => value + 6;
    unity_total = unity_adjust(unity_total);
    unity_total = unity_total + model_inference_unity_unity_score(2);
    unity_total = unity_total + model_inference_unity_unity_finish(3);
    if unity_total > 110 {
        unity_total = unity_total - 6;
    } else {
        unity_total = unity_total + 6;
    }
    return unity_total;
}

flow model_inference_unity_unity_prepare(seed: i32) -> i32 ![]
{
    var western_prepare_total = seed + 16;
    var western_prepare_cursor = 0;
    while western_prepare_cursor < 8 limit Iterations(8) {
        western_prepare_total = western_prepare_total + western_prepare_cursor + 0;
        western_prepare_cursor = western_prepare_cursor + 1;
    }
    if western_prepare_total % 2 == 0 {
        western_prepare_total = western_prepare_total + model_inference_unity_unity_score(1);
    } else {
        western_prepare_total = western_prepare_total - 1;
    }
    var western_prepare_left = western_prepare_total + seed;
    var western_prepare_right = western_prepare_left * 4;
    var western_prepare_merged = western_prepare_right - western_prepare_left;
    if western_prepare_merged > 8 {
        western_prepare_total = western_prepare_total + western_prepare_merged;
    }
    return western_prepare_total;
}

flow model_inference_unity_unity_route(seed: i32) -> i32 ![]
{
    var western_route_total = seed * 16;
    var western_route_cursor = 0;
    while western_route_cursor < 11 limit Iterations(11) {
        western_route_total = western_route_total + western_route_cursor + 0;
        western_route_cursor = western_route_cursor + 1;
    }
    if western_route_total % 2 == 0 {
        western_route_total = western_route_total + 6;
    } else {
        western_route_total = western_route_total - 1;
    }
    var western_route_left = western_route_total + seed;
    var western_route_right = western_route_left * 4;
    var western_route_merged = western_route_right - western_route_left;
    if western_route_merged > 8 {
        western_route_total = western_route_total + western_route_merged;
    }
    return western_route_total;
}

flow model_inference_unity_unity_score(seed: i32) -> i32 ![]
{
    var western_score_total = seed + 16;
    var western_score_cursor = 0;
    while western_score_cursor < 6 limit Iterations(6) {
        western_score_total = western_score_total + western_score_cursor + 0;
        western_score_cursor = western_score_cursor + 1;
    }
    if western_score_total % 2 == 0 {
        western_score_total = western_score_total + 6;
    } else {
        western_score_total = western_score_total - 1;
    }
    var western_score_left = western_score_total + seed;
    var western_score_right = western_score_left * 4;
    var western_score_merged = western_score_right - western_score_left;
    if western_score_merged > 8 {
        western_score_total = western_score_total + western_score_merged;
    }
    return western_score_total;
}

flow model_inference_unity_unity_finish(seed: i32) -> i32 ![]
{
    var western_finish_total = seed - 16;
    var western_finish_cursor = 0;
    while western_finish_cursor < 11 limit Iterations(11) {
        western_finish_total = western_finish_total + western_finish_cursor + 0;
        western_finish_cursor = western_finish_cursor + 1;
    }
    if western_finish_total % 2 == 0 {
        western_finish_total = western_finish_total + 6;
    } else {
        western_finish_total = western_finish_total - 1;
    }
    var western_finish_left = western_finish_total + seed;
    var western_finish_right = western_finish_left * 4;
    var western_finish_merged = western_finish_right - western_finish_left;
    if western_finish_merged > 8 {
        western_finish_total = western_finish_total + western_finish_merged;
    }
    return western_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var unity_seed = 5;
    if args.len() > 0 {
        unity_seed = unity_seed + 1;
    } else {
        unity_seed = unity_seed + 2;
    }
    let unity_result = model_inference_unity_unity_entry(unity_seed);
    if unity_result > 0 {
        return 0;
    }
    return 1;
}
