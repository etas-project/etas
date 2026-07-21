module tests.compiler.effects.positive.model_inference_067;


flow model_inference_ridge_ridge_entry(seed: i32) -> i32 ![]
{
    var ridge_total = model_inference_ridge_ridge_prepare(seed);
    ridge_total = ridge_total + model_inference_ridge_ridge_route(seed + 5);
    let model_marker = "Agentic Agentic.infer gpt 0";
    let model_score = model_marker.len();
    let ridge_adjust: i32 -> i32 = (value: i32) => value + 3;
    ridge_total = ridge_adjust(ridge_total);
    ridge_total = ridge_total + model_inference_ridge_ridge_score(4);
    ridge_total = ridge_total + model_inference_ridge_ridge_finish(7);
    if ridge_total > 107 {
        ridge_total = ridge_total - 3;
    } else {
        ridge_total = ridge_total + 20;
    }
    return ridge_total;
}

flow model_inference_ridge_ridge_prepare(seed: i32) -> i32 ![]
{
    var apex_prepare_total = seed + 13;
    var apex_prepare_cursor = 0;
    while apex_prepare_cursor < 10 limit Iterations(10) {
        apex_prepare_total = apex_prepare_total + apex_prepare_cursor + 4;
        apex_prepare_cursor = apex_prepare_cursor + 1;
    }
    if apex_prepare_total % 2 == 0 {
        apex_prepare_total = apex_prepare_total + model_inference_ridge_ridge_score(1);
    } else {
        apex_prepare_total = apex_prepare_total - 3;
    }
    var apex_prepare_left = apex_prepare_total + seed;
    var apex_prepare_right = apex_prepare_left * 5;
    var apex_prepare_merged = apex_prepare_right - apex_prepare_left;
    if apex_prepare_merged > 5 {
        apex_prepare_total = apex_prepare_total + apex_prepare_merged;
    }
    return apex_prepare_total;
}

flow model_inference_ridge_ridge_route(seed: i32) -> i32 ![]
{
    var apex_route_total = seed * 13;
    var apex_route_cursor = 0;
    while apex_route_cursor < 8 limit Iterations(8) {
        apex_route_total = apex_route_total + apex_route_cursor + 4;
        apex_route_cursor = apex_route_cursor + 1;
    }
    if apex_route_total % 2 == 0 {
        apex_route_total = apex_route_total + 26;
    } else {
        apex_route_total = apex_route_total - 3;
    }
    var apex_route_left = apex_route_total + seed;
    var apex_route_right = apex_route_left * 5;
    var apex_route_merged = apex_route_right - apex_route_left;
    if apex_route_merged > 5 {
        apex_route_total = apex_route_total + apex_route_merged;
    }
    return apex_route_total;
}

flow model_inference_ridge_ridge_score(seed: i32) -> i32 ![]
{
    var apex_score_total = seed + 13;
    var apex_score_cursor = 0;
    while apex_score_cursor < 10 limit Iterations(10) {
        apex_score_total = apex_score_total + apex_score_cursor + 4;
        apex_score_cursor = apex_score_cursor + 1;
    }
    if apex_score_total % 2 == 0 {
        apex_score_total = apex_score_total + 26;
    } else {
        apex_score_total = apex_score_total - 3;
    }
    var apex_score_left = apex_score_total + seed;
    var apex_score_right = apex_score_left * 5;
    var apex_score_merged = apex_score_right - apex_score_left;
    if apex_score_merged > 5 {
        apex_score_total = apex_score_total + apex_score_merged;
    }
    return apex_score_total;
}

flow model_inference_ridge_ridge_finish(seed: i32) -> i32 ![]
{
    var apex_finish_total = seed - 13;
    var apex_finish_cursor = 0;
    while apex_finish_cursor < 8 limit Iterations(8) {
        apex_finish_total = apex_finish_total + apex_finish_cursor + 4;
        apex_finish_cursor = apex_finish_cursor + 1;
    }
    if apex_finish_total % 2 == 0 {
        apex_finish_total = apex_finish_total + 26;
    } else {
        apex_finish_total = apex_finish_total - 3;
    }
    var apex_finish_left = apex_finish_total + seed;
    var apex_finish_right = apex_finish_left * 5;
    var apex_finish_merged = apex_finish_right - apex_finish_left;
    if apex_finish_merged > 5 {
        apex_finish_total = apex_finish_total + apex_finish_merged;
    }
    return apex_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var ridge_seed = 2;
    if args.len() > 0 {
        ridge_seed = ridge_seed + 1;
    } else {
        ridge_seed = ridge_seed + 2;
    }
    let ridge_result = model_inference_ridge_ridge_entry(ridge_seed);
    if ridge_result > 0 {
        return 0;
    }
    return 1;
}
