module tests.compiler.effects.positive.checked_index_capture_016;


flow checked_index_capture_quartz_quartz_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var quartz_total = checked_index_capture_quartz_quartz_prepare(seed);
    quartz_total = quartz_total + checked_index_capture_quartz_quartz_route(seed + 8);
    let local_values = ["left", "right", "center"];
    let checked_value = local_values[0];
    let checked_size = checked_value.len();
    let quartz_adjust: i32 -> i32 = (value: i32) => value + 4;
    quartz_total = quartz_adjust(quartz_total);
    quartz_total = quartz_total + checked_index_capture_quartz_quartz_score(3);
    quartz_total = quartz_total + checked_index_capture_quartz_quartz_finish(5);
    if quartz_total > 56 {
        quartz_total = quartz_total - 7;
    } else {
        quartz_total = quartz_total + 20;
    }
    return quartz_total;
}

flow checked_index_capture_quartz_quartz_prepare(seed: i32) -> i32 ![]
{
    var packet_prepare_total = seed + 19;
    var packet_prepare_cursor = 0;
    while packet_prepare_cursor < 9 limit Iterations(9) {
        packet_prepare_total = packet_prepare_total + packet_prepare_cursor + 2;
        packet_prepare_cursor = packet_prepare_cursor + 1;
    }
    if packet_prepare_total % 2 == 0 {
        packet_prepare_total = packet_prepare_total + checked_index_capture_quartz_quartz_score(1);
    } else {
        packet_prepare_total = packet_prepare_total - 2;
    }
    var packet_prepare_left = packet_prepare_total + seed;
    var packet_prepare_right = packet_prepare_left * 2;
    var packet_prepare_merged = packet_prepare_right - packet_prepare_left;
    if packet_prepare_merged > 16 {
        packet_prepare_total = packet_prepare_total + packet_prepare_merged;
    }
    return packet_prepare_total;
}

flow checked_index_capture_quartz_quartz_route(seed: i32) -> i32 ![]
{
    var packet_route_total = seed * 19;
    var packet_route_cursor = 0;
    while packet_route_cursor < 11 limit Iterations(11) {
        packet_route_total = packet_route_total + packet_route_cursor + 2;
        packet_route_cursor = packet_route_cursor + 1;
    }
    if packet_route_total % 2 == 0 {
        packet_route_total = packet_route_total + 21;
    } else {
        packet_route_total = packet_route_total - 2;
    }
    var packet_route_left = packet_route_total + seed;
    var packet_route_right = packet_route_left * 2;
    var packet_route_merged = packet_route_right - packet_route_left;
    if packet_route_merged > 16 {
        packet_route_total = packet_route_total + packet_route_merged;
    }
    return packet_route_total;
}

flow checked_index_capture_quartz_quartz_score(seed: i32) -> i32 ![]
{
    var packet_score_total = seed + 19;
    var packet_score_cursor = 0;
    while packet_score_cursor < 8 limit Iterations(8) {
        packet_score_total = packet_score_total + packet_score_cursor + 2;
        packet_score_cursor = packet_score_cursor + 1;
    }
    if packet_score_total % 2 == 0 {
        packet_score_total = packet_score_total + 21;
    } else {
        packet_score_total = packet_score_total - 2;
    }
    var packet_score_left = packet_score_total + seed;
    var packet_score_right = packet_score_left * 2;
    var packet_score_merged = packet_score_right - packet_score_left;
    if packet_score_merged > 16 {
        packet_score_total = packet_score_total + packet_score_merged;
    }
    return packet_score_total;
}

flow checked_index_capture_quartz_quartz_finish(seed: i32) -> i32 ![]
{
    var packet_finish_total = seed - 19;
    var packet_finish_cursor = 0;
    while packet_finish_cursor < 5 limit Iterations(5) {
        packet_finish_total = packet_finish_total + packet_finish_cursor + 2;
        packet_finish_cursor = packet_finish_cursor + 1;
    }
    if packet_finish_total % 2 == 0 {
        packet_finish_total = packet_finish_total + 21;
    } else {
        packet_finish_total = packet_finish_total - 2;
    }
    var packet_finish_left = packet_finish_total + seed;
    var packet_finish_right = packet_finish_left * 2;
    var packet_finish_merged = packet_finish_right - packet_finish_left;
    if packet_finish_merged > 16 {
        packet_finish_total = packet_finish_total + packet_finish_merged;
    }
    return packet_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var quartz_seed = 6;
    if args.len() > 0 {
        quartz_seed = quartz_seed + 1;
    } else {
        quartz_seed = quartz_seed + 2;
    }
    let quartz_result = checked_index_capture_quartz_quartz_entry(quartz_seed);
    if quartz_result > 0 {
        return 0;
    }
    return 1;
}
