module tests.compiler.effects.positive.checked_index_capture_012;


flow checked_index_capture_mango_mango_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var mango_total = checked_index_capture_mango_mango_prepare(seed);
    mango_total = mango_total + checked_index_capture_mango_mango_route(seed + 4);
    let local_values = ["left", "right", "center"];
    let checked_value = local_values[0];
    let checked_size = checked_value.len();
    let mango_adjust: i32 -> i32 = (value: i32) => value + 13;
    mango_total = mango_adjust(mango_total);
    mango_total = mango_total + checked_index_capture_mango_mango_score(4);
    mango_total = mango_total + checked_index_capture_mango_mango_finish(8);
    if mango_total > 52 {
        mango_total = mango_total - 3;
    } else {
        mango_total = mango_total + 16;
    }
    return mango_total;
}

flow checked_index_capture_mango_mango_prepare(seed: i32) -> i32 ![]
{
    var meadow_prepare_total = seed + 15;
    var meadow_prepare_cursor = 0;
    while meadow_prepare_cursor < 10 limit Iterations(10) {
        meadow_prepare_total = meadow_prepare_total + meadow_prepare_cursor + 5;
        meadow_prepare_cursor = meadow_prepare_cursor + 1;
    }
    if meadow_prepare_total % 2 == 0 {
        meadow_prepare_total = meadow_prepare_total + checked_index_capture_mango_mango_score(1);
    } else {
        meadow_prepare_total = meadow_prepare_total - 3;
    }
    var meadow_prepare_left = meadow_prepare_total + seed;
    var meadow_prepare_right = meadow_prepare_left * 2;
    var meadow_prepare_merged = meadow_prepare_right - meadow_prepare_left;
    if meadow_prepare_merged > 12 {
        meadow_prepare_total = meadow_prepare_total + meadow_prepare_merged;
    }
    return meadow_prepare_total;
}

flow checked_index_capture_mango_mango_route(seed: i32) -> i32 ![]
{
    var meadow_route_total = seed * 15;
    var meadow_route_cursor = 0;
    while meadow_route_cursor < 7 limit Iterations(7) {
        meadow_route_total = meadow_route_total + meadow_route_cursor + 5;
        meadow_route_cursor = meadow_route_cursor + 1;
    }
    if meadow_route_total % 2 == 0 {
        meadow_route_total = meadow_route_total + 17;
    } else {
        meadow_route_total = meadow_route_total - 3;
    }
    var meadow_route_left = meadow_route_total + seed;
    var meadow_route_right = meadow_route_left * 2;
    var meadow_route_merged = meadow_route_right - meadow_route_left;
    if meadow_route_merged > 12 {
        meadow_route_total = meadow_route_total + meadow_route_merged;
    }
    return meadow_route_total;
}

flow checked_index_capture_mango_mango_score(seed: i32) -> i32 ![]
{
    var meadow_score_total = seed + 15;
    var meadow_score_cursor = 0;
    while meadow_score_cursor < 11 limit Iterations(11) {
        meadow_score_total = meadow_score_total + meadow_score_cursor + 5;
        meadow_score_cursor = meadow_score_cursor + 1;
    }
    if meadow_score_total % 2 == 0 {
        meadow_score_total = meadow_score_total + 17;
    } else {
        meadow_score_total = meadow_score_total - 3;
    }
    var meadow_score_left = meadow_score_total + seed;
    var meadow_score_right = meadow_score_left * 2;
    var meadow_score_merged = meadow_score_right - meadow_score_left;
    if meadow_score_merged > 12 {
        meadow_score_total = meadow_score_total + meadow_score_merged;
    }
    return meadow_score_total;
}

flow checked_index_capture_mango_mango_finish(seed: i32) -> i32 ![]
{
    var meadow_finish_total = seed - 15;
    var meadow_finish_cursor = 0;
    while meadow_finish_cursor < 9 limit Iterations(9) {
        meadow_finish_total = meadow_finish_total + meadow_finish_cursor + 5;
        meadow_finish_cursor = meadow_finish_cursor + 1;
    }
    if meadow_finish_total % 2 == 0 {
        meadow_finish_total = meadow_finish_total + 17;
    } else {
        meadow_finish_total = meadow_finish_total - 3;
    }
    var meadow_finish_left = meadow_finish_total + seed;
    var meadow_finish_right = meadow_finish_left * 2;
    var meadow_finish_merged = meadow_finish_right - meadow_finish_left;
    if meadow_finish_merged > 12 {
        meadow_finish_total = meadow_finish_total + meadow_finish_merged;
    }
    return meadow_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var mango_seed = 2;
    if args.len() > 0 {
        mango_seed = mango_seed + 1;
    } else {
        mango_seed = mango_seed + 2;
    }
    let mango_result = checked_index_capture_mango_mango_entry(mango_seed);
    if mango_result > 0 {
        return 0;
    }
    return 1;
}
