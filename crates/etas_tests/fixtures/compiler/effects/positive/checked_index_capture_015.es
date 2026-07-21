module tests.compiler.effects.positive.checked_index_capture_015;


flow checked_index_capture_pearl_pearl_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var pearl_total = checked_index_capture_pearl_pearl_prepare(seed);
    pearl_total = pearl_total + checked_index_capture_pearl_pearl_route(seed + 7);
    let local_values = ["left", "right", "center"];
    let checked_value = local_values[0];
    let checked_size = checked_value.len();
    let pearl_adjust: i32 -> i32 = (value: i32) => value + 3;
    pearl_total = pearl_adjust(pearl_total);
    pearl_total = pearl_total + checked_index_capture_pearl_pearl_score(2);
    pearl_total = pearl_total + checked_index_capture_pearl_pearl_finish(4);
    if pearl_total > 55 {
        pearl_total = pearl_total - 6;
    } else {
        pearl_total = pearl_total + 19;
    }
    return pearl_total;
}

flow checked_index_capture_pearl_pearl_prepare(seed: i32) -> i32 ![]
{
    var isotope_prepare_total = seed + 18;
    var isotope_prepare_cursor = 0;
    while isotope_prepare_cursor < 8 limit Iterations(8) {
        isotope_prepare_total = isotope_prepare_total + isotope_prepare_cursor + 1;
        isotope_prepare_cursor = isotope_prepare_cursor + 1;
    }
    if isotope_prepare_total % 2 == 0 {
        isotope_prepare_total = isotope_prepare_total + checked_index_capture_pearl_pearl_score(1);
    } else {
        isotope_prepare_total = isotope_prepare_total - 1;
    }
    var isotope_prepare_left = isotope_prepare_total + seed;
    var isotope_prepare_right = isotope_prepare_left * 5;
    var isotope_prepare_merged = isotope_prepare_right - isotope_prepare_left;
    if isotope_prepare_merged > 15 {
        isotope_prepare_total = isotope_prepare_total + isotope_prepare_merged;
    }
    return isotope_prepare_total;
}

flow checked_index_capture_pearl_pearl_route(seed: i32) -> i32 ![]
{
    var isotope_route_total = seed * 18;
    var isotope_route_cursor = 0;
    while isotope_route_cursor < 10 limit Iterations(10) {
        isotope_route_total = isotope_route_total + isotope_route_cursor + 1;
        isotope_route_cursor = isotope_route_cursor + 1;
    }
    if isotope_route_total % 2 == 0 {
        isotope_route_total = isotope_route_total + 20;
    } else {
        isotope_route_total = isotope_route_total - 1;
    }
    var isotope_route_left = isotope_route_total + seed;
    var isotope_route_right = isotope_route_left * 5;
    var isotope_route_merged = isotope_route_right - isotope_route_left;
    if isotope_route_merged > 15 {
        isotope_route_total = isotope_route_total + isotope_route_merged;
    }
    return isotope_route_total;
}

flow checked_index_capture_pearl_pearl_score(seed: i32) -> i32 ![]
{
    var isotope_score_total = seed + 18;
    var isotope_score_cursor = 0;
    while isotope_score_cursor < 7 limit Iterations(7) {
        isotope_score_total = isotope_score_total + isotope_score_cursor + 1;
        isotope_score_cursor = isotope_score_cursor + 1;
    }
    if isotope_score_total % 2 == 0 {
        isotope_score_total = isotope_score_total + 20;
    } else {
        isotope_score_total = isotope_score_total - 1;
    }
    var isotope_score_left = isotope_score_total + seed;
    var isotope_score_right = isotope_score_left * 5;
    var isotope_score_merged = isotope_score_right - isotope_score_left;
    if isotope_score_merged > 15 {
        isotope_score_total = isotope_score_total + isotope_score_merged;
    }
    return isotope_score_total;
}

flow checked_index_capture_pearl_pearl_finish(seed: i32) -> i32 ![]
{
    var isotope_finish_total = seed - 18;
    var isotope_finish_cursor = 0;
    while isotope_finish_cursor < 12 limit Iterations(12) {
        isotope_finish_total = isotope_finish_total + isotope_finish_cursor + 1;
        isotope_finish_cursor = isotope_finish_cursor + 1;
    }
    if isotope_finish_total % 2 == 0 {
        isotope_finish_total = isotope_finish_total + 20;
    } else {
        isotope_finish_total = isotope_finish_total - 1;
    }
    var isotope_finish_left = isotope_finish_total + seed;
    var isotope_finish_right = isotope_finish_left * 5;
    var isotope_finish_merged = isotope_finish_right - isotope_finish_left;
    if isotope_finish_merged > 15 {
        isotope_finish_total = isotope_finish_total + isotope_finish_merged;
    }
    return isotope_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var pearl_seed = 5;
    if args.len() > 0 {
        pearl_seed = pearl_seed + 1;
    } else {
        pearl_seed = pearl_seed + 2;
    }
    let pearl_result = checked_index_capture_pearl_pearl_entry(pearl_seed);
    if pearl_result > 0 {
        return 0;
    }
    return 1;
}
