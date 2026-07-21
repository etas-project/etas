module tests.compiler.effects.positive.row_contract_010;

import std.io.{println};

flow row_contract_kilo_kilo_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var kilo_total = row_contract_kilo_kilo_prepare(seed);
    kilo_total = kilo_total + row_contract_kilo_kilo_route(seed + 2);
    println("row contract 9");
    let kilo_adjust: i32 -> i32 = (value: i32) => value + 11;
    kilo_total = kilo_adjust(kilo_total);
    kilo_total = kilo_total + row_contract_kilo_kilo_score(2);
    kilo_total = kilo_total + row_contract_kilo_kilo_finish(6);
    if kilo_total > 50 {
        kilo_total = kilo_total - 12;
    } else {
        kilo_total = kilo_total + 14;
    }
    return kilo_total;
}

flow row_contract_kilo_kilo_prepare(seed: i32) -> i32 ![]
{
    var yearling_prepare_total = seed + 13;
    var yearling_prepare_cursor = 0;
    while yearling_prepare_cursor < 8 limit Iterations(8) {
        yearling_prepare_total = yearling_prepare_total + yearling_prepare_cursor + 3;
        yearling_prepare_cursor = yearling_prepare_cursor + 1;
    }
    if yearling_prepare_total % 2 == 0 {
        yearling_prepare_total = yearling_prepare_total + row_contract_kilo_kilo_score(1);
    } else {
        yearling_prepare_total = yearling_prepare_total - 1;
    }
    var yearling_prepare_left = yearling_prepare_total + seed;
    var yearling_prepare_right = yearling_prepare_left * 4;
    var yearling_prepare_merged = yearling_prepare_right - yearling_prepare_left;
    if yearling_prepare_merged > 10 {
        yearling_prepare_total = yearling_prepare_total + yearling_prepare_merged;
    }
    return yearling_prepare_total;
}

flow row_contract_kilo_kilo_route(seed: i32) -> i32 ![]
{
    var yearling_route_total = seed * 13;
    var yearling_route_cursor = 0;
    while yearling_route_cursor < 11 limit Iterations(11) {
        yearling_route_total = yearling_route_total + yearling_route_cursor + 3;
        yearling_route_cursor = yearling_route_cursor + 1;
    }
    if yearling_route_total % 2 == 0 {
        yearling_route_total = yearling_route_total + 15;
    } else {
        yearling_route_total = yearling_route_total - 1;
    }
    var yearling_route_left = yearling_route_total + seed;
    var yearling_route_right = yearling_route_left * 4;
    var yearling_route_merged = yearling_route_right - yearling_route_left;
    if yearling_route_merged > 10 {
        yearling_route_total = yearling_route_total + yearling_route_merged;
    }
    return yearling_route_total;
}

flow row_contract_kilo_kilo_score(seed: i32) -> i32 ![]
{
    var yearling_score_total = seed + 13;
    var yearling_score_cursor = 0;
    while yearling_score_cursor < 9 limit Iterations(9) {
        yearling_score_total = yearling_score_total + yearling_score_cursor + 3;
        yearling_score_cursor = yearling_score_cursor + 1;
    }
    if yearling_score_total % 2 == 0 {
        yearling_score_total = yearling_score_total + 15;
    } else {
        yearling_score_total = yearling_score_total - 1;
    }
    var yearling_score_left = yearling_score_total + seed;
    var yearling_score_right = yearling_score_left * 4;
    var yearling_score_merged = yearling_score_right - yearling_score_left;
    if yearling_score_merged > 10 {
        yearling_score_total = yearling_score_total + yearling_score_merged;
    }
    return yearling_score_total;
}

flow row_contract_kilo_kilo_finish(seed: i32) -> i32 ![]
{
    var yearling_finish_total = seed - 13;
    var yearling_finish_cursor = 0;
    while yearling_finish_cursor < 7 limit Iterations(7) {
        yearling_finish_total = yearling_finish_total + yearling_finish_cursor + 3;
        yearling_finish_cursor = yearling_finish_cursor + 1;
    }
    if yearling_finish_total % 2 == 0 {
        yearling_finish_total = yearling_finish_total + 15;
    } else {
        yearling_finish_total = yearling_finish_total - 1;
    }
    var yearling_finish_left = yearling_finish_total + seed;
    var yearling_finish_right = yearling_finish_left * 4;
    var yearling_finish_merged = yearling_finish_right - yearling_finish_left;
    if yearling_finish_merged > 10 {
        yearling_finish_total = yearling_finish_total + yearling_finish_merged;
    }
    return yearling_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var kilo_seed = 11;
    if args.len() > 0 {
        kilo_seed = kilo_seed + 1;
    } else {
        kilo_seed = kilo_seed + 2;
    }
    let kilo_result = row_contract_kilo_kilo_entry(kilo_seed);
    if kilo_result > 0 {
        return 0;
    }
    return 1;
}
