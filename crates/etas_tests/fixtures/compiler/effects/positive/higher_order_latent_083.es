module tests.compiler.effects.positive.higher_order_latent_083;


flow higher_order_latent_iron_iron_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var iron_total = higher_order_latent_iron_iron_prepare(seed);
    iron_total = iron_total + higher_order_latent_iron_iron_route(seed + 3);
    let latent_stage: i32 -> i32 = (value: i32) => value + 9;
    let pipeline_marker = latent_stage(seed);
    let iron_adjust: i32 -> i32 = (value: i32) => value + 6;
    iron_total = iron_adjust(iron_total);
    iron_total = iron_total + higher_order_latent_iron_iron_score(5);
    iron_total = iron_total + higher_order_latent_iron_iron_finish(9);
    if iron_total > 123 {
        iron_total = iron_total - 8;
    } else {
        iron_total = iron_total + 19;
    }
    return iron_total;
}

flow higher_order_latent_iron_iron_prepare(seed: i32) -> i32 ![]
{
    var needle_prepare_total = seed + 10;
    var needle_prepare_cursor = 0;
    while needle_prepare_cursor < 11 limit Iterations(11) {
        needle_prepare_total = needle_prepare_total + needle_prepare_cursor + 6;
        needle_prepare_cursor = needle_prepare_cursor + 1;
    }
    if needle_prepare_total % 2 == 0 {
        needle_prepare_total = needle_prepare_total + higher_order_latent_iron_iron_score(1);
    } else {
        needle_prepare_total = needle_prepare_total - 4;
    }
    var needle_prepare_left = needle_prepare_total + seed;
    var needle_prepare_right = needle_prepare_left * 5;
    var needle_prepare_merged = needle_prepare_right - needle_prepare_left;
    if needle_prepare_merged > 21 {
        needle_prepare_total = needle_prepare_total + needle_prepare_merged;
    }
    return needle_prepare_total;
}

flow higher_order_latent_iron_iron_route(seed: i32) -> i32 ![]
{
    var needle_route_total = seed * 10;
    var needle_route_cursor = 0;
    while needle_route_cursor < 12 limit Iterations(12) {
        needle_route_total = needle_route_total + needle_route_cursor + 6;
        needle_route_cursor = needle_route_cursor + 1;
    }
    if needle_route_total % 2 == 0 {
        needle_route_total = needle_route_total + 19;
    } else {
        needle_route_total = needle_route_total - 4;
    }
    var needle_route_left = needle_route_total + seed;
    var needle_route_right = needle_route_left * 5;
    var needle_route_merged = needle_route_right - needle_route_left;
    if needle_route_merged > 21 {
        needle_route_total = needle_route_total + needle_route_merged;
    }
    return needle_route_total;
}

flow higher_order_latent_iron_iron_score(seed: i32) -> i32 ![]
{
    var needle_score_total = seed + 10;
    var needle_score_cursor = 0;
    while needle_score_cursor < 12 limit Iterations(12) {
        needle_score_total = needle_score_total + needle_score_cursor + 6;
        needle_score_cursor = needle_score_cursor + 1;
    }
    if needle_score_total % 2 == 0 {
        needle_score_total = needle_score_total + 19;
    } else {
        needle_score_total = needle_score_total - 4;
    }
    var needle_score_left = needle_score_total + seed;
    var needle_score_right = needle_score_left * 5;
    var needle_score_merged = needle_score_right - needle_score_left;
    if needle_score_merged > 21 {
        needle_score_total = needle_score_total + needle_score_merged;
    }
    return needle_score_total;
}

flow higher_order_latent_iron_iron_finish(seed: i32) -> i32 ![]
{
    var needle_finish_total = seed - 10;
    var needle_finish_cursor = 0;
    while needle_finish_cursor < 8 limit Iterations(8) {
        needle_finish_total = needle_finish_total + needle_finish_cursor + 6;
        needle_finish_cursor = needle_finish_cursor + 1;
    }
    if needle_finish_total % 2 == 0 {
        needle_finish_total = needle_finish_total + 19;
    } else {
        needle_finish_total = needle_finish_total - 4;
    }
    var needle_finish_left = needle_finish_total + seed;
    var needle_finish_right = needle_finish_left * 5;
    var needle_finish_merged = needle_finish_right - needle_finish_left;
    if needle_finish_merged > 21 {
        needle_finish_total = needle_finish_total + needle_finish_merged;
    }
    return needle_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var iron_seed = 7;
    if args.len() > 0 {
        iron_seed = iron_seed + 1;
    } else {
        iron_seed = iron_seed + 2;
    }
    let iron_result = higher_order_latent_iron_iron_entry(iron_seed);
    if iron_result > 0 {
        return 0;
    }
    return 1;
}
