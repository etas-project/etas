module tests.compiler.effects.positive.checked_index_capture_014;


flow checked_index_capture_opal_opal_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var opal_total = checked_index_capture_opal_opal_prepare(seed);
    opal_total = opal_total + checked_index_capture_opal_opal_route(seed + 6);
    let local_values = ["left", "right", "center"];
    let checked_value = local_values[0];
    let checked_size = checked_value.len();
    let opal_adjust: i32 -> i32 = (value: i32) => value + 2;
    opal_total = opal_adjust(opal_total);
    opal_total = opal_total + checked_index_capture_opal_opal_score(6);
    opal_total = opal_total + checked_index_capture_opal_opal_finish(3);
    if opal_total > 54 {
        opal_total = opal_total - 5;
    } else {
        opal_total = opal_total + 18;
    }
    return opal_total;
}

flow checked_index_capture_opal_opal_prepare(seed: i32) -> i32 ![]
{
    var bridge_prepare_total = seed + 17;
    var bridge_prepare_cursor = 0;
    while bridge_prepare_cursor < 12 limit Iterations(12) {
        bridge_prepare_total = bridge_prepare_total + bridge_prepare_cursor + 0;
        bridge_prepare_cursor = bridge_prepare_cursor + 1;
    }
    if bridge_prepare_total % 2 == 0 {
        bridge_prepare_total = bridge_prepare_total + checked_index_capture_opal_opal_score(1);
    } else {
        bridge_prepare_total = bridge_prepare_total - 5;
    }
    var bridge_prepare_left = bridge_prepare_total + seed;
    var bridge_prepare_right = bridge_prepare_left * 4;
    var bridge_prepare_merged = bridge_prepare_right - bridge_prepare_left;
    if bridge_prepare_merged > 14 {
        bridge_prepare_total = bridge_prepare_total + bridge_prepare_merged;
    }
    return bridge_prepare_total;
}

flow checked_index_capture_opal_opal_route(seed: i32) -> i32 ![]
{
    var bridge_route_total = seed * 17;
    var bridge_route_cursor = 0;
    while bridge_route_cursor < 9 limit Iterations(9) {
        bridge_route_total = bridge_route_total + bridge_route_cursor + 0;
        bridge_route_cursor = bridge_route_cursor + 1;
    }
    if bridge_route_total % 2 == 0 {
        bridge_route_total = bridge_route_total + 19;
    } else {
        bridge_route_total = bridge_route_total - 5;
    }
    var bridge_route_left = bridge_route_total + seed;
    var bridge_route_right = bridge_route_left * 4;
    var bridge_route_merged = bridge_route_right - bridge_route_left;
    if bridge_route_merged > 14 {
        bridge_route_total = bridge_route_total + bridge_route_merged;
    }
    return bridge_route_total;
}

flow checked_index_capture_opal_opal_score(seed: i32) -> i32 ![]
{
    var bridge_score_total = seed + 17;
    var bridge_score_cursor = 0;
    while bridge_score_cursor < 6 limit Iterations(6) {
        bridge_score_total = bridge_score_total + bridge_score_cursor + 0;
        bridge_score_cursor = bridge_score_cursor + 1;
    }
    if bridge_score_total % 2 == 0 {
        bridge_score_total = bridge_score_total + 19;
    } else {
        bridge_score_total = bridge_score_total - 5;
    }
    var bridge_score_left = bridge_score_total + seed;
    var bridge_score_right = bridge_score_left * 4;
    var bridge_score_merged = bridge_score_right - bridge_score_left;
    if bridge_score_merged > 14 {
        bridge_score_total = bridge_score_total + bridge_score_merged;
    }
    return bridge_score_total;
}

flow checked_index_capture_opal_opal_finish(seed: i32) -> i32 ![]
{
    var bridge_finish_total = seed - 17;
    var bridge_finish_cursor = 0;
    while bridge_finish_cursor < 11 limit Iterations(11) {
        bridge_finish_total = bridge_finish_total + bridge_finish_cursor + 0;
        bridge_finish_cursor = bridge_finish_cursor + 1;
    }
    if bridge_finish_total % 2 == 0 {
        bridge_finish_total = bridge_finish_total + 19;
    } else {
        bridge_finish_total = bridge_finish_total - 5;
    }
    var bridge_finish_left = bridge_finish_total + seed;
    var bridge_finish_right = bridge_finish_left * 4;
    var bridge_finish_merged = bridge_finish_right - bridge_finish_left;
    if bridge_finish_merged > 14 {
        bridge_finish_total = bridge_finish_total + bridge_finish_merged;
    }
    return bridge_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var opal_seed = 4;
    if args.len() > 0 {
        opal_seed = opal_seed + 1;
    } else {
        opal_seed = opal_seed + 2;
    }
    let opal_result = checked_index_capture_opal_opal_entry(opal_seed);
    if opal_result > 0 {
        return 0;
    }
    return 1;
}
