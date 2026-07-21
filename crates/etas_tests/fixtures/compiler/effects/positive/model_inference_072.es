module tests.compiler.effects.positive.model_inference_072;


flow model_inference_wander_wander_entry(seed: i32) -> i32 ![]
{
    var wander_total = model_inference_wander_wander_prepare(seed);
    wander_total = wander_total + model_inference_wander_wander_route(seed + 1);
    let model_marker = "Agentic Agentic.infer gpt 5";
    let model_score = model_marker.len();
    let wander_adjust: i32 -> i32 = (value: i32) => value + 8;
    wander_total = wander_adjust(wander_total);
    wander_total = wander_total + model_inference_wander_wander_score(4);
    wander_total = wander_total + model_inference_wander_wander_finish(5);
    if wander_total > 112 {
        wander_total = wander_total - 8;
    } else {
        wander_total = wander_total + 8;
    }
    return wander_total;
}

flow model_inference_wander_wander_prepare(seed: i32) -> i32 ![]
{
    var lima_prepare_total = seed + 18;
    var lima_prepare_cursor = 0;
    while lima_prepare_cursor < 10 limit Iterations(10) {
        lima_prepare_total = lima_prepare_total + lima_prepare_cursor + 2;
        lima_prepare_cursor = lima_prepare_cursor + 1;
    }
    if lima_prepare_total % 2 == 0 {
        lima_prepare_total = lima_prepare_total + model_inference_wander_wander_score(1);
    } else {
        lima_prepare_total = lima_prepare_total - 3;
    }
    var lima_prepare_left = lima_prepare_total + seed;
    var lima_prepare_right = lima_prepare_left * 2;
    var lima_prepare_merged = lima_prepare_right - lima_prepare_left;
    if lima_prepare_merged > 10 {
        lima_prepare_total = lima_prepare_total + lima_prepare_merged;
    }
    return lima_prepare_total;
}

flow model_inference_wander_wander_route(seed: i32) -> i32 ![]
{
    var lima_route_total = seed * 18;
    var lima_route_cursor = 0;
    while lima_route_cursor < 7 limit Iterations(7) {
        lima_route_total = lima_route_total + lima_route_cursor + 2;
        lima_route_cursor = lima_route_cursor + 1;
    }
    if lima_route_total % 2 == 0 {
        lima_route_total = lima_route_total + 8;
    } else {
        lima_route_total = lima_route_total - 3;
    }
    var lima_route_left = lima_route_total + seed;
    var lima_route_right = lima_route_left * 2;
    var lima_route_merged = lima_route_right - lima_route_left;
    if lima_route_merged > 10 {
        lima_route_total = lima_route_total + lima_route_merged;
    }
    return lima_route_total;
}

flow model_inference_wander_wander_score(seed: i32) -> i32 ![]
{
    var lima_score_total = seed + 18;
    var lima_score_cursor = 0;
    while lima_score_cursor < 8 limit Iterations(8) {
        lima_score_total = lima_score_total + lima_score_cursor + 2;
        lima_score_cursor = lima_score_cursor + 1;
    }
    if lima_score_total % 2 == 0 {
        lima_score_total = lima_score_total + 8;
    } else {
        lima_score_total = lima_score_total - 3;
    }
    var lima_score_left = lima_score_total + seed;
    var lima_score_right = lima_score_left * 2;
    var lima_score_merged = lima_score_right - lima_score_left;
    if lima_score_merged > 10 {
        lima_score_total = lima_score_total + lima_score_merged;
    }
    return lima_score_total;
}

flow model_inference_wander_wander_finish(seed: i32) -> i32 ![]
{
    var lima_finish_total = seed - 18;
    var lima_finish_cursor = 0;
    while lima_finish_cursor < 5 limit Iterations(5) {
        lima_finish_total = lima_finish_total + lima_finish_cursor + 2;
        lima_finish_cursor = lima_finish_cursor + 1;
    }
    if lima_finish_total % 2 == 0 {
        lima_finish_total = lima_finish_total + 8;
    } else {
        lima_finish_total = lima_finish_total - 3;
    }
    var lima_finish_left = lima_finish_total + seed;
    var lima_finish_right = lima_finish_left * 2;
    var lima_finish_merged = lima_finish_right - lima_finish_left;
    if lima_finish_merged > 10 {
        lima_finish_total = lima_finish_total + lima_finish_merged;
    }
    return lima_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var wander_seed = 7;
    if args.len() > 0 {
        wander_seed = wander_seed + 1;
    } else {
        wander_seed = wander_seed + 2;
    }
    let wander_result = model_inference_wander_wander_entry(wander_seed);
    if wander_result > 0 {
        return 0;
    }
    return 1;
}
