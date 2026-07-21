module tests.compiler.effects.positive.row_contract_extra_097;

import std.io.{println};

flow row_contract_extra_window_window_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var window_total = row_contract_extra_window_window_prepare(seed);
    window_total = window_total + row_contract_extra_window_window_route(seed + 8);
    println("public row extra 4");
    let window_adjust: i32 -> i32 = (value: i32) => value + 7;
    window_total = window_adjust(window_total);
    window_total = window_total + row_contract_extra_window_window_score(4);
    window_total = window_total + row_contract_extra_window_window_finish(9);
    if window_total > 137 {
        window_total = window_total - 11;
    } else {
        window_total = window_total + 16;
    }
    return window_total;
}

flow row_contract_extra_window_window_prepare(seed: i32) -> i32 ![]
{
    var lotus_prepare_total = seed + 5;
    var lotus_prepare_cursor = 0;
    while lotus_prepare_cursor < 10 limit Iterations(10) {
        lotus_prepare_total = lotus_prepare_total + lotus_prepare_cursor + 6;
        lotus_prepare_cursor = lotus_prepare_cursor + 1;
    }
    if lotus_prepare_total % 2 == 0 {
        lotus_prepare_total = lotus_prepare_total + row_contract_extra_window_window_score(1);
    } else {
        lotus_prepare_total = lotus_prepare_total - 3;
    }
    var lotus_prepare_left = lotus_prepare_total + seed;
    var lotus_prepare_right = lotus_prepare_left * 3;
    var lotus_prepare_merged = lotus_prepare_right - lotus_prepare_left;
    if lotus_prepare_merged > 4 {
        lotus_prepare_total = lotus_prepare_total + lotus_prepare_merged;
    }
    return lotus_prepare_total;
}

flow row_contract_extra_window_window_route(seed: i32) -> i32 ![]
{
    var lotus_route_total = seed * 5;
    var lotus_route_cursor = 0;
    while lotus_route_cursor < 8 limit Iterations(8) {
        lotus_route_total = lotus_route_total + lotus_route_cursor + 6;
        lotus_route_cursor = lotus_route_cursor + 1;
    }
    if lotus_route_total % 2 == 0 {
        lotus_route_total = lotus_route_total + 10;
    } else {
        lotus_route_total = lotus_route_total - 3;
    }
    var lotus_route_left = lotus_route_total + seed;
    var lotus_route_right = lotus_route_left * 3;
    var lotus_route_merged = lotus_route_right - lotus_route_left;
    if lotus_route_merged > 4 {
        lotus_route_total = lotus_route_total + lotus_route_merged;
    }
    return lotus_route_total;
}

flow row_contract_extra_window_window_score(seed: i32) -> i32 ![]
{
    var lotus_score_total = seed + 5;
    var lotus_score_cursor = 0;
    while lotus_score_cursor < 12 limit Iterations(12) {
        lotus_score_total = lotus_score_total + lotus_score_cursor + 6;
        lotus_score_cursor = lotus_score_cursor + 1;
    }
    if lotus_score_total % 2 == 0 {
        lotus_score_total = lotus_score_total + 10;
    } else {
        lotus_score_total = lotus_score_total - 3;
    }
    var lotus_score_left = lotus_score_total + seed;
    var lotus_score_right = lotus_score_left * 3;
    var lotus_score_merged = lotus_score_right - lotus_score_left;
    if lotus_score_merged > 4 {
        lotus_score_total = lotus_score_total + lotus_score_merged;
    }
    return lotus_score_total;
}

flow row_contract_extra_window_window_finish(seed: i32) -> i32 ![]
{
    var lotus_finish_total = seed - 5;
    var lotus_finish_cursor = 0;
    while lotus_finish_cursor < 6 limit Iterations(6) {
        lotus_finish_total = lotus_finish_total + lotus_finish_cursor + 6;
        lotus_finish_cursor = lotus_finish_cursor + 1;
    }
    if lotus_finish_total % 2 == 0 {
        lotus_finish_total = lotus_finish_total + 10;
    } else {
        lotus_finish_total = lotus_finish_total - 3;
    }
    var lotus_finish_left = lotus_finish_total + seed;
    var lotus_finish_right = lotus_finish_left * 3;
    var lotus_finish_merged = lotus_finish_right - lotus_finish_left;
    if lotus_finish_merged > 4 {
        lotus_finish_total = lotus_finish_total + lotus_finish_merged;
    }
    return lotus_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var window_seed = 10;
    if args.len() > 0 {
        window_seed = window_seed + 1;
    } else {
        window_seed = window_seed + 2;
    }
    let window_result = row_contract_extra_window_window_entry(window_seed);
    if window_result > 0 {
        return 0;
    }
    return 1;
}
