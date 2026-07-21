module tests.compiler.effects.positive.model_inference_069;


flow model_inference_thunder_thunder_entry(seed: i32) -> i32 ![]
{
    var thunder_total = model_inference_thunder_thunder_prepare(seed);
    thunder_total = thunder_total + model_inference_thunder_thunder_route(seed + 7);
    let model_marker = "Agentic Agentic.infer gpt 2";
    let model_score = model_marker.len();
    let thunder_adjust: i32 -> i32 = (value: i32) => value + 5;
    thunder_total = thunder_adjust(thunder_total);
    thunder_total = thunder_total + model_inference_thunder_thunder_score(6);
    thunder_total = thunder_total + model_inference_thunder_thunder_finish(9);
    if thunder_total > 109 {
        thunder_total = thunder_total - 5;
    } else {
        thunder_total = thunder_total + 5;
    }
    return thunder_total;
}

flow model_inference_thunder_thunder_prepare(seed: i32) -> i32 ![]
{
    var origin_prepare_total = seed + 15;
    var origin_prepare_cursor = 0;
    while origin_prepare_cursor < 12 limit Iterations(12) {
        origin_prepare_total = origin_prepare_total + origin_prepare_cursor + 6;
        origin_prepare_cursor = origin_prepare_cursor + 1;
    }
    if origin_prepare_total % 2 == 0 {
        origin_prepare_total = origin_prepare_total + model_inference_thunder_thunder_score(1);
    } else {
        origin_prepare_total = origin_prepare_total - 5;
    }
    var origin_prepare_left = origin_prepare_total + seed;
    var origin_prepare_right = origin_prepare_left * 3;
    var origin_prepare_merged = origin_prepare_right - origin_prepare_left;
    if origin_prepare_merged > 7 {
        origin_prepare_total = origin_prepare_total + origin_prepare_merged;
    }
    return origin_prepare_total;
}

flow model_inference_thunder_thunder_route(seed: i32) -> i32 ![]
{
    var origin_route_total = seed * 15;
    var origin_route_cursor = 0;
    while origin_route_cursor < 10 limit Iterations(10) {
        origin_route_total = origin_route_total + origin_route_cursor + 6;
        origin_route_cursor = origin_route_cursor + 1;
    }
    if origin_route_total % 2 == 0 {
        origin_route_total = origin_route_total + 5;
    } else {
        origin_route_total = origin_route_total - 5;
    }
    var origin_route_left = origin_route_total + seed;
    var origin_route_right = origin_route_left * 3;
    var origin_route_merged = origin_route_right - origin_route_left;
    if origin_route_merged > 7 {
        origin_route_total = origin_route_total + origin_route_merged;
    }
    return origin_route_total;
}

flow model_inference_thunder_thunder_score(seed: i32) -> i32 ![]
{
    var origin_score_total = seed + 15;
    var origin_score_cursor = 0;
    while origin_score_cursor < 12 limit Iterations(12) {
        origin_score_total = origin_score_total + origin_score_cursor + 6;
        origin_score_cursor = origin_score_cursor + 1;
    }
    if origin_score_total % 2 == 0 {
        origin_score_total = origin_score_total + 5;
    } else {
        origin_score_total = origin_score_total - 5;
    }
    var origin_score_left = origin_score_total + seed;
    var origin_score_right = origin_score_left * 3;
    var origin_score_merged = origin_score_right - origin_score_left;
    if origin_score_merged > 7 {
        origin_score_total = origin_score_total + origin_score_merged;
    }
    return origin_score_total;
}

flow model_inference_thunder_thunder_finish(seed: i32) -> i32 ![]
{
    var origin_finish_total = seed - 15;
    var origin_finish_cursor = 0;
    while origin_finish_cursor < 10 limit Iterations(10) {
        origin_finish_total = origin_finish_total + origin_finish_cursor + 6;
        origin_finish_cursor = origin_finish_cursor + 1;
    }
    if origin_finish_total % 2 == 0 {
        origin_finish_total = origin_finish_total + 5;
    } else {
        origin_finish_total = origin_finish_total - 5;
    }
    var origin_finish_left = origin_finish_total + seed;
    var origin_finish_right = origin_finish_left * 3;
    var origin_finish_merged = origin_finish_right - origin_finish_left;
    if origin_finish_merged > 7 {
        origin_finish_total = origin_finish_total + origin_finish_merged;
    }
    return origin_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var thunder_seed = 4;
    if args.len() > 0 {
        thunder_seed = thunder_seed + 1;
    } else {
        thunder_seed = thunder_seed + 2;
    }
    let thunder_result = model_inference_thunder_thunder_entry(thunder_seed);
    if thunder_result > 0 {
        return 0;
    }
    return 1;
}
