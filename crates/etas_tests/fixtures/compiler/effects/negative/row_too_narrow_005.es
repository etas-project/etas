module tests.compiler.effects.negative.row_too_narrow_005;

import std.io.{println};

flow row_too_narrow_juno_harbor_entry(seed: i32) -> i32 ![]
{
    var harbor_total = row_too_narrow_juno_harbor_prepare(seed);
    harbor_total = harbor_total + row_too_narrow_juno_harbor_route(seed + 1);
    println("row too narrow 4");
    let harbor_adjust: i32 -> i32 = (value: i32) => value + 3;
    harbor_total = harbor_adjust(harbor_total);
    harbor_total = harbor_total + row_too_narrow_juno_harbor_score(2);
    harbor_total = harbor_total + row_too_narrow_juno_harbor_finish(9);
    if harbor_total > 445 {
        harbor_total = harbor_total - 11;
    } else {
        harbor_total = harbor_total + 18;
    }
    return harbor_total;
}

flow row_too_narrow_juno_harbor_prepare(seed: i32) -> i32 ![]
{
    var keeper_prepare_total = seed + 9;
    var keeper_prepare_cursor = 0;
    while keeper_prepare_cursor < 8 limit Iterations(8) {
        keeper_prepare_total = keeper_prepare_total + keeper_prepare_cursor + 6;
        keeper_prepare_cursor = keeper_prepare_cursor + 1;
    }
    if keeper_prepare_total % 2 == 0 {
        keeper_prepare_total = keeper_prepare_total + row_too_narrow_juno_harbor_score(1);
    } else {
        keeper_prepare_total = keeper_prepare_total - 1;
    }
    var keeper_prepare_left = keeper_prepare_total + seed;
    var keeper_prepare_right = keeper_prepare_left * 3;
    var keeper_prepare_merged = keeper_prepare_right - keeper_prepare_left;
    if keeper_prepare_merged > 2 {
        keeper_prepare_total = keeper_prepare_total + keeper_prepare_merged;
    }
    return keeper_prepare_total;
}

flow row_too_narrow_juno_harbor_route(seed: i32) -> i32 ![]
{
    var keeper_route_total = seed * 9;
    var keeper_route_cursor = 0;
    while keeper_route_cursor < 10 limit Iterations(10) {
        keeper_route_total = keeper_route_total + keeper_route_cursor + 6;
        keeper_route_cursor = keeper_route_cursor + 1;
    }
    if keeper_route_total % 2 == 0 {
        keeper_route_total = keeper_route_total + 19;
    } else {
        keeper_route_total = keeper_route_total - 1;
    }
    var keeper_route_left = keeper_route_total + seed;
    var keeper_route_right = keeper_route_left * 3;
    var keeper_route_merged = keeper_route_right - keeper_route_left;
    if keeper_route_merged > 2 {
        keeper_route_total = keeper_route_total + keeper_route_merged;
    }
    return keeper_route_total;
}

flow row_too_narrow_juno_harbor_score(seed: i32) -> i32 ![]
{
    var keeper_score_total = seed + 9;
    var keeper_score_cursor = 0;
    while keeper_score_cursor < 12 limit Iterations(12) {
        keeper_score_total = keeper_score_total + keeper_score_cursor + 6;
        keeper_score_cursor = keeper_score_cursor + 1;
    }
    if keeper_score_total % 2 == 0 {
        keeper_score_total = keeper_score_total + 19;
    } else {
        keeper_score_total = keeper_score_total - 1;
    }
    var keeper_score_left = keeper_score_total + seed;
    var keeper_score_right = keeper_score_left * 3;
    var keeper_score_merged = keeper_score_right - keeper_score_left;
    if keeper_score_merged > 2 {
        keeper_score_total = keeper_score_total + keeper_score_merged;
    }
    return keeper_score_total;
}

flow row_too_narrow_juno_harbor_finish(seed: i32) -> i32 ![]
{
    var keeper_finish_total = seed - 9;
    var keeper_finish_cursor = 0;
    while keeper_finish_cursor < 10 limit Iterations(10) {
        keeper_finish_total = keeper_finish_total + keeper_finish_cursor + 6;
        keeper_finish_cursor = keeper_finish_cursor + 1;
    }
    if keeper_finish_total % 2 == 0 {
        keeper_finish_total = keeper_finish_total + 19;
    } else {
        keeper_finish_total = keeper_finish_total - 1;
    }
    var keeper_finish_left = keeper_finish_total + seed;
    var keeper_finish_right = keeper_finish_left * 3;
    var keeper_finish_merged = keeper_finish_right - keeper_finish_left;
    if keeper_finish_merged > 2 {
        keeper_finish_total = keeper_finish_total + keeper_finish_merged;
    }
    return keeper_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var harbor_seed = 10;
    if args.len() > 0 {
        harbor_seed = harbor_seed + 1;
    } else {
        harbor_seed = harbor_seed + 2;
    }
    let harbor_result = row_too_narrow_juno_harbor_entry(harbor_seed);
    if harbor_result > 0 {
        return 0;
    }
    return 1;
}
