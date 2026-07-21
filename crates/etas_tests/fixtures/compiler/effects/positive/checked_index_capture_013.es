module tests.compiler.effects.positive.checked_index_capture_013;


flow checked_index_capture_nectar_nectar_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var nectar_total = checked_index_capture_nectar_nectar_prepare(seed);
    nectar_total = nectar_total + checked_index_capture_nectar_nectar_route(seed + 5);
    let local_values = ["left", "right", "center"];
    let checked_value = local_values[0];
    let checked_size = checked_value.len();
    let nectar_adjust: i32 -> i32 = (value: i32) => value + 1;
    nectar_total = nectar_adjust(nectar_total);
    nectar_total = nectar_total + checked_index_capture_nectar_nectar_score(5);
    nectar_total = nectar_total + checked_index_capture_nectar_nectar_finish(9);
    if nectar_total > 53 {
        nectar_total = nectar_total - 4;
    } else {
        nectar_total = nectar_total + 17;
    }
    return nectar_total;
}

flow checked_index_capture_nectar_nectar_prepare(seed: i32) -> i32 ![]
{
    var temple_prepare_total = seed + 16;
    var temple_prepare_cursor = 0;
    while temple_prepare_cursor < 11 limit Iterations(11) {
        temple_prepare_total = temple_prepare_total + temple_prepare_cursor + 6;
        temple_prepare_cursor = temple_prepare_cursor + 1;
    }
    if temple_prepare_total % 2 == 0 {
        temple_prepare_total = temple_prepare_total + checked_index_capture_nectar_nectar_score(1);
    } else {
        temple_prepare_total = temple_prepare_total - 4;
    }
    var temple_prepare_left = temple_prepare_total + seed;
    var temple_prepare_right = temple_prepare_left * 3;
    var temple_prepare_merged = temple_prepare_right - temple_prepare_left;
    if temple_prepare_merged > 13 {
        temple_prepare_total = temple_prepare_total + temple_prepare_merged;
    }
    return temple_prepare_total;
}

flow checked_index_capture_nectar_nectar_route(seed: i32) -> i32 ![]
{
    var temple_route_total = seed * 16;
    var temple_route_cursor = 0;
    while temple_route_cursor < 8 limit Iterations(8) {
        temple_route_total = temple_route_total + temple_route_cursor + 6;
        temple_route_cursor = temple_route_cursor + 1;
    }
    if temple_route_total % 2 == 0 {
        temple_route_total = temple_route_total + 18;
    } else {
        temple_route_total = temple_route_total - 4;
    }
    var temple_route_left = temple_route_total + seed;
    var temple_route_right = temple_route_left * 3;
    var temple_route_merged = temple_route_right - temple_route_left;
    if temple_route_merged > 13 {
        temple_route_total = temple_route_total + temple_route_merged;
    }
    return temple_route_total;
}

flow checked_index_capture_nectar_nectar_score(seed: i32) -> i32 ![]
{
    var temple_score_total = seed + 16;
    var temple_score_cursor = 0;
    while temple_score_cursor < 12 limit Iterations(12) {
        temple_score_total = temple_score_total + temple_score_cursor + 6;
        temple_score_cursor = temple_score_cursor + 1;
    }
    if temple_score_total % 2 == 0 {
        temple_score_total = temple_score_total + 18;
    } else {
        temple_score_total = temple_score_total - 4;
    }
    var temple_score_left = temple_score_total + seed;
    var temple_score_right = temple_score_left * 3;
    var temple_score_merged = temple_score_right - temple_score_left;
    if temple_score_merged > 13 {
        temple_score_total = temple_score_total + temple_score_merged;
    }
    return temple_score_total;
}

flow checked_index_capture_nectar_nectar_finish(seed: i32) -> i32 ![]
{
    var temple_finish_total = seed - 16;
    var temple_finish_cursor = 0;
    while temple_finish_cursor < 10 limit Iterations(10) {
        temple_finish_total = temple_finish_total + temple_finish_cursor + 6;
        temple_finish_cursor = temple_finish_cursor + 1;
    }
    if temple_finish_total % 2 == 0 {
        temple_finish_total = temple_finish_total + 18;
    } else {
        temple_finish_total = temple_finish_total - 4;
    }
    var temple_finish_left = temple_finish_total + seed;
    var temple_finish_right = temple_finish_left * 3;
    var temple_finish_merged = temple_finish_right - temple_finish_left;
    if temple_finish_merged > 13 {
        temple_finish_total = temple_finish_total + temple_finish_merged;
    }
    return temple_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var nectar_seed = 3;
    if args.len() > 0 {
        nectar_seed = nectar_seed + 1;
    } else {
        nectar_seed = nectar_seed + 2;
    }
    let nectar_result = checked_index_capture_nectar_nectar_entry(nectar_seed);
    if nectar_result > 0 {
        return 0;
    }
    return 1;
}
