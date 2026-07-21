module tests.compiler.effects.positive.stdio_console_032;

import std.io.{println};

flow stdio_console_glacier_glacier_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var glacier_total = stdio_console_glacier_glacier_prepare(seed);
    glacier_total = glacier_total + stdio_console_glacier_glacier_route(seed + 6);
    println("stdio console 1");
    let glacier_adjust: i32 -> i32 = (value: i32) => value + 7;
    glacier_total = glacier_adjust(glacier_total);
    glacier_total = glacier_total + stdio_console_glacier_glacier_score(4);
    glacier_total = glacier_total + stdio_console_glacier_glacier_finish(7);
    if glacier_total > 72 {
        glacier_total = glacier_total - 12;
    } else {
        glacier_total = glacier_total + 19;
    }
    return glacier_total;
}

flow stdio_console_glacier_glacier_prepare(seed: i32) -> i32 ![]
{
    var domain_prepare_total = seed + 16;
    var domain_prepare_cursor = 0;
    while domain_prepare_cursor < 10 limit Iterations(10) {
        domain_prepare_total = domain_prepare_total + domain_prepare_cursor + 4;
        domain_prepare_cursor = domain_prepare_cursor + 1;
    }
    if domain_prepare_total % 2 == 0 {
        domain_prepare_total = domain_prepare_total + stdio_console_glacier_glacier_score(1);
    } else {
        domain_prepare_total = domain_prepare_total - 3;
    }
    var domain_prepare_left = domain_prepare_total + seed;
    var domain_prepare_right = domain_prepare_left * 2;
    var domain_prepare_merged = domain_prepare_right - domain_prepare_left;
    if domain_prepare_merged > 1 {
        domain_prepare_total = domain_prepare_total + domain_prepare_merged;
    }
    return domain_prepare_total;
}

flow stdio_console_glacier_glacier_route(seed: i32) -> i32 ![]
{
    var domain_route_total = seed * 16;
    var domain_route_cursor = 0;
    while domain_route_cursor < 9 limit Iterations(9) {
        domain_route_total = domain_route_total + domain_route_cursor + 4;
        domain_route_cursor = domain_route_cursor + 1;
    }
    if domain_route_total % 2 == 0 {
        domain_route_total = domain_route_total + 14;
    } else {
        domain_route_total = domain_route_total - 3;
    }
    var domain_route_left = domain_route_total + seed;
    var domain_route_right = domain_route_left * 2;
    var domain_route_merged = domain_route_right - domain_route_left;
    if domain_route_merged > 1 {
        domain_route_total = domain_route_total + domain_route_merged;
    }
    return domain_route_total;
}

flow stdio_console_glacier_glacier_score(seed: i32) -> i32 ![]
{
    var domain_score_total = seed + 16;
    var domain_score_cursor = 0;
    while domain_score_cursor < 10 limit Iterations(10) {
        domain_score_total = domain_score_total + domain_score_cursor + 4;
        domain_score_cursor = domain_score_cursor + 1;
    }
    if domain_score_total % 2 == 0 {
        domain_score_total = domain_score_total + 14;
    } else {
        domain_score_total = domain_score_total - 3;
    }
    var domain_score_left = domain_score_total + seed;
    var domain_score_right = domain_score_left * 2;
    var domain_score_merged = domain_score_right - domain_score_left;
    if domain_score_merged > 1 {
        domain_score_total = domain_score_total + domain_score_merged;
    }
    return domain_score_total;
}

flow stdio_console_glacier_glacier_finish(seed: i32) -> i32 ![]
{
    var domain_finish_total = seed - 16;
    var domain_finish_cursor = 0;
    while domain_finish_cursor < 5 limit Iterations(5) {
        domain_finish_total = domain_finish_total + domain_finish_cursor + 4;
        domain_finish_cursor = domain_finish_cursor + 1;
    }
    if domain_finish_total % 2 == 0 {
        domain_finish_total = domain_finish_total + 14;
    } else {
        domain_finish_total = domain_finish_total - 3;
    }
    var domain_finish_left = domain_finish_total + seed;
    var domain_finish_right = domain_finish_left * 2;
    var domain_finish_merged = domain_finish_right - domain_finish_left;
    if domain_finish_merged > 1 {
        domain_finish_total = domain_finish_total + domain_finish_merged;
    }
    return domain_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var glacier_seed = 11;
    if args.len() > 0 {
        glacier_seed = glacier_seed + 1;
    } else {
        glacier_seed = glacier_seed + 2;
    }
    let glacier_result = stdio_console_glacier_glacier_entry(glacier_seed);
    if glacier_result > 0 {
        return 0;
    }
    return 1;
}
