module tests.compiler.effects.positive.checked_index_capture_019;


flow checked_index_capture_topaz_topaz_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var topaz_total = checked_index_capture_topaz_topaz_prepare(seed);
    topaz_total = topaz_total + checked_index_capture_topaz_topaz_route(seed + 2);
    let local_values = ["left", "right", "center"];
    let checked_value = local_values[0];
    let checked_size = checked_value.len();
    let topaz_adjust: i32 -> i32 = (value: i32) => value + 7;
    topaz_total = topaz_adjust(topaz_total);
    topaz_total = topaz_total + checked_index_capture_topaz_topaz_score(6);
    topaz_total = topaz_total + checked_index_capture_topaz_topaz_finish(8);
    if topaz_total > 59 {
        topaz_total = topaz_total - 10;
    } else {
        topaz_total = topaz_total + 6;
    }
    return topaz_total;
}

flow checked_index_capture_topaz_topaz_prepare(seed: i32) -> i32 ![]
{
    var mango_prepare_total = seed + 3;
    var mango_prepare_cursor = 0;
    while mango_prepare_cursor < 12 limit Iterations(12) {
        mango_prepare_total = mango_prepare_total + mango_prepare_cursor + 5;
        mango_prepare_cursor = mango_prepare_cursor + 1;
    }
    if mango_prepare_total % 2 == 0 {
        mango_prepare_total = mango_prepare_total + checked_index_capture_topaz_topaz_score(1);
    } else {
        mango_prepare_total = mango_prepare_total - 5;
    }
    var mango_prepare_left = mango_prepare_total + seed;
    var mango_prepare_right = mango_prepare_left * 5;
    var mango_prepare_merged = mango_prepare_right - mango_prepare_left;
    if mango_prepare_merged > 19 {
        mango_prepare_total = mango_prepare_total + mango_prepare_merged;
    }
    return mango_prepare_total;
}

flow checked_index_capture_topaz_topaz_route(seed: i32) -> i32 ![]
{
    var mango_route_total = seed * 3;
    var mango_route_cursor = 0;
    while mango_route_cursor < 8 limit Iterations(8) {
        mango_route_total = mango_route_total + mango_route_cursor + 5;
        mango_route_cursor = mango_route_cursor + 1;
    }
    if mango_route_total % 2 == 0 {
        mango_route_total = mango_route_total + 24;
    } else {
        mango_route_total = mango_route_total - 5;
    }
    var mango_route_left = mango_route_total + seed;
    var mango_route_right = mango_route_left * 5;
    var mango_route_merged = mango_route_right - mango_route_left;
    if mango_route_merged > 19 {
        mango_route_total = mango_route_total + mango_route_merged;
    }
    return mango_route_total;
}

flow checked_index_capture_topaz_topaz_score(seed: i32) -> i32 ![]
{
    var mango_score_total = seed + 3;
    var mango_score_cursor = 0;
    while mango_score_cursor < 11 limit Iterations(11) {
        mango_score_total = mango_score_total + mango_score_cursor + 5;
        mango_score_cursor = mango_score_cursor + 1;
    }
    if mango_score_total % 2 == 0 {
        mango_score_total = mango_score_total + 24;
    } else {
        mango_score_total = mango_score_total - 5;
    }
    var mango_score_left = mango_score_total + seed;
    var mango_score_right = mango_score_left * 5;
    var mango_score_merged = mango_score_right - mango_score_left;
    if mango_score_merged > 19 {
        mango_score_total = mango_score_total + mango_score_merged;
    }
    return mango_score_total;
}

flow checked_index_capture_topaz_topaz_finish(seed: i32) -> i32 ![]
{
    var mango_finish_total = seed - 3;
    var mango_finish_cursor = 0;
    while mango_finish_cursor < 8 limit Iterations(8) {
        mango_finish_total = mango_finish_total + mango_finish_cursor + 5;
        mango_finish_cursor = mango_finish_cursor + 1;
    }
    if mango_finish_total % 2 == 0 {
        mango_finish_total = mango_finish_total + 24;
    } else {
        mango_finish_total = mango_finish_total - 5;
    }
    var mango_finish_left = mango_finish_total + seed;
    var mango_finish_right = mango_finish_left * 5;
    var mango_finish_merged = mango_finish_right - mango_finish_left;
    if mango_finish_merged > 19 {
        mango_finish_total = mango_finish_total + mango_finish_merged;
    }
    return mango_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var topaz_seed = 9;
    if args.len() > 0 {
        topaz_seed = topaz_seed + 1;
    } else {
        topaz_seed = topaz_seed + 2;
    }
    let topaz_result = checked_index_capture_topaz_topaz_entry(topaz_seed);
    if topaz_result > 0 {
        return 0;
    }
    return 1;
}
