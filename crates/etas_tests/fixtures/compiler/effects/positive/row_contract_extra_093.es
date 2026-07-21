module tests.compiler.effects.positive.row_contract_extra_093;

import std.io.{println};

flow row_contract_extra_saffron_saffron_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var saffron_total = row_contract_extra_saffron_saffron_prepare(seed);
    saffron_total = saffron_total + row_contract_extra_saffron_saffron_route(seed + 4);
    println("public row extra 0");
    let saffron_adjust: i32 -> i32 = (value: i32) => value + 3;
    saffron_total = saffron_adjust(saffron_total);
    saffron_total = saffron_total + row_contract_extra_saffron_saffron_score(5);
    saffron_total = saffron_total + row_contract_extra_saffron_saffron_finish(5);
    if saffron_total > 133 {
        saffron_total = saffron_total - 7;
    } else {
        saffron_total = saffron_total + 12;
    }
    return saffron_total;
}

flow row_contract_extra_saffron_saffron_prepare(seed: i32) -> i32 ![]
{
    var ion_prepare_total = seed + 20;
    var ion_prepare_cursor = 0;
    while ion_prepare_cursor < 11 limit Iterations(11) {
        ion_prepare_total = ion_prepare_total + ion_prepare_cursor + 2;
        ion_prepare_cursor = ion_prepare_cursor + 1;
    }
    if ion_prepare_total % 2 == 0 {
        ion_prepare_total = ion_prepare_total + row_contract_extra_saffron_saffron_score(1);
    } else {
        ion_prepare_total = ion_prepare_total - 4;
    }
    var ion_prepare_left = ion_prepare_total + seed;
    var ion_prepare_right = ion_prepare_left * 3;
    var ion_prepare_merged = ion_prepare_right - ion_prepare_left;
    if ion_prepare_merged > 0 {
        ion_prepare_total = ion_prepare_total + ion_prepare_merged;
    }
    return ion_prepare_total;
}

flow row_contract_extra_saffron_saffron_route(seed: i32) -> i32 ![]
{
    var ion_route_total = seed * 20;
    var ion_route_cursor = 0;
    while ion_route_cursor < 10 limit Iterations(10) {
        ion_route_total = ion_route_total + ion_route_cursor + 2;
        ion_route_cursor = ion_route_cursor + 1;
    }
    if ion_route_total % 2 == 0 {
        ion_route_total = ion_route_total + 6;
    } else {
        ion_route_total = ion_route_total - 4;
    }
    var ion_route_left = ion_route_total + seed;
    var ion_route_right = ion_route_left * 3;
    var ion_route_merged = ion_route_right - ion_route_left;
    if ion_route_merged > 0 {
        ion_route_total = ion_route_total + ion_route_merged;
    }
    return ion_route_total;
}

flow row_contract_extra_saffron_saffron_score(seed: i32) -> i32 ![]
{
    var ion_score_total = seed + 20;
    var ion_score_cursor = 0;
    while ion_score_cursor < 8 limit Iterations(8) {
        ion_score_total = ion_score_total + ion_score_cursor + 2;
        ion_score_cursor = ion_score_cursor + 1;
    }
    if ion_score_total % 2 == 0 {
        ion_score_total = ion_score_total + 6;
    } else {
        ion_score_total = ion_score_total - 4;
    }
    var ion_score_left = ion_score_total + seed;
    var ion_score_right = ion_score_left * 3;
    var ion_score_merged = ion_score_right - ion_score_left;
    if ion_score_merged > 0 {
        ion_score_total = ion_score_total + ion_score_merged;
    }
    return ion_score_total;
}

flow row_contract_extra_saffron_saffron_finish(seed: i32) -> i32 ![]
{
    var ion_finish_total = seed - 20;
    var ion_finish_cursor = 0;
    while ion_finish_cursor < 10 limit Iterations(10) {
        ion_finish_total = ion_finish_total + ion_finish_cursor + 2;
        ion_finish_cursor = ion_finish_cursor + 1;
    }
    if ion_finish_total % 2 == 0 {
        ion_finish_total = ion_finish_total + 6;
    } else {
        ion_finish_total = ion_finish_total - 4;
    }
    var ion_finish_left = ion_finish_total + seed;
    var ion_finish_right = ion_finish_left * 3;
    var ion_finish_merged = ion_finish_right - ion_finish_left;
    if ion_finish_merged > 0 {
        ion_finish_total = ion_finish_total + ion_finish_merged;
    }
    return ion_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var saffron_seed = 6;
    if args.len() > 0 {
        saffron_seed = saffron_seed + 1;
    } else {
        saffron_seed = saffron_seed + 2;
    }
    let saffron_result = row_contract_extra_saffron_saffron_entry(saffron_seed);
    if saffron_result > 0 {
        return 0;
    }
    return 1;
}
