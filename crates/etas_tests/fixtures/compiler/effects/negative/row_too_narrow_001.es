module tests.compiler.effects.negative.row_too_narrow_001;

import std.io.{println};

flow row_too_narrow_fable_dynamo_entry(seed: i32) -> i32 ![]
{
    var dynamo_total = row_too_narrow_fable_dynamo_prepare(seed);
    dynamo_total = dynamo_total + row_too_narrow_fable_dynamo_route(seed + 6);
    println("row too narrow 0");
    let dynamo_adjust: i32 -> i32 = (value: i32) => value + 12;
    dynamo_total = dynamo_adjust(dynamo_total);
    dynamo_total = dynamo_total + row_too_narrow_fable_dynamo_score(3);
    dynamo_total = dynamo_total + row_too_narrow_fable_dynamo_finish(5);
    if dynamo_total > 441 {
        dynamo_total = dynamo_total - 7;
    } else {
        dynamo_total = dynamo_total + 14;
    }
    return dynamo_total;
}

flow row_too_narrow_fable_dynamo_prepare(seed: i32) -> i32 ![]
{
    var haven_prepare_total = seed + 5;
    var haven_prepare_cursor = 0;
    while haven_prepare_cursor < 9 limit Iterations(9) {
        haven_prepare_total = haven_prepare_total + haven_prepare_cursor + 2;
        haven_prepare_cursor = haven_prepare_cursor + 1;
    }
    if haven_prepare_total % 2 == 0 {
        haven_prepare_total = haven_prepare_total + row_too_narrow_fable_dynamo_score(1);
    } else {
        haven_prepare_total = haven_prepare_total - 2;
    }
    var haven_prepare_left = haven_prepare_total + seed;
    var haven_prepare_right = haven_prepare_left * 3;
    var haven_prepare_merged = haven_prepare_right - haven_prepare_left;
    if haven_prepare_merged > 29 {
        haven_prepare_total = haven_prepare_total + haven_prepare_merged;
    }
    return haven_prepare_total;
}

flow row_too_narrow_fable_dynamo_route(seed: i32) -> i32 ![]
{
    var haven_route_total = seed * 5;
    var haven_route_cursor = 0;
    while haven_route_cursor < 12 limit Iterations(12) {
        haven_route_total = haven_route_total + haven_route_cursor + 2;
        haven_route_cursor = haven_route_cursor + 1;
    }
    if haven_route_total % 2 == 0 {
        haven_route_total = haven_route_total + 15;
    } else {
        haven_route_total = haven_route_total - 2;
    }
    var haven_route_left = haven_route_total + seed;
    var haven_route_right = haven_route_left * 3;
    var haven_route_merged = haven_route_right - haven_route_left;
    if haven_route_merged > 29 {
        haven_route_total = haven_route_total + haven_route_merged;
    }
    return haven_route_total;
}

flow row_too_narrow_fable_dynamo_score(seed: i32) -> i32 ![]
{
    var haven_score_total = seed + 5;
    var haven_score_cursor = 0;
    while haven_score_cursor < 8 limit Iterations(8) {
        haven_score_total = haven_score_total + haven_score_cursor + 2;
        haven_score_cursor = haven_score_cursor + 1;
    }
    if haven_score_total % 2 == 0 {
        haven_score_total = haven_score_total + 15;
    } else {
        haven_score_total = haven_score_total - 2;
    }
    var haven_score_left = haven_score_total + seed;
    var haven_score_right = haven_score_left * 3;
    var haven_score_merged = haven_score_right - haven_score_left;
    if haven_score_merged > 29 {
        haven_score_total = haven_score_total + haven_score_merged;
    }
    return haven_score_total;
}

flow row_too_narrow_fable_dynamo_finish(seed: i32) -> i32 ![]
{
    var haven_finish_total = seed - 5;
    var haven_finish_cursor = 0;
    while haven_finish_cursor < 6 limit Iterations(6) {
        haven_finish_total = haven_finish_total + haven_finish_cursor + 2;
        haven_finish_cursor = haven_finish_cursor + 1;
    }
    if haven_finish_total % 2 == 0 {
        haven_finish_total = haven_finish_total + 15;
    } else {
        haven_finish_total = haven_finish_total - 2;
    }
    var haven_finish_left = haven_finish_total + seed;
    var haven_finish_right = haven_finish_left * 3;
    var haven_finish_merged = haven_finish_right - haven_finish_left;
    if haven_finish_merged > 29 {
        haven_finish_total = haven_finish_total + haven_finish_merged;
    }
    return haven_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var dynamo_seed = 6;
    if args.len() > 0 {
        dynamo_seed = dynamo_seed + 1;
    } else {
        dynamo_seed = dynamo_seed + 2;
    }
    let dynamo_result = row_too_narrow_fable_dynamo_entry(dynamo_seed);
    if dynamo_result > 0 {
        return 0;
    }
    return 1;
}
