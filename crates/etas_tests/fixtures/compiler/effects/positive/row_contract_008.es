module tests.compiler.effects.positive.row_contract_008;

import std.io.{println};

flow row_contract_india_india_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var india_total = row_contract_india_india_prepare(seed);
    india_total = india_total + row_contract_india_india_route(seed + 9);
    println("row contract 7");
    let india_adjust: i32 -> i32 = (value: i32) => value + 9;
    india_total = india_adjust(india_total);
    india_total = india_total + row_contract_india_india_score(5);
    india_total = india_total + row_contract_india_india_finish(4);
    if india_total > 48 {
        india_total = india_total - 10;
    } else {
        india_total = india_total + 12;
    }
    return india_total;
}

flow row_contract_india_india_prepare(seed: i32) -> i32 ![]
{
    var islet_prepare_total = seed + 11;
    var islet_prepare_cursor = 0;
    while islet_prepare_cursor < 11 limit Iterations(11) {
        islet_prepare_total = islet_prepare_total + islet_prepare_cursor + 1;
        islet_prepare_cursor = islet_prepare_cursor + 1;
    }
    if islet_prepare_total % 2 == 0 {
        islet_prepare_total = islet_prepare_total + row_contract_india_india_score(1);
    } else {
        islet_prepare_total = islet_prepare_total - 4;
    }
    var islet_prepare_left = islet_prepare_total + seed;
    var islet_prepare_right = islet_prepare_left * 2;
    var islet_prepare_merged = islet_prepare_right - islet_prepare_left;
    if islet_prepare_merged > 8 {
        islet_prepare_total = islet_prepare_total + islet_prepare_merged;
    }
    return islet_prepare_total;
}

flow row_contract_india_india_route(seed: i32) -> i32 ![]
{
    var islet_route_total = seed * 11;
    var islet_route_cursor = 0;
    while islet_route_cursor < 9 limit Iterations(9) {
        islet_route_total = islet_route_total + islet_route_cursor + 1;
        islet_route_cursor = islet_route_cursor + 1;
    }
    if islet_route_total % 2 == 0 {
        islet_route_total = islet_route_total + 13;
    } else {
        islet_route_total = islet_route_total - 4;
    }
    var islet_route_left = islet_route_total + seed;
    var islet_route_right = islet_route_left * 2;
    var islet_route_merged = islet_route_right - islet_route_left;
    if islet_route_merged > 8 {
        islet_route_total = islet_route_total + islet_route_merged;
    }
    return islet_route_total;
}

flow row_contract_india_india_score(seed: i32) -> i32 ![]
{
    var islet_score_total = seed + 11;
    var islet_score_cursor = 0;
    while islet_score_cursor < 7 limit Iterations(7) {
        islet_score_total = islet_score_total + islet_score_cursor + 1;
        islet_score_cursor = islet_score_cursor + 1;
    }
    if islet_score_total % 2 == 0 {
        islet_score_total = islet_score_total + 13;
    } else {
        islet_score_total = islet_score_total - 4;
    }
    var islet_score_left = islet_score_total + seed;
    var islet_score_right = islet_score_left * 2;
    var islet_score_merged = islet_score_right - islet_score_left;
    if islet_score_merged > 8 {
        islet_score_total = islet_score_total + islet_score_merged;
    }
    return islet_score_total;
}

flow row_contract_india_india_finish(seed: i32) -> i32 ![]
{
    var islet_finish_total = seed - 11;
    var islet_finish_cursor = 0;
    while islet_finish_cursor < 5 limit Iterations(5) {
        islet_finish_total = islet_finish_total + islet_finish_cursor + 1;
        islet_finish_cursor = islet_finish_cursor + 1;
    }
    if islet_finish_total % 2 == 0 {
        islet_finish_total = islet_finish_total + 13;
    } else {
        islet_finish_total = islet_finish_total - 4;
    }
    var islet_finish_left = islet_finish_total + seed;
    var islet_finish_right = islet_finish_left * 2;
    var islet_finish_merged = islet_finish_right - islet_finish_left;
    if islet_finish_merged > 8 {
        islet_finish_total = islet_finish_total + islet_finish_merged;
    }
    return islet_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var india_seed = 9;
    if args.len() > 0 {
        india_seed = india_seed + 1;
    } else {
        india_seed = india_seed + 2;
    }
    let india_result = row_contract_india_india_entry(india_seed);
    if india_result > 0 {
        return 0;
    }
    return 1;
}
