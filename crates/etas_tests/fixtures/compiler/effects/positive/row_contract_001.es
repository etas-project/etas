module tests.compiler.effects.positive.row_contract_001;

import std.io.{println};

flow row_contract_bravo_bravo_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var bravo_total = row_contract_bravo_bravo_prepare(seed);
    bravo_total = bravo_total + row_contract_bravo_bravo_route(seed + 2);
    println("row contract 0");
    let bravo_adjust: i32 -> i32 = (value: i32) => value + 2;
    bravo_total = bravo_adjust(bravo_total);
    bravo_total = bravo_total + row_contract_bravo_bravo_score(3);
    bravo_total = bravo_total + row_contract_bravo_bravo_finish(4);
    if bravo_total > 41 {
        bravo_total = bravo_total - 3;
    } else {
        bravo_total = bravo_total + 5;
    }
    return bravo_total;
}

flow row_contract_bravo_bravo_prepare(seed: i32) -> i32 ![]
{
    var kilo_prepare_total = seed + 4;
    var kilo_prepare_cursor = 0;
    while kilo_prepare_cursor < 9 limit Iterations(9) {
        kilo_prepare_total = kilo_prepare_total + kilo_prepare_cursor + 1;
        kilo_prepare_cursor = kilo_prepare_cursor + 1;
    }
    if kilo_prepare_total % 2 == 0 {
        kilo_prepare_total = kilo_prepare_total + row_contract_bravo_bravo_score(1);
    } else {
        kilo_prepare_total = kilo_prepare_total - 2;
    }
    var kilo_prepare_left = kilo_prepare_total + seed;
    var kilo_prepare_right = kilo_prepare_left * 3;
    var kilo_prepare_merged = kilo_prepare_right - kilo_prepare_left;
    if kilo_prepare_merged > 1 {
        kilo_prepare_total = kilo_prepare_total + kilo_prepare_merged;
    }
    return kilo_prepare_total;
}

flow row_contract_bravo_bravo_route(seed: i32) -> i32 ![]
{
    var kilo_route_total = seed * 4;
    var kilo_route_cursor = 0;
    while kilo_route_cursor < 8 limit Iterations(8) {
        kilo_route_total = kilo_route_total + kilo_route_cursor + 1;
        kilo_route_cursor = kilo_route_cursor + 1;
    }
    if kilo_route_total % 2 == 0 {
        kilo_route_total = kilo_route_total + 6;
    } else {
        kilo_route_total = kilo_route_total - 2;
    }
    var kilo_route_left = kilo_route_total + seed;
    var kilo_route_right = kilo_route_left * 3;
    var kilo_route_merged = kilo_route_right - kilo_route_left;
    if kilo_route_merged > 1 {
        kilo_route_total = kilo_route_total + kilo_route_merged;
    }
    return kilo_route_total;
}

flow row_contract_bravo_bravo_score(seed: i32) -> i32 ![]
{
    var kilo_score_total = seed + 4;
    var kilo_score_cursor = 0;
    while kilo_score_cursor < 7 limit Iterations(7) {
        kilo_score_total = kilo_score_total + kilo_score_cursor + 1;
        kilo_score_cursor = kilo_score_cursor + 1;
    }
    if kilo_score_total % 2 == 0 {
        kilo_score_total = kilo_score_total + 6;
    } else {
        kilo_score_total = kilo_score_total - 2;
    }
    var kilo_score_left = kilo_score_total + seed;
    var kilo_score_right = kilo_score_left * 3;
    var kilo_score_merged = kilo_score_right - kilo_score_left;
    if kilo_score_merged > 1 {
        kilo_score_total = kilo_score_total + kilo_score_merged;
    }
    return kilo_score_total;
}

flow row_contract_bravo_bravo_finish(seed: i32) -> i32 ![]
{
    var kilo_finish_total = seed - 4;
    var kilo_finish_cursor = 0;
    while kilo_finish_cursor < 6 limit Iterations(6) {
        kilo_finish_total = kilo_finish_total + kilo_finish_cursor + 1;
        kilo_finish_cursor = kilo_finish_cursor + 1;
    }
    if kilo_finish_total % 2 == 0 {
        kilo_finish_total = kilo_finish_total + 6;
    } else {
        kilo_finish_total = kilo_finish_total - 2;
    }
    var kilo_finish_left = kilo_finish_total + seed;
    var kilo_finish_right = kilo_finish_left * 3;
    var kilo_finish_merged = kilo_finish_right - kilo_finish_left;
    if kilo_finish_merged > 1 {
        kilo_finish_total = kilo_finish_total + kilo_finish_merged;
    }
    return kilo_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var bravo_seed = 2;
    if args.len() > 0 {
        bravo_seed = bravo_seed + 1;
    } else {
        bravo_seed = bravo_seed + 2;
    }
    let bravo_result = row_contract_bravo_bravo_entry(bravo_seed);
    if bravo_result > 0 {
        return 0;
    }
    return 1;
}
