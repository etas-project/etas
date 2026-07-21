module tests.compiler.effects.positive.row_contract_extra_096;

import std.io.{println};

flow row_contract_extra_valley_valley_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var valley_total = row_contract_extra_valley_valley_prepare(seed);
    valley_total = valley_total + row_contract_extra_valley_valley_route(seed + 7);
    println("public row extra 3");
    let valley_adjust: i32 -> i32 = (value: i32) => value + 6;
    valley_total = valley_adjust(valley_total);
    valley_total = valley_total + row_contract_extra_valley_valley_score(3);
    valley_total = valley_total + row_contract_extra_valley_valley_finish(8);
    if valley_total > 136 {
        valley_total = valley_total - 10;
    } else {
        valley_total = valley_total + 15;
    }
    return valley_total;
}

flow row_contract_extra_valley_valley_prepare(seed: i32) -> i32 ![]
{
    var equinox_prepare_total = seed + 4;
    var equinox_prepare_cursor = 0;
    while equinox_prepare_cursor < 9 limit Iterations(9) {
        equinox_prepare_total = equinox_prepare_total + equinox_prepare_cursor + 5;
        equinox_prepare_cursor = equinox_prepare_cursor + 1;
    }
    if equinox_prepare_total % 2 == 0 {
        equinox_prepare_total = equinox_prepare_total + row_contract_extra_valley_valley_score(1);
    } else {
        equinox_prepare_total = equinox_prepare_total - 2;
    }
    var equinox_prepare_left = equinox_prepare_total + seed;
    var equinox_prepare_right = equinox_prepare_left * 2;
    var equinox_prepare_merged = equinox_prepare_right - equinox_prepare_left;
    if equinox_prepare_merged > 3 {
        equinox_prepare_total = equinox_prepare_total + equinox_prepare_merged;
    }
    return equinox_prepare_total;
}

flow row_contract_extra_valley_valley_route(seed: i32) -> i32 ![]
{
    var equinox_route_total = seed * 4;
    var equinox_route_cursor = 0;
    while equinox_route_cursor < 7 limit Iterations(7) {
        equinox_route_total = equinox_route_total + equinox_route_cursor + 5;
        equinox_route_cursor = equinox_route_cursor + 1;
    }
    if equinox_route_total % 2 == 0 {
        equinox_route_total = equinox_route_total + 9;
    } else {
        equinox_route_total = equinox_route_total - 2;
    }
    var equinox_route_left = equinox_route_total + seed;
    var equinox_route_right = equinox_route_left * 2;
    var equinox_route_merged = equinox_route_right - equinox_route_left;
    if equinox_route_merged > 3 {
        equinox_route_total = equinox_route_total + equinox_route_merged;
    }
    return equinox_route_total;
}

flow row_contract_extra_valley_valley_score(seed: i32) -> i32 ![]
{
    var equinox_score_total = seed + 4;
    var equinox_score_cursor = 0;
    while equinox_score_cursor < 11 limit Iterations(11) {
        equinox_score_total = equinox_score_total + equinox_score_cursor + 5;
        equinox_score_cursor = equinox_score_cursor + 1;
    }
    if equinox_score_total % 2 == 0 {
        equinox_score_total = equinox_score_total + 9;
    } else {
        equinox_score_total = equinox_score_total - 2;
    }
    var equinox_score_left = equinox_score_total + seed;
    var equinox_score_right = equinox_score_left * 2;
    var equinox_score_merged = equinox_score_right - equinox_score_left;
    if equinox_score_merged > 3 {
        equinox_score_total = equinox_score_total + equinox_score_merged;
    }
    return equinox_score_total;
}

flow row_contract_extra_valley_valley_finish(seed: i32) -> i32 ![]
{
    var equinox_finish_total = seed - 4;
    var equinox_finish_cursor = 0;
    while equinox_finish_cursor < 5 limit Iterations(5) {
        equinox_finish_total = equinox_finish_total + equinox_finish_cursor + 5;
        equinox_finish_cursor = equinox_finish_cursor + 1;
    }
    if equinox_finish_total % 2 == 0 {
        equinox_finish_total = equinox_finish_total + 9;
    } else {
        equinox_finish_total = equinox_finish_total - 2;
    }
    var equinox_finish_left = equinox_finish_total + seed;
    var equinox_finish_right = equinox_finish_left * 2;
    var equinox_finish_merged = equinox_finish_right - equinox_finish_left;
    if equinox_finish_merged > 3 {
        equinox_finish_total = equinox_finish_total + equinox_finish_merged;
    }
    return equinox_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var valley_seed = 9;
    if args.len() > 0 {
        valley_seed = valley_seed + 1;
    } else {
        valley_seed = valley_seed + 2;
    }
    let valley_result = row_contract_extra_valley_valley_entry(valley_seed);
    if valley_result > 0 {
        return 0;
    }
    return 1;
}
