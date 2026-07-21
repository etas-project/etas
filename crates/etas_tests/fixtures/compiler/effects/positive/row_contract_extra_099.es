module tests.compiler.effects.positive.row_contract_extra_099;

import std.io.{println};

flow row_contract_extra_zodiac_zodiac_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var zodiac_total = row_contract_extra_zodiac_zodiac_prepare(seed);
    zodiac_total = zodiac_total + row_contract_extra_zodiac_zodiac_route(seed + 1);
    println("public row extra 6");
    let zodiac_adjust: i32 -> i32 = (value: i32) => value + 9;
    zodiac_total = zodiac_adjust(zodiac_total);
    zodiac_total = zodiac_total + row_contract_extra_zodiac_zodiac_score(6);
    zodiac_total = zodiac_total + row_contract_extra_zodiac_zodiac_finish(4);
    if zodiac_total > 139 {
        zodiac_total = zodiac_total - 2;
    } else {
        zodiac_total = zodiac_total + 18;
    }
    return zodiac_total;
}

flow row_contract_extra_zodiac_zodiac_prepare(seed: i32) -> i32 ![]
{
    var beacon_prepare_total = seed + 7;
    var beacon_prepare_cursor = 0;
    while beacon_prepare_cursor < 12 limit Iterations(12) {
        beacon_prepare_total = beacon_prepare_total + beacon_prepare_cursor + 1;
        beacon_prepare_cursor = beacon_prepare_cursor + 1;
    }
    if beacon_prepare_total % 2 == 0 {
        beacon_prepare_total = beacon_prepare_total + row_contract_extra_zodiac_zodiac_score(1);
    } else {
        beacon_prepare_total = beacon_prepare_total - 5;
    }
    var beacon_prepare_left = beacon_prepare_total + seed;
    var beacon_prepare_right = beacon_prepare_left * 5;
    var beacon_prepare_merged = beacon_prepare_right - beacon_prepare_left;
    if beacon_prepare_merged > 6 {
        beacon_prepare_total = beacon_prepare_total + beacon_prepare_merged;
    }
    return beacon_prepare_total;
}

flow row_contract_extra_zodiac_zodiac_route(seed: i32) -> i32 ![]
{
    var beacon_route_total = seed * 7;
    var beacon_route_cursor = 0;
    while beacon_route_cursor < 10 limit Iterations(10) {
        beacon_route_total = beacon_route_total + beacon_route_cursor + 1;
        beacon_route_cursor = beacon_route_cursor + 1;
    }
    if beacon_route_total % 2 == 0 {
        beacon_route_total = beacon_route_total + 12;
    } else {
        beacon_route_total = beacon_route_total - 5;
    }
    var beacon_route_left = beacon_route_total + seed;
    var beacon_route_right = beacon_route_left * 5;
    var beacon_route_merged = beacon_route_right - beacon_route_left;
    if beacon_route_merged > 6 {
        beacon_route_total = beacon_route_total + beacon_route_merged;
    }
    return beacon_route_total;
}

flow row_contract_extra_zodiac_zodiac_score(seed: i32) -> i32 ![]
{
    var beacon_score_total = seed + 7;
    var beacon_score_cursor = 0;
    while beacon_score_cursor < 7 limit Iterations(7) {
        beacon_score_total = beacon_score_total + beacon_score_cursor + 1;
        beacon_score_cursor = beacon_score_cursor + 1;
    }
    if beacon_score_total % 2 == 0 {
        beacon_score_total = beacon_score_total + 12;
    } else {
        beacon_score_total = beacon_score_total - 5;
    }
    var beacon_score_left = beacon_score_total + seed;
    var beacon_score_right = beacon_score_left * 5;
    var beacon_score_merged = beacon_score_right - beacon_score_left;
    if beacon_score_merged > 6 {
        beacon_score_total = beacon_score_total + beacon_score_merged;
    }
    return beacon_score_total;
}

flow row_contract_extra_zodiac_zodiac_finish(seed: i32) -> i32 ![]
{
    var beacon_finish_total = seed - 7;
    var beacon_finish_cursor = 0;
    while beacon_finish_cursor < 8 limit Iterations(8) {
        beacon_finish_total = beacon_finish_total + beacon_finish_cursor + 1;
        beacon_finish_cursor = beacon_finish_cursor + 1;
    }
    if beacon_finish_total % 2 == 0 {
        beacon_finish_total = beacon_finish_total + 12;
    } else {
        beacon_finish_total = beacon_finish_total - 5;
    }
    var beacon_finish_left = beacon_finish_total + seed;
    var beacon_finish_right = beacon_finish_left * 5;
    var beacon_finish_merged = beacon_finish_right - beacon_finish_left;
    if beacon_finish_merged > 6 {
        beacon_finish_total = beacon_finish_total + beacon_finish_merged;
    }
    return beacon_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var zodiac_seed = 1;
    if args.len() > 0 {
        zodiac_seed = zodiac_seed + 1;
    } else {
        zodiac_seed = zodiac_seed + 2;
    }
    let zodiac_result = row_contract_extra_zodiac_zodiac_entry(zodiac_seed);
    if zodiac_result > 0 {
        return 0;
    }
    return 1;
}
