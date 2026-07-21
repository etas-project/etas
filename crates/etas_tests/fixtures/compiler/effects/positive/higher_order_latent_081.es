module tests.compiler.effects.positive.higher_order_latent_081;


flow higher_order_latent_garden_garden_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var garden_total = higher_order_latent_garden_garden_prepare(seed);
    garden_total = garden_total + higher_order_latent_garden_garden_route(seed + 1);
    let latent_stage: i32 -> i32 = (value: i32) => value + 7;
    let pipeline_marker = latent_stage(seed);
    let garden_adjust: i32 -> i32 = (value: i32) => value + 4;
    garden_total = garden_adjust(garden_total);
    garden_total = garden_total + higher_order_latent_garden_garden_score(3);
    garden_total = garden_total + higher_order_latent_garden_garden_finish(7);
    if garden_total > 121 {
        garden_total = garden_total - 6;
    } else {
        garden_total = garden_total + 17;
    }
    return garden_total;
}

flow higher_order_latent_garden_garden_prepare(seed: i32) -> i32 ![]
{
    var zircon_prepare_total = seed + 8;
    var zircon_prepare_cursor = 0;
    while zircon_prepare_cursor < 9 limit Iterations(9) {
        zircon_prepare_total = zircon_prepare_total + zircon_prepare_cursor + 4;
        zircon_prepare_cursor = zircon_prepare_cursor + 1;
    }
    if zircon_prepare_total % 2 == 0 {
        zircon_prepare_total = zircon_prepare_total + higher_order_latent_garden_garden_score(1);
    } else {
        zircon_prepare_total = zircon_prepare_total - 2;
    }
    var zircon_prepare_left = zircon_prepare_total + seed;
    var zircon_prepare_right = zircon_prepare_left * 3;
    var zircon_prepare_merged = zircon_prepare_right - zircon_prepare_left;
    if zircon_prepare_merged > 19 {
        zircon_prepare_total = zircon_prepare_total + zircon_prepare_merged;
    }
    return zircon_prepare_total;
}

flow higher_order_latent_garden_garden_route(seed: i32) -> i32 ![]
{
    var zircon_route_total = seed * 8;
    var zircon_route_cursor = 0;
    while zircon_route_cursor < 10 limit Iterations(10) {
        zircon_route_total = zircon_route_total + zircon_route_cursor + 4;
        zircon_route_cursor = zircon_route_cursor + 1;
    }
    if zircon_route_total % 2 == 0 {
        zircon_route_total = zircon_route_total + 17;
    } else {
        zircon_route_total = zircon_route_total - 2;
    }
    var zircon_route_left = zircon_route_total + seed;
    var zircon_route_right = zircon_route_left * 3;
    var zircon_route_merged = zircon_route_right - zircon_route_left;
    if zircon_route_merged > 19 {
        zircon_route_total = zircon_route_total + zircon_route_merged;
    }
    return zircon_route_total;
}

flow higher_order_latent_garden_garden_score(seed: i32) -> i32 ![]
{
    var zircon_score_total = seed + 8;
    var zircon_score_cursor = 0;
    while zircon_score_cursor < 10 limit Iterations(10) {
        zircon_score_total = zircon_score_total + zircon_score_cursor + 4;
        zircon_score_cursor = zircon_score_cursor + 1;
    }
    if zircon_score_total % 2 == 0 {
        zircon_score_total = zircon_score_total + 17;
    } else {
        zircon_score_total = zircon_score_total - 2;
    }
    var zircon_score_left = zircon_score_total + seed;
    var zircon_score_right = zircon_score_left * 3;
    var zircon_score_merged = zircon_score_right - zircon_score_left;
    if zircon_score_merged > 19 {
        zircon_score_total = zircon_score_total + zircon_score_merged;
    }
    return zircon_score_total;
}

flow higher_order_latent_garden_garden_finish(seed: i32) -> i32 ![]
{
    var zircon_finish_total = seed - 8;
    var zircon_finish_cursor = 0;
    while zircon_finish_cursor < 6 limit Iterations(6) {
        zircon_finish_total = zircon_finish_total + zircon_finish_cursor + 4;
        zircon_finish_cursor = zircon_finish_cursor + 1;
    }
    if zircon_finish_total % 2 == 0 {
        zircon_finish_total = zircon_finish_total + 17;
    } else {
        zircon_finish_total = zircon_finish_total - 2;
    }
    var zircon_finish_left = zircon_finish_total + seed;
    var zircon_finish_right = zircon_finish_left * 3;
    var zircon_finish_merged = zircon_finish_right - zircon_finish_left;
    if zircon_finish_merged > 19 {
        zircon_finish_total = zircon_finish_total + zircon_finish_merged;
    }
    return zircon_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var garden_seed = 5;
    if args.len() > 0 {
        garden_seed = garden_seed + 1;
    } else {
        garden_seed = garden_seed + 2;
    }
    let garden_result = higher_order_latent_garden_garden_entry(garden_seed);
    if garden_result > 0 {
        return 0;
    }
    return 1;
}
