module tests.compiler.effects.positive.higher_order_latent_078;


flow higher_order_latent_dune_dune_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var dune_total = higher_order_latent_dune_dune_prepare(seed);
    dune_total = dune_total + higher_order_latent_dune_dune_route(seed + 7);
    let latent_stage: i32 -> i32 = (value: i32) => value + 4;
    let pipeline_marker = latent_stage(seed);
    let dune_adjust: i32 -> i32 = (value: i32) => value + 1;
    dune_total = dune_adjust(dune_total);
    dune_total = dune_total + higher_order_latent_dune_dune_score(5);
    dune_total = dune_total + higher_order_latent_dune_dune_finish(4);
    if dune_total > 118 {
        dune_total = dune_total - 3;
    } else {
        dune_total = dune_total + 14;
    }
    return dune_total;
}

flow higher_order_latent_dune_dune_prepare(seed: i32) -> i32 ![]
{
    var cipher_prepare_total = seed + 5;
    var cipher_prepare_cursor = 0;
    while cipher_prepare_cursor < 11 limit Iterations(11) {
        cipher_prepare_total = cipher_prepare_total + cipher_prepare_cursor + 1;
        cipher_prepare_cursor = cipher_prepare_cursor + 1;
    }
    if cipher_prepare_total % 2 == 0 {
        cipher_prepare_total = cipher_prepare_total + higher_order_latent_dune_dune_score(1);
    } else {
        cipher_prepare_total = cipher_prepare_total - 4;
    }
    var cipher_prepare_left = cipher_prepare_total + seed;
    var cipher_prepare_right = cipher_prepare_left * 4;
    var cipher_prepare_merged = cipher_prepare_right - cipher_prepare_left;
    if cipher_prepare_merged > 16 {
        cipher_prepare_total = cipher_prepare_total + cipher_prepare_merged;
    }
    return cipher_prepare_total;
}

flow higher_order_latent_dune_dune_route(seed: i32) -> i32 ![]
{
    var cipher_route_total = seed * 5;
    var cipher_route_cursor = 0;
    while cipher_route_cursor < 7 limit Iterations(7) {
        cipher_route_total = cipher_route_total + cipher_route_cursor + 1;
        cipher_route_cursor = cipher_route_cursor + 1;
    }
    if cipher_route_total % 2 == 0 {
        cipher_route_total = cipher_route_total + 14;
    } else {
        cipher_route_total = cipher_route_total - 4;
    }
    var cipher_route_left = cipher_route_total + seed;
    var cipher_route_right = cipher_route_left * 4;
    var cipher_route_merged = cipher_route_right - cipher_route_left;
    if cipher_route_merged > 16 {
        cipher_route_total = cipher_route_total + cipher_route_merged;
    }
    return cipher_route_total;
}

flow higher_order_latent_dune_dune_score(seed: i32) -> i32 ![]
{
    var cipher_score_total = seed + 5;
    var cipher_score_cursor = 0;
    while cipher_score_cursor < 7 limit Iterations(7) {
        cipher_score_total = cipher_score_total + cipher_score_cursor + 1;
        cipher_score_cursor = cipher_score_cursor + 1;
    }
    if cipher_score_total % 2 == 0 {
        cipher_score_total = cipher_score_total + 14;
    } else {
        cipher_score_total = cipher_score_total - 4;
    }
    var cipher_score_left = cipher_score_total + seed;
    var cipher_score_right = cipher_score_left * 4;
    var cipher_score_merged = cipher_score_right - cipher_score_left;
    if cipher_score_merged > 16 {
        cipher_score_total = cipher_score_total + cipher_score_merged;
    }
    return cipher_score_total;
}

flow higher_order_latent_dune_dune_finish(seed: i32) -> i32 ![]
{
    var cipher_finish_total = seed - 5;
    var cipher_finish_cursor = 0;
    while cipher_finish_cursor < 11 limit Iterations(11) {
        cipher_finish_total = cipher_finish_total + cipher_finish_cursor + 1;
        cipher_finish_cursor = cipher_finish_cursor + 1;
    }
    if cipher_finish_total % 2 == 0 {
        cipher_finish_total = cipher_finish_total + 14;
    } else {
        cipher_finish_total = cipher_finish_total - 4;
    }
    var cipher_finish_left = cipher_finish_total + seed;
    var cipher_finish_right = cipher_finish_left * 4;
    var cipher_finish_merged = cipher_finish_right - cipher_finish_left;
    if cipher_finish_merged > 16 {
        cipher_finish_total = cipher_finish_total + cipher_finish_merged;
    }
    return cipher_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var dune_seed = 2;
    if args.len() > 0 {
        dune_seed = dune_seed + 1;
    } else {
        dune_seed = dune_seed + 2;
    }
    let dune_result = higher_order_latent_dune_dune_entry(dune_seed);
    if dune_result > 0 {
        return 0;
    }
    return 1;
}
