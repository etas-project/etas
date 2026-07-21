module tests.compiler.effects.positive.row_contract_006;

import std.io.{println};

flow row_contract_golf_golf_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var golf_total = row_contract_golf_golf_prepare(seed);
    golf_total = golf_total + row_contract_golf_golf_route(seed + 7);
    println("row contract 5");
    let golf_adjust: i32 -> i32 = (value: i32) => value + 7;
    golf_total = golf_adjust(golf_total);
    golf_total = golf_total + row_contract_golf_golf_score(3);
    golf_total = golf_total + row_contract_golf_golf_finish(9);
    if golf_total > 46 {
        golf_total = golf_total - 8;
    } else {
        golf_total = golf_total + 10;
    }
    return golf_total;
}

flow row_contract_golf_golf_prepare(seed: i32) -> i32 ![]
{
    var tidal_prepare_total = seed + 9;
    var tidal_prepare_cursor = 0;
    while tidal_prepare_cursor < 9 limit Iterations(9) {
        tidal_prepare_total = tidal_prepare_total + tidal_prepare_cursor + 6;
        tidal_prepare_cursor = tidal_prepare_cursor + 1;
    }
    if tidal_prepare_total % 2 == 0 {
        tidal_prepare_total = tidal_prepare_total + row_contract_golf_golf_score(1);
    } else {
        tidal_prepare_total = tidal_prepare_total - 2;
    }
    var tidal_prepare_left = tidal_prepare_total + seed;
    var tidal_prepare_right = tidal_prepare_left * 4;
    var tidal_prepare_merged = tidal_prepare_right - tidal_prepare_left;
    if tidal_prepare_merged > 6 {
        tidal_prepare_total = tidal_prepare_total + tidal_prepare_merged;
    }
    return tidal_prepare_total;
}

flow row_contract_golf_golf_route(seed: i32) -> i32 ![]
{
    var tidal_route_total = seed * 9;
    var tidal_route_cursor = 0;
    while tidal_route_cursor < 7 limit Iterations(7) {
        tidal_route_total = tidal_route_total + tidal_route_cursor + 6;
        tidal_route_cursor = tidal_route_cursor + 1;
    }
    if tidal_route_total % 2 == 0 {
        tidal_route_total = tidal_route_total + 11;
    } else {
        tidal_route_total = tidal_route_total - 2;
    }
    var tidal_route_left = tidal_route_total + seed;
    var tidal_route_right = tidal_route_left * 4;
    var tidal_route_merged = tidal_route_right - tidal_route_left;
    if tidal_route_merged > 6 {
        tidal_route_total = tidal_route_total + tidal_route_merged;
    }
    return tidal_route_total;
}

flow row_contract_golf_golf_score(seed: i32) -> i32 ![]
{
    var tidal_score_total = seed + 9;
    var tidal_score_cursor = 0;
    while tidal_score_cursor < 12 limit Iterations(12) {
        tidal_score_total = tidal_score_total + tidal_score_cursor + 6;
        tidal_score_cursor = tidal_score_cursor + 1;
    }
    if tidal_score_total % 2 == 0 {
        tidal_score_total = tidal_score_total + 11;
    } else {
        tidal_score_total = tidal_score_total - 2;
    }
    var tidal_score_left = tidal_score_total + seed;
    var tidal_score_right = tidal_score_left * 4;
    var tidal_score_merged = tidal_score_right - tidal_score_left;
    if tidal_score_merged > 6 {
        tidal_score_total = tidal_score_total + tidal_score_merged;
    }
    return tidal_score_total;
}

flow row_contract_golf_golf_finish(seed: i32) -> i32 ![]
{
    var tidal_finish_total = seed - 9;
    var tidal_finish_cursor = 0;
    while tidal_finish_cursor < 11 limit Iterations(11) {
        tidal_finish_total = tidal_finish_total + tidal_finish_cursor + 6;
        tidal_finish_cursor = tidal_finish_cursor + 1;
    }
    if tidal_finish_total % 2 == 0 {
        tidal_finish_total = tidal_finish_total + 11;
    } else {
        tidal_finish_total = tidal_finish_total - 2;
    }
    var tidal_finish_left = tidal_finish_total + seed;
    var tidal_finish_right = tidal_finish_left * 4;
    var tidal_finish_merged = tidal_finish_right - tidal_finish_left;
    if tidal_finish_merged > 6 {
        tidal_finish_total = tidal_finish_total + tidal_finish_merged;
    }
    return tidal_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var golf_seed = 7;
    if args.len() > 0 {
        golf_seed = golf_seed + 1;
    } else {
        golf_seed = golf_seed + 2;
    }
    let golf_result = row_contract_golf_golf_entry(golf_seed);
    if golf_result > 0 {
        return 0;
    }
    return 1;
}
