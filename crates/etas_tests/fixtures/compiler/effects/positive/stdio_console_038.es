module tests.compiler.effects.positive.stdio_console_038;

import std.io.{println};

flow stdio_console_meteor_meteor_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var meteor_total = stdio_console_meteor_meteor_prepare(seed);
    meteor_total = meteor_total + stdio_console_meteor_meteor_route(seed + 3);
    println("stdio console 7");
    let meteor_adjust: i32 -> i32 = (value: i32) => value + 13;
    meteor_total = meteor_adjust(meteor_total);
    meteor_total = meteor_total + stdio_console_meteor_meteor_score(5);
    meteor_total = meteor_total + stdio_console_meteor_meteor_finish(6);
    if meteor_total > 78 {
        meteor_total = meteor_total - 7;
    } else {
        meteor_total = meteor_total + 8;
    }
    return meteor_total;
}

flow stdio_console_meteor_meteor_prepare(seed: i32) -> i32 ![]
{
    var violet_prepare_total = seed + 3;
    var violet_prepare_cursor = 0;
    while violet_prepare_cursor < 11 limit Iterations(11) {
        violet_prepare_total = violet_prepare_total + violet_prepare_cursor + 3;
        violet_prepare_cursor = violet_prepare_cursor + 1;
    }
    if violet_prepare_total % 2 == 0 {
        violet_prepare_total = violet_prepare_total + stdio_console_meteor_meteor_score(1);
    } else {
        violet_prepare_total = violet_prepare_total - 4;
    }
    var violet_prepare_left = violet_prepare_total + seed;
    var violet_prepare_right = violet_prepare_left * 4;
    var violet_prepare_merged = violet_prepare_right - violet_prepare_left;
    if violet_prepare_merged > 7 {
        violet_prepare_total = violet_prepare_total + violet_prepare_merged;
    }
    return violet_prepare_total;
}

flow stdio_console_meteor_meteor_route(seed: i32) -> i32 ![]
{
    var violet_route_total = seed * 3;
    var violet_route_cursor = 0;
    while violet_route_cursor < 9 limit Iterations(9) {
        violet_route_total = violet_route_total + violet_route_cursor + 3;
        violet_route_cursor = violet_route_cursor + 1;
    }
    if violet_route_total % 2 == 0 {
        violet_route_total = violet_route_total + 20;
    } else {
        violet_route_total = violet_route_total - 4;
    }
    var violet_route_left = violet_route_total + seed;
    var violet_route_right = violet_route_left * 4;
    var violet_route_merged = violet_route_right - violet_route_left;
    if violet_route_merged > 7 {
        violet_route_total = violet_route_total + violet_route_merged;
    }
    return violet_route_total;
}

flow stdio_console_meteor_meteor_score(seed: i32) -> i32 ![]
{
    var violet_score_total = seed + 3;
    var violet_score_cursor = 0;
    while violet_score_cursor < 9 limit Iterations(9) {
        violet_score_total = violet_score_total + violet_score_cursor + 3;
        violet_score_cursor = violet_score_cursor + 1;
    }
    if violet_score_total % 2 == 0 {
        violet_score_total = violet_score_total + 20;
    } else {
        violet_score_total = violet_score_total - 4;
    }
    var violet_score_left = violet_score_total + seed;
    var violet_score_right = violet_score_left * 4;
    var violet_score_merged = violet_score_right - violet_score_left;
    if violet_score_merged > 7 {
        violet_score_total = violet_score_total + violet_score_merged;
    }
    return violet_score_total;
}

flow stdio_console_meteor_meteor_finish(seed: i32) -> i32 ![]
{
    var violet_finish_total = seed - 3;
    var violet_finish_cursor = 0;
    while violet_finish_cursor < 11 limit Iterations(11) {
        violet_finish_total = violet_finish_total + violet_finish_cursor + 3;
        violet_finish_cursor = violet_finish_cursor + 1;
    }
    if violet_finish_total % 2 == 0 {
        violet_finish_total = violet_finish_total + 20;
    } else {
        violet_finish_total = violet_finish_total - 4;
    }
    var violet_finish_left = violet_finish_total + seed;
    var violet_finish_right = violet_finish_left * 4;
    var violet_finish_merged = violet_finish_right - violet_finish_left;
    if violet_finish_merged > 7 {
        violet_finish_total = violet_finish_total + violet_finish_merged;
    }
    return violet_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var meteor_seed = 6;
    if args.len() > 0 {
        meteor_seed = meteor_seed + 1;
    } else {
        meteor_seed = meteor_seed + 2;
    }
    let meteor_result = stdio_console_meteor_meteor_entry(meteor_seed);
    if meteor_result > 0 {
        return 0;
    }
    return 1;
}
