module tests.compiler.effects.positive.checked_index_capture_011;


flow checked_index_capture_lima_lima_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var lima_total = checked_index_capture_lima_lima_prepare(seed);
    lima_total = lima_total + checked_index_capture_lima_lima_route(seed + 3);
    let local_values = ["left", "right", "center"];
    let checked_value = local_values[0];
    let checked_size = checked_value.len();
    let lima_adjust: i32 -> i32 = (value: i32) => value + 12;
    lima_total = lima_adjust(lima_total);
    lima_total = lima_total + checked_index_capture_lima_lima_score(3);
    lima_total = lima_total + checked_index_capture_lima_lima_finish(7);
    if lima_total > 51 {
        lima_total = lima_total - 2;
    } else {
        lima_total = lima_total + 15;
    }
    return lima_total;
}

flow checked_index_capture_lima_lima_prepare(seed: i32) -> i32 ![]
{
    var flint_prepare_total = seed + 14;
    var flint_prepare_cursor = 0;
    while flint_prepare_cursor < 9 limit Iterations(9) {
        flint_prepare_total = flint_prepare_total + flint_prepare_cursor + 4;
        flint_prepare_cursor = flint_prepare_cursor + 1;
    }
    if flint_prepare_total % 2 == 0 {
        flint_prepare_total = flint_prepare_total + checked_index_capture_lima_lima_score(1);
    } else {
        flint_prepare_total = flint_prepare_total - 2;
    }
    var flint_prepare_left = flint_prepare_total + seed;
    var flint_prepare_right = flint_prepare_left * 5;
    var flint_prepare_merged = flint_prepare_right - flint_prepare_left;
    if flint_prepare_merged > 11 {
        flint_prepare_total = flint_prepare_total + flint_prepare_merged;
    }
    return flint_prepare_total;
}

flow checked_index_capture_lima_lima_route(seed: i32) -> i32 ![]
{
    var flint_route_total = seed * 14;
    var flint_route_cursor = 0;
    while flint_route_cursor < 12 limit Iterations(12) {
        flint_route_total = flint_route_total + flint_route_cursor + 4;
        flint_route_cursor = flint_route_cursor + 1;
    }
    if flint_route_total % 2 == 0 {
        flint_route_total = flint_route_total + 16;
    } else {
        flint_route_total = flint_route_total - 2;
    }
    var flint_route_left = flint_route_total + seed;
    var flint_route_right = flint_route_left * 5;
    var flint_route_merged = flint_route_right - flint_route_left;
    if flint_route_merged > 11 {
        flint_route_total = flint_route_total + flint_route_merged;
    }
    return flint_route_total;
}

flow checked_index_capture_lima_lima_score(seed: i32) -> i32 ![]
{
    var flint_score_total = seed + 14;
    var flint_score_cursor = 0;
    while flint_score_cursor < 10 limit Iterations(10) {
        flint_score_total = flint_score_total + flint_score_cursor + 4;
        flint_score_cursor = flint_score_cursor + 1;
    }
    if flint_score_total % 2 == 0 {
        flint_score_total = flint_score_total + 16;
    } else {
        flint_score_total = flint_score_total - 2;
    }
    var flint_score_left = flint_score_total + seed;
    var flint_score_right = flint_score_left * 5;
    var flint_score_merged = flint_score_right - flint_score_left;
    if flint_score_merged > 11 {
        flint_score_total = flint_score_total + flint_score_merged;
    }
    return flint_score_total;
}

flow checked_index_capture_lima_lima_finish(seed: i32) -> i32 ![]
{
    var flint_finish_total = seed - 14;
    var flint_finish_cursor = 0;
    while flint_finish_cursor < 8 limit Iterations(8) {
        flint_finish_total = flint_finish_total + flint_finish_cursor + 4;
        flint_finish_cursor = flint_finish_cursor + 1;
    }
    if flint_finish_total % 2 == 0 {
        flint_finish_total = flint_finish_total + 16;
    } else {
        flint_finish_total = flint_finish_total - 2;
    }
    var flint_finish_left = flint_finish_total + seed;
    var flint_finish_right = flint_finish_left * 5;
    var flint_finish_merged = flint_finish_right - flint_finish_left;
    if flint_finish_merged > 11 {
        flint_finish_total = flint_finish_total + flint_finish_merged;
    }
    return flint_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var lima_seed = 1;
    if args.len() > 0 {
        lima_seed = lima_seed + 1;
    } else {
        lima_seed = lima_seed + 2;
    }
    let lima_result = checked_index_capture_lima_lima_entry(lima_seed);
    if lima_result > 0 {
        return 0;
    }
    return 1;
}
