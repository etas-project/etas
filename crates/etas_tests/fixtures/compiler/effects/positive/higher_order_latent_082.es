module tests.compiler.effects.positive.higher_order_latent_082;


flow higher_order_latent_haven_haven_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var haven_total = higher_order_latent_haven_haven_prepare(seed);
    haven_total = haven_total + higher_order_latent_haven_haven_route(seed + 2);
    let latent_stage: i32 -> i32 = (value: i32) => value + 8;
    let pipeline_marker = latent_stage(seed);
    let haven_adjust: i32 -> i32 = (value: i32) => value + 5;
    haven_total = haven_adjust(haven_total);
    haven_total = haven_total + higher_order_latent_haven_haven_score(4);
    haven_total = haven_total + higher_order_latent_haven_haven_finish(8);
    if haven_total > 122 {
        haven_total = haven_total - 7;
    } else {
        haven_total = haven_total + 18;
    }
    return haven_total;
}

flow higher_order_latent_haven_haven_prepare(seed: i32) -> i32 ![]
{
    var garden_prepare_total = seed + 9;
    var garden_prepare_cursor = 0;
    while garden_prepare_cursor < 10 limit Iterations(10) {
        garden_prepare_total = garden_prepare_total + garden_prepare_cursor + 5;
        garden_prepare_cursor = garden_prepare_cursor + 1;
    }
    if garden_prepare_total % 2 == 0 {
        garden_prepare_total = garden_prepare_total + higher_order_latent_haven_haven_score(1);
    } else {
        garden_prepare_total = garden_prepare_total - 3;
    }
    var garden_prepare_left = garden_prepare_total + seed;
    var garden_prepare_right = garden_prepare_left * 4;
    var garden_prepare_merged = garden_prepare_right - garden_prepare_left;
    if garden_prepare_merged > 20 {
        garden_prepare_total = garden_prepare_total + garden_prepare_merged;
    }
    return garden_prepare_total;
}

flow higher_order_latent_haven_haven_route(seed: i32) -> i32 ![]
{
    var garden_route_total = seed * 9;
    var garden_route_cursor = 0;
    while garden_route_cursor < 11 limit Iterations(11) {
        garden_route_total = garden_route_total + garden_route_cursor + 5;
        garden_route_cursor = garden_route_cursor + 1;
    }
    if garden_route_total % 2 == 0 {
        garden_route_total = garden_route_total + 18;
    } else {
        garden_route_total = garden_route_total - 3;
    }
    var garden_route_left = garden_route_total + seed;
    var garden_route_right = garden_route_left * 4;
    var garden_route_merged = garden_route_right - garden_route_left;
    if garden_route_merged > 20 {
        garden_route_total = garden_route_total + garden_route_merged;
    }
    return garden_route_total;
}

flow higher_order_latent_haven_haven_score(seed: i32) -> i32 ![]
{
    var garden_score_total = seed + 9;
    var garden_score_cursor = 0;
    while garden_score_cursor < 11 limit Iterations(11) {
        garden_score_total = garden_score_total + garden_score_cursor + 5;
        garden_score_cursor = garden_score_cursor + 1;
    }
    if garden_score_total % 2 == 0 {
        garden_score_total = garden_score_total + 18;
    } else {
        garden_score_total = garden_score_total - 3;
    }
    var garden_score_left = garden_score_total + seed;
    var garden_score_right = garden_score_left * 4;
    var garden_score_merged = garden_score_right - garden_score_left;
    if garden_score_merged > 20 {
        garden_score_total = garden_score_total + garden_score_merged;
    }
    return garden_score_total;
}

flow higher_order_latent_haven_haven_finish(seed: i32) -> i32 ![]
{
    var garden_finish_total = seed - 9;
    var garden_finish_cursor = 0;
    while garden_finish_cursor < 7 limit Iterations(7) {
        garden_finish_total = garden_finish_total + garden_finish_cursor + 5;
        garden_finish_cursor = garden_finish_cursor + 1;
    }
    if garden_finish_total % 2 == 0 {
        garden_finish_total = garden_finish_total + 18;
    } else {
        garden_finish_total = garden_finish_total - 3;
    }
    var garden_finish_left = garden_finish_total + seed;
    var garden_finish_right = garden_finish_left * 4;
    var garden_finish_merged = garden_finish_right - garden_finish_left;
    if garden_finish_merged > 20 {
        garden_finish_total = garden_finish_total + garden_finish_merged;
    }
    return garden_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var haven_seed = 6;
    if args.len() > 0 {
        haven_seed = haven_seed + 1;
    } else {
        haven_seed = haven_seed + 2;
    }
    let haven_result = higher_order_latent_haven_haven_entry(haven_seed);
    if haven_result > 0 {
        return 0;
    }
    return 1;
}
