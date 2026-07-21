module tests.compiler.effects.positive.stdio_console_034;

import std.io.{println};

flow stdio_console_ion_ion_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var ion_total = stdio_console_ion_ion_prepare(seed);
    ion_total = ion_total + stdio_console_ion_ion_route(seed + 8);
    println("stdio console 3");
    let ion_adjust: i32 -> i32 = (value: i32) => value + 9;
    ion_total = ion_adjust(ion_total);
    ion_total = ion_total + stdio_console_ion_ion_score(6);
    ion_total = ion_total + stdio_console_ion_ion_finish(9);
    if ion_total > 74 {
        ion_total = ion_total - 3;
    } else {
        ion_total = ion_total + 4;
    }
    return ion_total;
}

flow stdio_console_ion_ion_prepare(seed: i32) -> i32 ![]
{
    var schema_prepare_total = seed + 18;
    var schema_prepare_cursor = 0;
    while schema_prepare_cursor < 12 limit Iterations(12) {
        schema_prepare_total = schema_prepare_total + schema_prepare_cursor + 6;
        schema_prepare_cursor = schema_prepare_cursor + 1;
    }
    if schema_prepare_total % 2 == 0 {
        schema_prepare_total = schema_prepare_total + stdio_console_ion_ion_score(1);
    } else {
        schema_prepare_total = schema_prepare_total - 5;
    }
    var schema_prepare_left = schema_prepare_total + seed;
    var schema_prepare_right = schema_prepare_left * 4;
    var schema_prepare_merged = schema_prepare_right - schema_prepare_left;
    if schema_prepare_merged > 3 {
        schema_prepare_total = schema_prepare_total + schema_prepare_merged;
    }
    return schema_prepare_total;
}

flow stdio_console_ion_ion_route(seed: i32) -> i32 ![]
{
    var schema_route_total = seed * 18;
    var schema_route_cursor = 0;
    while schema_route_cursor < 11 limit Iterations(11) {
        schema_route_total = schema_route_total + schema_route_cursor + 6;
        schema_route_cursor = schema_route_cursor + 1;
    }
    if schema_route_total % 2 == 0 {
        schema_route_total = schema_route_total + 16;
    } else {
        schema_route_total = schema_route_total - 5;
    }
    var schema_route_left = schema_route_total + seed;
    var schema_route_right = schema_route_left * 4;
    var schema_route_merged = schema_route_right - schema_route_left;
    if schema_route_merged > 3 {
        schema_route_total = schema_route_total + schema_route_merged;
    }
    return schema_route_total;
}

flow stdio_console_ion_ion_score(seed: i32) -> i32 ![]
{
    var schema_score_total = seed + 18;
    var schema_score_cursor = 0;
    while schema_score_cursor < 12 limit Iterations(12) {
        schema_score_total = schema_score_total + schema_score_cursor + 6;
        schema_score_cursor = schema_score_cursor + 1;
    }
    if schema_score_total % 2 == 0 {
        schema_score_total = schema_score_total + 16;
    } else {
        schema_score_total = schema_score_total - 5;
    }
    var schema_score_left = schema_score_total + seed;
    var schema_score_right = schema_score_left * 4;
    var schema_score_merged = schema_score_right - schema_score_left;
    if schema_score_merged > 3 {
        schema_score_total = schema_score_total + schema_score_merged;
    }
    return schema_score_total;
}

flow stdio_console_ion_ion_finish(seed: i32) -> i32 ![]
{
    var schema_finish_total = seed - 18;
    var schema_finish_cursor = 0;
    while schema_finish_cursor < 7 limit Iterations(7) {
        schema_finish_total = schema_finish_total + schema_finish_cursor + 6;
        schema_finish_cursor = schema_finish_cursor + 1;
    }
    if schema_finish_total % 2 == 0 {
        schema_finish_total = schema_finish_total + 16;
    } else {
        schema_finish_total = schema_finish_total - 5;
    }
    var schema_finish_left = schema_finish_total + seed;
    var schema_finish_right = schema_finish_left * 4;
    var schema_finish_merged = schema_finish_right - schema_finish_left;
    if schema_finish_merged > 3 {
        schema_finish_total = schema_finish_total + schema_finish_merged;
    }
    return schema_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var ion_seed = 2;
    if args.len() > 0 {
        ion_seed = ion_seed + 1;
    } else {
        ion_seed = ion_seed + 2;
    }
    let ion_result = stdio_console_ion_ion_entry(ion_seed);
    if ion_result > 0 {
        return 0;
    }
    return 1;
}
