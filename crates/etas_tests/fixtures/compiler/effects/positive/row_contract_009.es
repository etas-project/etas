module tests.compiler.effects.positive.row_contract_009;

import std.io.{println};

flow row_contract_juliet_juliet_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var juliet_total = row_contract_juliet_juliet_prepare(seed);
    juliet_total = juliet_total + row_contract_juliet_juliet_route(seed + 1);
    println("row contract 8");
    let juliet_adjust: i32 -> i32 = (value: i32) => value + 10;
    juliet_total = juliet_adjust(juliet_total);
    juliet_total = juliet_total + row_contract_juliet_juliet_score(6);
    juliet_total = juliet_total + row_contract_juliet_juliet_finish(5);
    if juliet_total > 49 {
        juliet_total = juliet_total - 11;
    } else {
        juliet_total = juliet_total + 13;
    }
    return juliet_total;
}

flow row_contract_juliet_juliet_prepare(seed: i32) -> i32 ![]
{
    var pulse_prepare_total = seed + 12;
    var pulse_prepare_cursor = 0;
    while pulse_prepare_cursor < 12 limit Iterations(12) {
        pulse_prepare_total = pulse_prepare_total + pulse_prepare_cursor + 2;
        pulse_prepare_cursor = pulse_prepare_cursor + 1;
    }
    if pulse_prepare_total % 2 == 0 {
        pulse_prepare_total = pulse_prepare_total + row_contract_juliet_juliet_score(1);
    } else {
        pulse_prepare_total = pulse_prepare_total - 5;
    }
    var pulse_prepare_left = pulse_prepare_total + seed;
    var pulse_prepare_right = pulse_prepare_left * 3;
    var pulse_prepare_merged = pulse_prepare_right - pulse_prepare_left;
    if pulse_prepare_merged > 9 {
        pulse_prepare_total = pulse_prepare_total + pulse_prepare_merged;
    }
    return pulse_prepare_total;
}

flow row_contract_juliet_juliet_route(seed: i32) -> i32 ![]
{
    var pulse_route_total = seed * 12;
    var pulse_route_cursor = 0;
    while pulse_route_cursor < 10 limit Iterations(10) {
        pulse_route_total = pulse_route_total + pulse_route_cursor + 2;
        pulse_route_cursor = pulse_route_cursor + 1;
    }
    if pulse_route_total % 2 == 0 {
        pulse_route_total = pulse_route_total + 14;
    } else {
        pulse_route_total = pulse_route_total - 5;
    }
    var pulse_route_left = pulse_route_total + seed;
    var pulse_route_right = pulse_route_left * 3;
    var pulse_route_merged = pulse_route_right - pulse_route_left;
    if pulse_route_merged > 9 {
        pulse_route_total = pulse_route_total + pulse_route_merged;
    }
    return pulse_route_total;
}

flow row_contract_juliet_juliet_score(seed: i32) -> i32 ![]
{
    var pulse_score_total = seed + 12;
    var pulse_score_cursor = 0;
    while pulse_score_cursor < 8 limit Iterations(8) {
        pulse_score_total = pulse_score_total + pulse_score_cursor + 2;
        pulse_score_cursor = pulse_score_cursor + 1;
    }
    if pulse_score_total % 2 == 0 {
        pulse_score_total = pulse_score_total + 14;
    } else {
        pulse_score_total = pulse_score_total - 5;
    }
    var pulse_score_left = pulse_score_total + seed;
    var pulse_score_right = pulse_score_left * 3;
    var pulse_score_merged = pulse_score_right - pulse_score_left;
    if pulse_score_merged > 9 {
        pulse_score_total = pulse_score_total + pulse_score_merged;
    }
    return pulse_score_total;
}

flow row_contract_juliet_juliet_finish(seed: i32) -> i32 ![]
{
    var pulse_finish_total = seed - 12;
    var pulse_finish_cursor = 0;
    while pulse_finish_cursor < 6 limit Iterations(6) {
        pulse_finish_total = pulse_finish_total + pulse_finish_cursor + 2;
        pulse_finish_cursor = pulse_finish_cursor + 1;
    }
    if pulse_finish_total % 2 == 0 {
        pulse_finish_total = pulse_finish_total + 14;
    } else {
        pulse_finish_total = pulse_finish_total - 5;
    }
    var pulse_finish_left = pulse_finish_total + seed;
    var pulse_finish_right = pulse_finish_left * 3;
    var pulse_finish_merged = pulse_finish_right - pulse_finish_left;
    if pulse_finish_merged > 9 {
        pulse_finish_total = pulse_finish_total + pulse_finish_merged;
    }
    return pulse_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var juliet_seed = 10;
    if args.len() > 0 {
        juliet_seed = juliet_seed + 1;
    } else {
        juliet_seed = juliet_seed + 2;
    }
    let juliet_result = row_contract_juliet_juliet_entry(juliet_seed);
    if juliet_result > 0 {
        return 0;
    }
    return 1;
}
