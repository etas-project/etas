module tests.compiler.effects.positive.model_inference_068;


flow model_inference_signal_signal_entry(seed: i32) -> i32 ![]
{
    var signal_total = model_inference_signal_signal_prepare(seed);
    signal_total = signal_total + model_inference_signal_signal_route(seed + 6);
    let model_marker = "Agentic Agentic.infer gpt 1";
    let model_score = model_marker.len();
    let signal_adjust: i32 -> i32 = (value: i32) => value + 4;
    signal_total = signal_adjust(signal_total);
    signal_total = signal_total + model_inference_signal_signal_score(5);
    signal_total = signal_total + model_inference_signal_signal_finish(8);
    if signal_total > 108 {
        signal_total = signal_total - 4;
    } else {
        signal_total = signal_total + 4;
    }
    return signal_total;
}

flow model_inference_signal_signal_prepare(seed: i32) -> i32 ![]
{
    var horizon_prepare_total = seed + 14;
    var horizon_prepare_cursor = 0;
    while horizon_prepare_cursor < 11 limit Iterations(11) {
        horizon_prepare_total = horizon_prepare_total + horizon_prepare_cursor + 5;
        horizon_prepare_cursor = horizon_prepare_cursor + 1;
    }
    if horizon_prepare_total % 2 == 0 {
        horizon_prepare_total = horizon_prepare_total + model_inference_signal_signal_score(1);
    } else {
        horizon_prepare_total = horizon_prepare_total - 4;
    }
    var horizon_prepare_left = horizon_prepare_total + seed;
    var horizon_prepare_right = horizon_prepare_left * 2;
    var horizon_prepare_merged = horizon_prepare_right - horizon_prepare_left;
    if horizon_prepare_merged > 6 {
        horizon_prepare_total = horizon_prepare_total + horizon_prepare_merged;
    }
    return horizon_prepare_total;
}

flow model_inference_signal_signal_route(seed: i32) -> i32 ![]
{
    var horizon_route_total = seed * 14;
    var horizon_route_cursor = 0;
    while horizon_route_cursor < 9 limit Iterations(9) {
        horizon_route_total = horizon_route_total + horizon_route_cursor + 5;
        horizon_route_cursor = horizon_route_cursor + 1;
    }
    if horizon_route_total % 2 == 0 {
        horizon_route_total = horizon_route_total + 27;
    } else {
        horizon_route_total = horizon_route_total - 4;
    }
    var horizon_route_left = horizon_route_total + seed;
    var horizon_route_right = horizon_route_left * 2;
    var horizon_route_merged = horizon_route_right - horizon_route_left;
    if horizon_route_merged > 6 {
        horizon_route_total = horizon_route_total + horizon_route_merged;
    }
    return horizon_route_total;
}

flow model_inference_signal_signal_score(seed: i32) -> i32 ![]
{
    var horizon_score_total = seed + 14;
    var horizon_score_cursor = 0;
    while horizon_score_cursor < 11 limit Iterations(11) {
        horizon_score_total = horizon_score_total + horizon_score_cursor + 5;
        horizon_score_cursor = horizon_score_cursor + 1;
    }
    if horizon_score_total % 2 == 0 {
        horizon_score_total = horizon_score_total + 27;
    } else {
        horizon_score_total = horizon_score_total - 4;
    }
    var horizon_score_left = horizon_score_total + seed;
    var horizon_score_right = horizon_score_left * 2;
    var horizon_score_merged = horizon_score_right - horizon_score_left;
    if horizon_score_merged > 6 {
        horizon_score_total = horizon_score_total + horizon_score_merged;
    }
    return horizon_score_total;
}

flow model_inference_signal_signal_finish(seed: i32) -> i32 ![]
{
    var horizon_finish_total = seed - 14;
    var horizon_finish_cursor = 0;
    while horizon_finish_cursor < 9 limit Iterations(9) {
        horizon_finish_total = horizon_finish_total + horizon_finish_cursor + 5;
        horizon_finish_cursor = horizon_finish_cursor + 1;
    }
    if horizon_finish_total % 2 == 0 {
        horizon_finish_total = horizon_finish_total + 27;
    } else {
        horizon_finish_total = horizon_finish_total - 4;
    }
    var horizon_finish_left = horizon_finish_total + seed;
    var horizon_finish_right = horizon_finish_left * 2;
    var horizon_finish_merged = horizon_finish_right - horizon_finish_left;
    if horizon_finish_merged > 6 {
        horizon_finish_total = horizon_finish_total + horizon_finish_merged;
    }
    return horizon_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var signal_seed = 3;
    if args.len() > 0 {
        signal_seed = signal_seed + 1;
    } else {
        signal_seed = signal_seed + 2;
    }
    let signal_result = model_inference_signal_signal_entry(signal_seed);
    if signal_result > 0 {
        return 0;
    }
    return 1;
}
