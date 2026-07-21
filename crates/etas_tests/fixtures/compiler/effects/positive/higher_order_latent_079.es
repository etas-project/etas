module tests.compiler.effects.positive.higher_order_latent_079;


flow higher_order_latent_engine_engine_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var engine_total = higher_order_latent_engine_engine_prepare(seed);
    engine_total = engine_total + higher_order_latent_engine_engine_route(seed + 8);
    let latent_stage: i32 -> i32 = (value: i32) => value + 5;
    let pipeline_marker = latent_stage(seed);
    let engine_adjust: i32 -> i32 = (value: i32) => value + 2;
    engine_total = engine_adjust(engine_total);
    engine_total = engine_total + higher_order_latent_engine_engine_score(6);
    engine_total = engine_total + higher_order_latent_engine_engine_finish(5);
    if engine_total > 119 {
        engine_total = engine_total - 4;
    } else {
        engine_total = engine_total + 15;
    }
    return engine_total;
}

flow higher_order_latent_engine_engine_prepare(seed: i32) -> i32 ![]
{
    var jade_prepare_total = seed + 6;
    var jade_prepare_cursor = 0;
    while jade_prepare_cursor < 12 limit Iterations(12) {
        jade_prepare_total = jade_prepare_total + jade_prepare_cursor + 2;
        jade_prepare_cursor = jade_prepare_cursor + 1;
    }
    if jade_prepare_total % 2 == 0 {
        jade_prepare_total = jade_prepare_total + higher_order_latent_engine_engine_score(1);
    } else {
        jade_prepare_total = jade_prepare_total - 5;
    }
    var jade_prepare_left = jade_prepare_total + seed;
    var jade_prepare_right = jade_prepare_left * 5;
    var jade_prepare_merged = jade_prepare_right - jade_prepare_left;
    if jade_prepare_merged > 17 {
        jade_prepare_total = jade_prepare_total + jade_prepare_merged;
    }
    return jade_prepare_total;
}

flow higher_order_latent_engine_engine_route(seed: i32) -> i32 ![]
{
    var jade_route_total = seed * 6;
    var jade_route_cursor = 0;
    while jade_route_cursor < 8 limit Iterations(8) {
        jade_route_total = jade_route_total + jade_route_cursor + 2;
        jade_route_cursor = jade_route_cursor + 1;
    }
    if jade_route_total % 2 == 0 {
        jade_route_total = jade_route_total + 15;
    } else {
        jade_route_total = jade_route_total - 5;
    }
    var jade_route_left = jade_route_total + seed;
    var jade_route_right = jade_route_left * 5;
    var jade_route_merged = jade_route_right - jade_route_left;
    if jade_route_merged > 17 {
        jade_route_total = jade_route_total + jade_route_merged;
    }
    return jade_route_total;
}

flow higher_order_latent_engine_engine_score(seed: i32) -> i32 ![]
{
    var jade_score_total = seed + 6;
    var jade_score_cursor = 0;
    while jade_score_cursor < 8 limit Iterations(8) {
        jade_score_total = jade_score_total + jade_score_cursor + 2;
        jade_score_cursor = jade_score_cursor + 1;
    }
    if jade_score_total % 2 == 0 {
        jade_score_total = jade_score_total + 15;
    } else {
        jade_score_total = jade_score_total - 5;
    }
    var jade_score_left = jade_score_total + seed;
    var jade_score_right = jade_score_left * 5;
    var jade_score_merged = jade_score_right - jade_score_left;
    if jade_score_merged > 17 {
        jade_score_total = jade_score_total + jade_score_merged;
    }
    return jade_score_total;
}

flow higher_order_latent_engine_engine_finish(seed: i32) -> i32 ![]
{
    var jade_finish_total = seed - 6;
    var jade_finish_cursor = 0;
    while jade_finish_cursor < 12 limit Iterations(12) {
        jade_finish_total = jade_finish_total + jade_finish_cursor + 2;
        jade_finish_cursor = jade_finish_cursor + 1;
    }
    if jade_finish_total % 2 == 0 {
        jade_finish_total = jade_finish_total + 15;
    } else {
        jade_finish_total = jade_finish_total - 5;
    }
    var jade_finish_left = jade_finish_total + seed;
    var jade_finish_right = jade_finish_left * 5;
    var jade_finish_merged = jade_finish_right - jade_finish_left;
    if jade_finish_merged > 17 {
        jade_finish_total = jade_finish_total + jade_finish_merged;
    }
    return jade_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var engine_seed = 3;
    if args.len() > 0 {
        engine_seed = engine_seed + 1;
    } else {
        engine_seed = engine_seed + 2;
    }
    let engine_result = higher_order_latent_engine_engine_entry(engine_seed);
    if engine_result > 0 {
        return 0;
    }
    return 1;
}
