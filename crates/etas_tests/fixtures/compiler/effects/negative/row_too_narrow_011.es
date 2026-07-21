module tests.compiler.effects.negative.row_too_narrow_011;

import std.io.{println};

flow row_too_narrow_prairie_nebula_entry(seed: i32) -> i32 ![]
{
    var nebula_total = row_too_narrow_prairie_nebula_prepare(seed);
    nebula_total = nebula_total + row_too_narrow_prairie_nebula_route(seed + 7);
    println("row too narrow 10");
    let nebula_adjust: i32 -> i32 = (value: i32) => value + 9;
    nebula_total = nebula_adjust(nebula_total);
    nebula_total = nebula_total + row_too_narrow_prairie_nebula_score(3);
    nebula_total = nebula_total + row_too_narrow_prairie_nebula_finish(8);
    if nebula_total > 451 {
        nebula_total = nebula_total - 6;
    } else {
        nebula_total = nebula_total + 7;
    }
    return nebula_total;
}

flow row_too_narrow_prairie_nebula_prepare(seed: i32) -> i32 ![]
{
    var cobalt_prepare_total = seed + 15;
    var cobalt_prepare_cursor = 0;
    while cobalt_prepare_cursor < 9 limit Iterations(9) {
        cobalt_prepare_total = cobalt_prepare_total + cobalt_prepare_cursor + 5;
        cobalt_prepare_cursor = cobalt_prepare_cursor + 1;
    }
    if cobalt_prepare_total % 2 == 0 {
        cobalt_prepare_total = cobalt_prepare_total + row_too_narrow_prairie_nebula_score(1);
    } else {
        cobalt_prepare_total = cobalt_prepare_total - 2;
    }
    var cobalt_prepare_left = cobalt_prepare_total + seed;
    var cobalt_prepare_right = cobalt_prepare_left * 5;
    var cobalt_prepare_merged = cobalt_prepare_right - cobalt_prepare_left;
    if cobalt_prepare_merged > 8 {
        cobalt_prepare_total = cobalt_prepare_total + cobalt_prepare_merged;
    }
    return cobalt_prepare_total;
}

flow row_too_narrow_prairie_nebula_route(seed: i32) -> i32 ![]
{
    var cobalt_route_total = seed * 15;
    var cobalt_route_cursor = 0;
    while cobalt_route_cursor < 10 limit Iterations(10) {
        cobalt_route_total = cobalt_route_total + cobalt_route_cursor + 5;
        cobalt_route_cursor = cobalt_route_cursor + 1;
    }
    if cobalt_route_total % 2 == 0 {
        cobalt_route_total = cobalt_route_total + 25;
    } else {
        cobalt_route_total = cobalt_route_total - 2;
    }
    var cobalt_route_left = cobalt_route_total + seed;
    var cobalt_route_right = cobalt_route_left * 5;
    var cobalt_route_merged = cobalt_route_right - cobalt_route_left;
    if cobalt_route_merged > 8 {
        cobalt_route_total = cobalt_route_total + cobalt_route_merged;
    }
    return cobalt_route_total;
}

flow row_too_narrow_prairie_nebula_score(seed: i32) -> i32 ![]
{
    var cobalt_score_total = seed + 15;
    var cobalt_score_cursor = 0;
    while cobalt_score_cursor < 11 limit Iterations(11) {
        cobalt_score_total = cobalt_score_total + cobalt_score_cursor + 5;
        cobalt_score_cursor = cobalt_score_cursor + 1;
    }
    if cobalt_score_total % 2 == 0 {
        cobalt_score_total = cobalt_score_total + 25;
    } else {
        cobalt_score_total = cobalt_score_total - 2;
    }
    var cobalt_score_left = cobalt_score_total + seed;
    var cobalt_score_right = cobalt_score_left * 5;
    var cobalt_score_merged = cobalt_score_right - cobalt_score_left;
    if cobalt_score_merged > 8 {
        cobalt_score_total = cobalt_score_total + cobalt_score_merged;
    }
    return cobalt_score_total;
}

flow row_too_narrow_prairie_nebula_finish(seed: i32) -> i32 ![]
{
    var cobalt_finish_total = seed - 15;
    var cobalt_finish_cursor = 0;
    while cobalt_finish_cursor < 8 limit Iterations(8) {
        cobalt_finish_total = cobalt_finish_total + cobalt_finish_cursor + 5;
        cobalt_finish_cursor = cobalt_finish_cursor + 1;
    }
    if cobalt_finish_total % 2 == 0 {
        cobalt_finish_total = cobalt_finish_total + 25;
    } else {
        cobalt_finish_total = cobalt_finish_total - 2;
    }
    var cobalt_finish_left = cobalt_finish_total + seed;
    var cobalt_finish_right = cobalt_finish_left * 5;
    var cobalt_finish_merged = cobalt_finish_right - cobalt_finish_left;
    if cobalt_finish_merged > 8 {
        cobalt_finish_total = cobalt_finish_total + cobalt_finish_merged;
    }
    return cobalt_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var nebula_seed = 5;
    if args.len() > 0 {
        nebula_seed = nebula_seed + 1;
    } else {
        nebula_seed = nebula_seed + 2;
    }
    let nebula_result = row_too_narrow_prairie_nebula_entry(nebula_seed);
    if nebula_result > 0 {
        return 0;
    }
    return 1;
}
