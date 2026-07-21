module tests.compiler.effects.positive.row_contract_extra_094;

import std.io.{println};

flow row_contract_extra_temple_temple_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var temple_total = row_contract_extra_temple_temple_prepare(seed);
    temple_total = temple_total + row_contract_extra_temple_temple_route(seed + 5);
    println("public row extra 1");
    let temple_adjust: i32 -> i32 = (value: i32) => value + 4;
    temple_total = temple_adjust(temple_total);
    temple_total = temple_total + row_contract_extra_temple_temple_score(6);
    temple_total = temple_total + row_contract_extra_temple_temple_finish(6);
    if temple_total > 134 {
        temple_total = temple_total - 8;
    } else {
        temple_total = temple_total + 13;
    }
    return temple_total;
}

flow row_contract_extra_temple_temple_prepare(seed: i32) -> i32 ![]
{
    var prairie_prepare_total = seed + 21;
    var prairie_prepare_cursor = 0;
    while prairie_prepare_cursor < 12 limit Iterations(12) {
        prairie_prepare_total = prairie_prepare_total + prairie_prepare_cursor + 3;
        prairie_prepare_cursor = prairie_prepare_cursor + 1;
    }
    if prairie_prepare_total % 2 == 0 {
        prairie_prepare_total = prairie_prepare_total + row_contract_extra_temple_temple_score(1);
    } else {
        prairie_prepare_total = prairie_prepare_total - 5;
    }
    var prairie_prepare_left = prairie_prepare_total + seed;
    var prairie_prepare_right = prairie_prepare_left * 4;
    var prairie_prepare_merged = prairie_prepare_right - prairie_prepare_left;
    if prairie_prepare_merged > 1 {
        prairie_prepare_total = prairie_prepare_total + prairie_prepare_merged;
    }
    return prairie_prepare_total;
}

flow row_contract_extra_temple_temple_route(seed: i32) -> i32 ![]
{
    var prairie_route_total = seed * 21;
    var prairie_route_cursor = 0;
    while prairie_route_cursor < 11 limit Iterations(11) {
        prairie_route_total = prairie_route_total + prairie_route_cursor + 3;
        prairie_route_cursor = prairie_route_cursor + 1;
    }
    if prairie_route_total % 2 == 0 {
        prairie_route_total = prairie_route_total + 7;
    } else {
        prairie_route_total = prairie_route_total - 5;
    }
    var prairie_route_left = prairie_route_total + seed;
    var prairie_route_right = prairie_route_left * 4;
    var prairie_route_merged = prairie_route_right - prairie_route_left;
    if prairie_route_merged > 1 {
        prairie_route_total = prairie_route_total + prairie_route_merged;
    }
    return prairie_route_total;
}

flow row_contract_extra_temple_temple_score(seed: i32) -> i32 ![]
{
    var prairie_score_total = seed + 21;
    var prairie_score_cursor = 0;
    while prairie_score_cursor < 9 limit Iterations(9) {
        prairie_score_total = prairie_score_total + prairie_score_cursor + 3;
        prairie_score_cursor = prairie_score_cursor + 1;
    }
    if prairie_score_total % 2 == 0 {
        prairie_score_total = prairie_score_total + 7;
    } else {
        prairie_score_total = prairie_score_total - 5;
    }
    var prairie_score_left = prairie_score_total + seed;
    var prairie_score_right = prairie_score_left * 4;
    var prairie_score_merged = prairie_score_right - prairie_score_left;
    if prairie_score_merged > 1 {
        prairie_score_total = prairie_score_total + prairie_score_merged;
    }
    return prairie_score_total;
}

flow row_contract_extra_temple_temple_finish(seed: i32) -> i32 ![]
{
    var prairie_finish_total = seed - 21;
    var prairie_finish_cursor = 0;
    while prairie_finish_cursor < 11 limit Iterations(11) {
        prairie_finish_total = prairie_finish_total + prairie_finish_cursor + 3;
        prairie_finish_cursor = prairie_finish_cursor + 1;
    }
    if prairie_finish_total % 2 == 0 {
        prairie_finish_total = prairie_finish_total + 7;
    } else {
        prairie_finish_total = prairie_finish_total - 5;
    }
    var prairie_finish_left = prairie_finish_total + seed;
    var prairie_finish_right = prairie_finish_left * 4;
    var prairie_finish_merged = prairie_finish_right - prairie_finish_left;
    if prairie_finish_merged > 1 {
        prairie_finish_total = prairie_finish_total + prairie_finish_merged;
    }
    return prairie_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var temple_seed = 7;
    if args.len() > 0 {
        temple_seed = temple_seed + 1;
    } else {
        temple_seed = temple_seed + 2;
    }
    let temple_result = row_contract_extra_temple_temple_entry(temple_seed);
    if temple_result > 0 {
        return 0;
    }
    return 1;
}
