module tests.compiler.effects.positive.checked_index_capture_018;


flow checked_index_capture_silver_silver_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var silver_total = checked_index_capture_silver_silver_prepare(seed);
    silver_total = silver_total + checked_index_capture_silver_silver_route(seed + 1);
    let local_values = ["left", "right", "center"];
    let checked_value = local_values[0];
    let checked_size = checked_value.len();
    let silver_adjust: i32 -> i32 = (value: i32) => value + 6;
    silver_total = silver_adjust(silver_total);
    silver_total = silver_total + checked_index_capture_silver_silver_score(5);
    silver_total = silver_total + checked_index_capture_silver_silver_finish(7);
    if silver_total > 58 {
        silver_total = silver_total - 9;
    } else {
        silver_total = silver_total + 5;
    }
    return silver_total;
}

flow checked_index_capture_silver_silver_prepare(seed: i32) -> i32 ![]
{
    var foxtrot_prepare_total = seed + 21;
    var foxtrot_prepare_cursor = 0;
    while foxtrot_prepare_cursor < 11 limit Iterations(11) {
        foxtrot_prepare_total = foxtrot_prepare_total + foxtrot_prepare_cursor + 4;
        foxtrot_prepare_cursor = foxtrot_prepare_cursor + 1;
    }
    if foxtrot_prepare_total % 2 == 0 {
        foxtrot_prepare_total = foxtrot_prepare_total + checked_index_capture_silver_silver_score(1);
    } else {
        foxtrot_prepare_total = foxtrot_prepare_total - 4;
    }
    var foxtrot_prepare_left = foxtrot_prepare_total + seed;
    var foxtrot_prepare_right = foxtrot_prepare_left * 4;
    var foxtrot_prepare_merged = foxtrot_prepare_right - foxtrot_prepare_left;
    if foxtrot_prepare_merged > 18 {
        foxtrot_prepare_total = foxtrot_prepare_total + foxtrot_prepare_merged;
    }
    return foxtrot_prepare_total;
}

flow checked_index_capture_silver_silver_route(seed: i32) -> i32 ![]
{
    var foxtrot_route_total = seed * 21;
    var foxtrot_route_cursor = 0;
    while foxtrot_route_cursor < 7 limit Iterations(7) {
        foxtrot_route_total = foxtrot_route_total + foxtrot_route_cursor + 4;
        foxtrot_route_cursor = foxtrot_route_cursor + 1;
    }
    if foxtrot_route_total % 2 == 0 {
        foxtrot_route_total = foxtrot_route_total + 23;
    } else {
        foxtrot_route_total = foxtrot_route_total - 4;
    }
    var foxtrot_route_left = foxtrot_route_total + seed;
    var foxtrot_route_right = foxtrot_route_left * 4;
    var foxtrot_route_merged = foxtrot_route_right - foxtrot_route_left;
    if foxtrot_route_merged > 18 {
        foxtrot_route_total = foxtrot_route_total + foxtrot_route_merged;
    }
    return foxtrot_route_total;
}

flow checked_index_capture_silver_silver_score(seed: i32) -> i32 ![]
{
    var foxtrot_score_total = seed + 21;
    var foxtrot_score_cursor = 0;
    while foxtrot_score_cursor < 10 limit Iterations(10) {
        foxtrot_score_total = foxtrot_score_total + foxtrot_score_cursor + 4;
        foxtrot_score_cursor = foxtrot_score_cursor + 1;
    }
    if foxtrot_score_total % 2 == 0 {
        foxtrot_score_total = foxtrot_score_total + 23;
    } else {
        foxtrot_score_total = foxtrot_score_total - 4;
    }
    var foxtrot_score_left = foxtrot_score_total + seed;
    var foxtrot_score_right = foxtrot_score_left * 4;
    var foxtrot_score_merged = foxtrot_score_right - foxtrot_score_left;
    if foxtrot_score_merged > 18 {
        foxtrot_score_total = foxtrot_score_total + foxtrot_score_merged;
    }
    return foxtrot_score_total;
}

flow checked_index_capture_silver_silver_finish(seed: i32) -> i32 ![]
{
    var foxtrot_finish_total = seed - 21;
    var foxtrot_finish_cursor = 0;
    while foxtrot_finish_cursor < 7 limit Iterations(7) {
        foxtrot_finish_total = foxtrot_finish_total + foxtrot_finish_cursor + 4;
        foxtrot_finish_cursor = foxtrot_finish_cursor + 1;
    }
    if foxtrot_finish_total % 2 == 0 {
        foxtrot_finish_total = foxtrot_finish_total + 23;
    } else {
        foxtrot_finish_total = foxtrot_finish_total - 4;
    }
    var foxtrot_finish_left = foxtrot_finish_total + seed;
    var foxtrot_finish_right = foxtrot_finish_left * 4;
    var foxtrot_finish_merged = foxtrot_finish_right - foxtrot_finish_left;
    if foxtrot_finish_merged > 18 {
        foxtrot_finish_total = foxtrot_finish_total + foxtrot_finish_merged;
    }
    return foxtrot_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var silver_seed = 8;
    if args.len() > 0 {
        silver_seed = silver_seed + 1;
    } else {
        silver_seed = silver_seed + 2;
    }
    let silver_result = checked_index_capture_silver_silver_entry(silver_seed);
    if silver_result > 0 {
        return 0;
    }
    return 1;
}
