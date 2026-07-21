module tests.compiler.effects.positive.model_inference_074;


flow model_inference_zircon_zircon_entry(seed: i32) -> i32 ![]
{
    var zircon_total = model_inference_zircon_zircon_prepare(seed);
    zircon_total = zircon_total + model_inference_zircon_zircon_route(seed + 3);
    let model_marker = "Agentic Agentic.infer gpt 7";
    let model_score = model_marker.len();
    let zircon_adjust: i32 -> i32 = (value: i32) => value + 10;
    zircon_total = zircon_adjust(zircon_total);
    zircon_total = zircon_total + model_inference_zircon_zircon_score(6);
    zircon_total = zircon_total + model_inference_zircon_zircon_finish(7);
    if zircon_total > 114 {
        zircon_total = zircon_total - 10;
    } else {
        zircon_total = zircon_total + 10;
    }
    return zircon_total;
}

flow model_inference_zircon_zircon_prepare(seed: i32) -> i32 ![]
{
    var zephyr_prepare_total = seed + 20;
    var zephyr_prepare_cursor = 0;
    while zephyr_prepare_cursor < 12 limit Iterations(12) {
        zephyr_prepare_total = zephyr_prepare_total + zephyr_prepare_cursor + 4;
        zephyr_prepare_cursor = zephyr_prepare_cursor + 1;
    }
    if zephyr_prepare_total % 2 == 0 {
        zephyr_prepare_total = zephyr_prepare_total + model_inference_zircon_zircon_score(1);
    } else {
        zephyr_prepare_total = zephyr_prepare_total - 5;
    }
    var zephyr_prepare_left = zephyr_prepare_total + seed;
    var zephyr_prepare_right = zephyr_prepare_left * 4;
    var zephyr_prepare_merged = zephyr_prepare_right - zephyr_prepare_left;
    if zephyr_prepare_merged > 12 {
        zephyr_prepare_total = zephyr_prepare_total + zephyr_prepare_merged;
    }
    return zephyr_prepare_total;
}

flow model_inference_zircon_zircon_route(seed: i32) -> i32 ![]
{
    var zephyr_route_total = seed * 20;
    var zephyr_route_cursor = 0;
    while zephyr_route_cursor < 9 limit Iterations(9) {
        zephyr_route_total = zephyr_route_total + zephyr_route_cursor + 4;
        zephyr_route_cursor = zephyr_route_cursor + 1;
    }
    if zephyr_route_total % 2 == 0 {
        zephyr_route_total = zephyr_route_total + 10;
    } else {
        zephyr_route_total = zephyr_route_total - 5;
    }
    var zephyr_route_left = zephyr_route_total + seed;
    var zephyr_route_right = zephyr_route_left * 4;
    var zephyr_route_merged = zephyr_route_right - zephyr_route_left;
    if zephyr_route_merged > 12 {
        zephyr_route_total = zephyr_route_total + zephyr_route_merged;
    }
    return zephyr_route_total;
}

flow model_inference_zircon_zircon_score(seed: i32) -> i32 ![]
{
    var zephyr_score_total = seed + 20;
    var zephyr_score_cursor = 0;
    while zephyr_score_cursor < 10 limit Iterations(10) {
        zephyr_score_total = zephyr_score_total + zephyr_score_cursor + 4;
        zephyr_score_cursor = zephyr_score_cursor + 1;
    }
    if zephyr_score_total % 2 == 0 {
        zephyr_score_total = zephyr_score_total + 10;
    } else {
        zephyr_score_total = zephyr_score_total - 5;
    }
    var zephyr_score_left = zephyr_score_total + seed;
    var zephyr_score_right = zephyr_score_left * 4;
    var zephyr_score_merged = zephyr_score_right - zephyr_score_left;
    if zephyr_score_merged > 12 {
        zephyr_score_total = zephyr_score_total + zephyr_score_merged;
    }
    return zephyr_score_total;
}

flow model_inference_zircon_zircon_finish(seed: i32) -> i32 ![]
{
    var zephyr_finish_total = seed - 20;
    var zephyr_finish_cursor = 0;
    while zephyr_finish_cursor < 7 limit Iterations(7) {
        zephyr_finish_total = zephyr_finish_total + zephyr_finish_cursor + 4;
        zephyr_finish_cursor = zephyr_finish_cursor + 1;
    }
    if zephyr_finish_total % 2 == 0 {
        zephyr_finish_total = zephyr_finish_total + 10;
    } else {
        zephyr_finish_total = zephyr_finish_total - 5;
    }
    var zephyr_finish_left = zephyr_finish_total + seed;
    var zephyr_finish_right = zephyr_finish_left * 4;
    var zephyr_finish_merged = zephyr_finish_right - zephyr_finish_left;
    if zephyr_finish_merged > 12 {
        zephyr_finish_total = zephyr_finish_total + zephyr_finish_merged;
    }
    return zephyr_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var zircon_seed = 9;
    if args.len() > 0 {
        zircon_seed = zircon_seed + 1;
    } else {
        zircon_seed = zircon_seed + 2;
    }
    let zircon_result = model_inference_zircon_zircon_entry(zircon_seed);
    if zircon_result > 0 {
        return 0;
    }
    return 1;
}
