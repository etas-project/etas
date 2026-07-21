module tests.compiler.effects.positive.model_inference_071;


flow model_inference_vector_vector_entry(seed: i32) -> i32 ![]
{
    var vector_total = model_inference_vector_vector_prepare(seed);
    vector_total = vector_total + model_inference_vector_vector_route(seed + 9);
    let model_marker = "Agentic Agentic.infer gpt 4";
    let model_score = model_marker.len();
    let vector_adjust: i32 -> i32 = (value: i32) => value + 7;
    vector_total = vector_adjust(vector_total);
    vector_total = vector_total + model_inference_vector_vector_score(3);
    vector_total = vector_total + model_inference_vector_vector_finish(4);
    if vector_total > 111 {
        vector_total = vector_total - 7;
    } else {
        vector_total = vector_total + 7;
    }
    return vector_total;
}

flow model_inference_vector_vector_prepare(seed: i32) -> i32 ![]
{
    var echo_prepare_total = seed + 17;
    var echo_prepare_cursor = 0;
    while echo_prepare_cursor < 9 limit Iterations(9) {
        echo_prepare_total = echo_prepare_total + echo_prepare_cursor + 1;
        echo_prepare_cursor = echo_prepare_cursor + 1;
    }
    if echo_prepare_total % 2 == 0 {
        echo_prepare_total = echo_prepare_total + model_inference_vector_vector_score(1);
    } else {
        echo_prepare_total = echo_prepare_total - 2;
    }
    var echo_prepare_left = echo_prepare_total + seed;
    var echo_prepare_right = echo_prepare_left * 5;
    var echo_prepare_merged = echo_prepare_right - echo_prepare_left;
    if echo_prepare_merged > 9 {
        echo_prepare_total = echo_prepare_total + echo_prepare_merged;
    }
    return echo_prepare_total;
}

flow model_inference_vector_vector_route(seed: i32) -> i32 ![]
{
    var echo_route_total = seed * 17;
    var echo_route_cursor = 0;
    while echo_route_cursor < 12 limit Iterations(12) {
        echo_route_total = echo_route_total + echo_route_cursor + 1;
        echo_route_cursor = echo_route_cursor + 1;
    }
    if echo_route_total % 2 == 0 {
        echo_route_total = echo_route_total + 7;
    } else {
        echo_route_total = echo_route_total - 2;
    }
    var echo_route_left = echo_route_total + seed;
    var echo_route_right = echo_route_left * 5;
    var echo_route_merged = echo_route_right - echo_route_left;
    if echo_route_merged > 9 {
        echo_route_total = echo_route_total + echo_route_merged;
    }
    return echo_route_total;
}

flow model_inference_vector_vector_score(seed: i32) -> i32 ![]
{
    var echo_score_total = seed + 17;
    var echo_score_cursor = 0;
    while echo_score_cursor < 7 limit Iterations(7) {
        echo_score_total = echo_score_total + echo_score_cursor + 1;
        echo_score_cursor = echo_score_cursor + 1;
    }
    if echo_score_total % 2 == 0 {
        echo_score_total = echo_score_total + 7;
    } else {
        echo_score_total = echo_score_total - 2;
    }
    var echo_score_left = echo_score_total + seed;
    var echo_score_right = echo_score_left * 5;
    var echo_score_merged = echo_score_right - echo_score_left;
    if echo_score_merged > 9 {
        echo_score_total = echo_score_total + echo_score_merged;
    }
    return echo_score_total;
}

flow model_inference_vector_vector_finish(seed: i32) -> i32 ![]
{
    var echo_finish_total = seed - 17;
    var echo_finish_cursor = 0;
    while echo_finish_cursor < 12 limit Iterations(12) {
        echo_finish_total = echo_finish_total + echo_finish_cursor + 1;
        echo_finish_cursor = echo_finish_cursor + 1;
    }
    if echo_finish_total % 2 == 0 {
        echo_finish_total = echo_finish_total + 7;
    } else {
        echo_finish_total = echo_finish_total - 2;
    }
    var echo_finish_left = echo_finish_total + seed;
    var echo_finish_right = echo_finish_left * 5;
    var echo_finish_merged = echo_finish_right - echo_finish_left;
    if echo_finish_merged > 9 {
        echo_finish_total = echo_finish_total + echo_finish_merged;
    }
    return echo_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var vector_seed = 6;
    if args.len() > 0 {
        vector_seed = vector_seed + 1;
    } else {
        vector_seed = vector_seed + 2;
    }
    let vector_result = model_inference_vector_vector_entry(vector_seed);
    if vector_result > 0 {
        return 0;
    }
    return 1;
}
