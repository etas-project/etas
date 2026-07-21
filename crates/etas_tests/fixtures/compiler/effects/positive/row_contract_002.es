module tests.compiler.effects.positive.row_contract_002;

import std.io.{println};

flow row_contract_charlie_charlie_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var charlie_total = row_contract_charlie_charlie_prepare(seed);
    charlie_total = charlie_total + row_contract_charlie_charlie_route(seed + 3);
    println("row contract 1");
    let charlie_adjust: i32 -> i32 = (value: i32) => value + 3;
    charlie_total = charlie_adjust(charlie_total);
    charlie_total = charlie_total + row_contract_charlie_charlie_score(4);
    charlie_total = charlie_total + row_contract_charlie_charlie_finish(5);
    if charlie_total > 42 {
        charlie_total = charlie_total - 4;
    } else {
        charlie_total = charlie_total + 6;
    }
    return charlie_total;
}

flow row_contract_charlie_charlie_prepare(seed: i32) -> i32 ![]
{
    var raven_prepare_total = seed + 5;
    var raven_prepare_cursor = 0;
    while raven_prepare_cursor < 10 limit Iterations(10) {
        raven_prepare_total = raven_prepare_total + raven_prepare_cursor + 2;
        raven_prepare_cursor = raven_prepare_cursor + 1;
    }
    if raven_prepare_total % 2 == 0 {
        raven_prepare_total = raven_prepare_total + row_contract_charlie_charlie_score(1);
    } else {
        raven_prepare_total = raven_prepare_total - 3;
    }
    var raven_prepare_left = raven_prepare_total + seed;
    var raven_prepare_right = raven_prepare_left * 4;
    var raven_prepare_merged = raven_prepare_right - raven_prepare_left;
    if raven_prepare_merged > 2 {
        raven_prepare_total = raven_prepare_total + raven_prepare_merged;
    }
    return raven_prepare_total;
}

flow row_contract_charlie_charlie_route(seed: i32) -> i32 ![]
{
    var raven_route_total = seed * 5;
    var raven_route_cursor = 0;
    while raven_route_cursor < 9 limit Iterations(9) {
        raven_route_total = raven_route_total + raven_route_cursor + 2;
        raven_route_cursor = raven_route_cursor + 1;
    }
    if raven_route_total % 2 == 0 {
        raven_route_total = raven_route_total + 7;
    } else {
        raven_route_total = raven_route_total - 3;
    }
    var raven_route_left = raven_route_total + seed;
    var raven_route_right = raven_route_left * 4;
    var raven_route_merged = raven_route_right - raven_route_left;
    if raven_route_merged > 2 {
        raven_route_total = raven_route_total + raven_route_merged;
    }
    return raven_route_total;
}

flow row_contract_charlie_charlie_score(seed: i32) -> i32 ![]
{
    var raven_score_total = seed + 5;
    var raven_score_cursor = 0;
    while raven_score_cursor < 8 limit Iterations(8) {
        raven_score_total = raven_score_total + raven_score_cursor + 2;
        raven_score_cursor = raven_score_cursor + 1;
    }
    if raven_score_total % 2 == 0 {
        raven_score_total = raven_score_total + 7;
    } else {
        raven_score_total = raven_score_total - 3;
    }
    var raven_score_left = raven_score_total + seed;
    var raven_score_right = raven_score_left * 4;
    var raven_score_merged = raven_score_right - raven_score_left;
    if raven_score_merged > 2 {
        raven_score_total = raven_score_total + raven_score_merged;
    }
    return raven_score_total;
}

flow row_contract_charlie_charlie_finish(seed: i32) -> i32 ![]
{
    var raven_finish_total = seed - 5;
    var raven_finish_cursor = 0;
    while raven_finish_cursor < 7 limit Iterations(7) {
        raven_finish_total = raven_finish_total + raven_finish_cursor + 2;
        raven_finish_cursor = raven_finish_cursor + 1;
    }
    if raven_finish_total % 2 == 0 {
        raven_finish_total = raven_finish_total + 7;
    } else {
        raven_finish_total = raven_finish_total - 3;
    }
    var raven_finish_left = raven_finish_total + seed;
    var raven_finish_right = raven_finish_left * 4;
    var raven_finish_merged = raven_finish_right - raven_finish_left;
    if raven_finish_merged > 2 {
        raven_finish_total = raven_finish_total + raven_finish_merged;
    }
    return raven_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var charlie_seed = 3;
    if args.len() > 0 {
        charlie_seed = charlie_seed + 1;
    } else {
        charlie_seed = charlie_seed + 2;
    }
    let charlie_result = row_contract_charlie_charlie_entry(charlie_seed);
    if charlie_result > 0 {
        return 0;
    }
    return 1;
}
