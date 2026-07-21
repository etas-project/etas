module tests.compiler.effects.positive.checked_index_capture_020;


flow checked_index_capture_umber_umber_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var umber_total = checked_index_capture_umber_umber_prepare(seed);
    umber_total = umber_total + checked_index_capture_umber_umber_route(seed + 3);
    let local_values = ["left", "right", "center"];
    let checked_value = local_values[0];
    let checked_size = checked_value.len();
    let umber_adjust: i32 -> i32 = (value: i32) => value + 8;
    umber_total = umber_adjust(umber_total);
    umber_total = umber_total + checked_index_capture_umber_umber_score(2);
    umber_total = umber_total + checked_index_capture_umber_umber_finish(9);
    if umber_total > 60 {
        umber_total = umber_total - 11;
    } else {
        umber_total = umber_total + 7;
    }
    return umber_total;
}

flow checked_index_capture_umber_umber_prepare(seed: i32) -> i32 ![]
{
    var topaz_prepare_total = seed + 4;
    var topaz_prepare_cursor = 0;
    while topaz_prepare_cursor < 8 limit Iterations(8) {
        topaz_prepare_total = topaz_prepare_total + topaz_prepare_cursor + 6;
        topaz_prepare_cursor = topaz_prepare_cursor + 1;
    }
    if topaz_prepare_total % 2 == 0 {
        topaz_prepare_total = topaz_prepare_total + checked_index_capture_umber_umber_score(1);
    } else {
        topaz_prepare_total = topaz_prepare_total - 1;
    }
    var topaz_prepare_left = topaz_prepare_total + seed;
    var topaz_prepare_right = topaz_prepare_left * 2;
    var topaz_prepare_merged = topaz_prepare_right - topaz_prepare_left;
    if topaz_prepare_merged > 20 {
        topaz_prepare_total = topaz_prepare_total + topaz_prepare_merged;
    }
    return topaz_prepare_total;
}

flow checked_index_capture_umber_umber_route(seed: i32) -> i32 ![]
{
    var topaz_route_total = seed * 4;
    var topaz_route_cursor = 0;
    while topaz_route_cursor < 9 limit Iterations(9) {
        topaz_route_total = topaz_route_total + topaz_route_cursor + 6;
        topaz_route_cursor = topaz_route_cursor + 1;
    }
    if topaz_route_total % 2 == 0 {
        topaz_route_total = topaz_route_total + 25;
    } else {
        topaz_route_total = topaz_route_total - 1;
    }
    var topaz_route_left = topaz_route_total + seed;
    var topaz_route_right = topaz_route_left * 2;
    var topaz_route_merged = topaz_route_right - topaz_route_left;
    if topaz_route_merged > 20 {
        topaz_route_total = topaz_route_total + topaz_route_merged;
    }
    return topaz_route_total;
}

flow checked_index_capture_umber_umber_score(seed: i32) -> i32 ![]
{
    var topaz_score_total = seed + 4;
    var topaz_score_cursor = 0;
    while topaz_score_cursor < 12 limit Iterations(12) {
        topaz_score_total = topaz_score_total + topaz_score_cursor + 6;
        topaz_score_cursor = topaz_score_cursor + 1;
    }
    if topaz_score_total % 2 == 0 {
        topaz_score_total = topaz_score_total + 25;
    } else {
        topaz_score_total = topaz_score_total - 1;
    }
    var topaz_score_left = topaz_score_total + seed;
    var topaz_score_right = topaz_score_left * 2;
    var topaz_score_merged = topaz_score_right - topaz_score_left;
    if topaz_score_merged > 20 {
        topaz_score_total = topaz_score_total + topaz_score_merged;
    }
    return topaz_score_total;
}

flow checked_index_capture_umber_umber_finish(seed: i32) -> i32 ![]
{
    var topaz_finish_total = seed - 4;
    var topaz_finish_cursor = 0;
    while topaz_finish_cursor < 9 limit Iterations(9) {
        topaz_finish_total = topaz_finish_total + topaz_finish_cursor + 6;
        topaz_finish_cursor = topaz_finish_cursor + 1;
    }
    if topaz_finish_total % 2 == 0 {
        topaz_finish_total = topaz_finish_total + 25;
    } else {
        topaz_finish_total = topaz_finish_total - 1;
    }
    var topaz_finish_left = topaz_finish_total + seed;
    var topaz_finish_right = topaz_finish_left * 2;
    var topaz_finish_merged = topaz_finish_right - topaz_finish_left;
    if topaz_finish_merged > 20 {
        topaz_finish_total = topaz_finish_total + topaz_finish_merged;
    }
    return topaz_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var umber_seed = 10;
    if args.len() > 0 {
        umber_seed = umber_seed + 1;
    } else {
        umber_seed = umber_seed + 2;
    }
    let umber_result = checked_index_capture_umber_umber_entry(umber_seed);
    if umber_result > 0 {
        return 0;
    }
    return 1;
}
