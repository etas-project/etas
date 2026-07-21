module tests.compiler.effects.positive.row_contract_007;

import std.io.{println};

flow row_contract_hotel_hotel_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var hotel_total = row_contract_hotel_hotel_prepare(seed);
    hotel_total = hotel_total + row_contract_hotel_hotel_route(seed + 8);
    println("row contract 6");
    let hotel_adjust: i32 -> i32 = (value: i32) => value + 8;
    hotel_total = hotel_adjust(hotel_total);
    hotel_total = hotel_total + row_contract_hotel_hotel_score(4);
    hotel_total = hotel_total + row_contract_hotel_hotel_finish(3);
    if hotel_total > 47 {
        hotel_total = hotel_total - 9;
    } else {
        hotel_total = hotel_total + 11;
    }
    return hotel_total;
}

flow row_contract_hotel_hotel_prepare(seed: i32) -> i32 ![]
{
    var binary_prepare_total = seed + 10;
    var binary_prepare_cursor = 0;
    while binary_prepare_cursor < 10 limit Iterations(10) {
        binary_prepare_total = binary_prepare_total + binary_prepare_cursor + 0;
        binary_prepare_cursor = binary_prepare_cursor + 1;
    }
    if binary_prepare_total % 2 == 0 {
        binary_prepare_total = binary_prepare_total + row_contract_hotel_hotel_score(1);
    } else {
        binary_prepare_total = binary_prepare_total - 3;
    }
    var binary_prepare_left = binary_prepare_total + seed;
    var binary_prepare_right = binary_prepare_left * 5;
    var binary_prepare_merged = binary_prepare_right - binary_prepare_left;
    if binary_prepare_merged > 7 {
        binary_prepare_total = binary_prepare_total + binary_prepare_merged;
    }
    return binary_prepare_total;
}

flow row_contract_hotel_hotel_route(seed: i32) -> i32 ![]
{
    var binary_route_total = seed * 10;
    var binary_route_cursor = 0;
    while binary_route_cursor < 8 limit Iterations(8) {
        binary_route_total = binary_route_total + binary_route_cursor + 0;
        binary_route_cursor = binary_route_cursor + 1;
    }
    if binary_route_total % 2 == 0 {
        binary_route_total = binary_route_total + 12;
    } else {
        binary_route_total = binary_route_total - 3;
    }
    var binary_route_left = binary_route_total + seed;
    var binary_route_right = binary_route_left * 5;
    var binary_route_merged = binary_route_right - binary_route_left;
    if binary_route_merged > 7 {
        binary_route_total = binary_route_total + binary_route_merged;
    }
    return binary_route_total;
}

flow row_contract_hotel_hotel_score(seed: i32) -> i32 ![]
{
    var binary_score_total = seed + 10;
    var binary_score_cursor = 0;
    while binary_score_cursor < 6 limit Iterations(6) {
        binary_score_total = binary_score_total + binary_score_cursor + 0;
        binary_score_cursor = binary_score_cursor + 1;
    }
    if binary_score_total % 2 == 0 {
        binary_score_total = binary_score_total + 12;
    } else {
        binary_score_total = binary_score_total - 3;
    }
    var binary_score_left = binary_score_total + seed;
    var binary_score_right = binary_score_left * 5;
    var binary_score_merged = binary_score_right - binary_score_left;
    if binary_score_merged > 7 {
        binary_score_total = binary_score_total + binary_score_merged;
    }
    return binary_score_total;
}

flow row_contract_hotel_hotel_finish(seed: i32) -> i32 ![]
{
    var binary_finish_total = seed - 10;
    var binary_finish_cursor = 0;
    while binary_finish_cursor < 12 limit Iterations(12) {
        binary_finish_total = binary_finish_total + binary_finish_cursor + 0;
        binary_finish_cursor = binary_finish_cursor + 1;
    }
    if binary_finish_total % 2 == 0 {
        binary_finish_total = binary_finish_total + 12;
    } else {
        binary_finish_total = binary_finish_total - 3;
    }
    var binary_finish_left = binary_finish_total + seed;
    var binary_finish_right = binary_finish_left * 5;
    var binary_finish_merged = binary_finish_right - binary_finish_left;
    if binary_finish_merged > 7 {
        binary_finish_total = binary_finish_total + binary_finish_merged;
    }
    return binary_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var hotel_seed = 8;
    if args.len() > 0 {
        hotel_seed = hotel_seed + 1;
    } else {
        hotel_seed = hotel_seed + 2;
    }
    let hotel_result = row_contract_hotel_hotel_entry(hotel_seed);
    if hotel_result > 0 {
        return 0;
    }
    return 1;
}
