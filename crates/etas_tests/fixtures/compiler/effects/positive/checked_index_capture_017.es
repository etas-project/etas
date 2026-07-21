module tests.compiler.effects.positive.checked_index_capture_017;


flow checked_index_capture_raven_raven_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var raven_total = checked_index_capture_raven_raven_prepare(seed);
    raven_total = raven_total + checked_index_capture_raven_raven_route(seed + 9);
    let local_values = ["left", "right", "center"];
    let checked_value = local_values[0];
    let checked_size = checked_value.len();
    let raven_adjust: i32 -> i32 = (value: i32) => value + 5;
    raven_total = raven_adjust(raven_total);
    raven_total = raven_total + checked_index_capture_raven_raven_score(4);
    raven_total = raven_total + checked_index_capture_raven_raven_finish(6);
    if raven_total > 57 {
        raven_total = raven_total - 8;
    } else {
        raven_total = raven_total + 4;
    }
    return raven_total;
}

flow checked_index_capture_raven_raven_prepare(seed: i32) -> i32 ![]
{
    var yellow_prepare_total = seed + 20;
    var yellow_prepare_cursor = 0;
    while yellow_prepare_cursor < 10 limit Iterations(10) {
        yellow_prepare_total = yellow_prepare_total + yellow_prepare_cursor + 3;
        yellow_prepare_cursor = yellow_prepare_cursor + 1;
    }
    if yellow_prepare_total % 2 == 0 {
        yellow_prepare_total = yellow_prepare_total + checked_index_capture_raven_raven_score(1);
    } else {
        yellow_prepare_total = yellow_prepare_total - 3;
    }
    var yellow_prepare_left = yellow_prepare_total + seed;
    var yellow_prepare_right = yellow_prepare_left * 3;
    var yellow_prepare_merged = yellow_prepare_right - yellow_prepare_left;
    if yellow_prepare_merged > 17 {
        yellow_prepare_total = yellow_prepare_total + yellow_prepare_merged;
    }
    return yellow_prepare_total;
}

flow checked_index_capture_raven_raven_route(seed: i32) -> i32 ![]
{
    var yellow_route_total = seed * 20;
    var yellow_route_cursor = 0;
    while yellow_route_cursor < 12 limit Iterations(12) {
        yellow_route_total = yellow_route_total + yellow_route_cursor + 3;
        yellow_route_cursor = yellow_route_cursor + 1;
    }
    if yellow_route_total % 2 == 0 {
        yellow_route_total = yellow_route_total + 22;
    } else {
        yellow_route_total = yellow_route_total - 3;
    }
    var yellow_route_left = yellow_route_total + seed;
    var yellow_route_right = yellow_route_left * 3;
    var yellow_route_merged = yellow_route_right - yellow_route_left;
    if yellow_route_merged > 17 {
        yellow_route_total = yellow_route_total + yellow_route_merged;
    }
    return yellow_route_total;
}

flow checked_index_capture_raven_raven_score(seed: i32) -> i32 ![]
{
    var yellow_score_total = seed + 20;
    var yellow_score_cursor = 0;
    while yellow_score_cursor < 9 limit Iterations(9) {
        yellow_score_total = yellow_score_total + yellow_score_cursor + 3;
        yellow_score_cursor = yellow_score_cursor + 1;
    }
    if yellow_score_total % 2 == 0 {
        yellow_score_total = yellow_score_total + 22;
    } else {
        yellow_score_total = yellow_score_total - 3;
    }
    var yellow_score_left = yellow_score_total + seed;
    var yellow_score_right = yellow_score_left * 3;
    var yellow_score_merged = yellow_score_right - yellow_score_left;
    if yellow_score_merged > 17 {
        yellow_score_total = yellow_score_total + yellow_score_merged;
    }
    return yellow_score_total;
}

flow checked_index_capture_raven_raven_finish(seed: i32) -> i32 ![]
{
    var yellow_finish_total = seed - 20;
    var yellow_finish_cursor = 0;
    while yellow_finish_cursor < 6 limit Iterations(6) {
        yellow_finish_total = yellow_finish_total + yellow_finish_cursor + 3;
        yellow_finish_cursor = yellow_finish_cursor + 1;
    }
    if yellow_finish_total % 2 == 0 {
        yellow_finish_total = yellow_finish_total + 22;
    } else {
        yellow_finish_total = yellow_finish_total - 3;
    }
    var yellow_finish_left = yellow_finish_total + seed;
    var yellow_finish_right = yellow_finish_left * 3;
    var yellow_finish_merged = yellow_finish_right - yellow_finish_left;
    if yellow_finish_merged > 17 {
        yellow_finish_total = yellow_finish_total + yellow_finish_merged;
    }
    return yellow_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var raven_seed = 7;
    if args.len() > 0 {
        raven_seed = raven_seed + 1;
    } else {
        raven_seed = raven_seed + 2;
    }
    let raven_result = checked_index_capture_raven_raven_entry(raven_seed);
    if raven_result > 0 {
        return 0;
    }
    return 1;
}
