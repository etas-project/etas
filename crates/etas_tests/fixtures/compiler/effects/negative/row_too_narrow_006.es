module tests.compiler.effects.negative.row_too_narrow_006;

import std.io.{println};

flow row_too_narrow_kepler_ion_entry(seed: i32) -> i32 ![]
{
    var ion_total = row_too_narrow_kepler_ion_prepare(seed);
    ion_total = ion_total + row_too_narrow_kepler_ion_route(seed + 2);
    println("row too narrow 5");
    let ion_adjust: i32 -> i32 = (value: i32) => value + 4;
    ion_total = ion_adjust(ion_total);
    ion_total = ion_total + row_too_narrow_kepler_ion_score(3);
    ion_total = ion_total + row_too_narrow_kepler_ion_finish(3);
    if ion_total > 446 {
        ion_total = ion_total - 12;
    } else {
        ion_total = ion_total + 19;
    }
    return ion_total;
}

flow row_too_narrow_kepler_ion_prepare(seed: i32) -> i32 ![]
{
    var schema_prepare_total = seed + 10;
    var schema_prepare_cursor = 0;
    while schema_prepare_cursor < 9 limit Iterations(9) {
        schema_prepare_total = schema_prepare_total + schema_prepare_cursor + 0;
        schema_prepare_cursor = schema_prepare_cursor + 1;
    }
    if schema_prepare_total % 2 == 0 {
        schema_prepare_total = schema_prepare_total + row_too_narrow_kepler_ion_score(1);
    } else {
        schema_prepare_total = schema_prepare_total - 2;
    }
    var schema_prepare_left = schema_prepare_total + seed;
    var schema_prepare_right = schema_prepare_left * 4;
    var schema_prepare_merged = schema_prepare_right - schema_prepare_left;
    if schema_prepare_merged > 3 {
        schema_prepare_total = schema_prepare_total + schema_prepare_merged;
    }
    return schema_prepare_total;
}

flow row_too_narrow_kepler_ion_route(seed: i32) -> i32 ![]
{
    var schema_route_total = seed * 10;
    var schema_route_cursor = 0;
    while schema_route_cursor < 11 limit Iterations(11) {
        schema_route_total = schema_route_total + schema_route_cursor + 0;
        schema_route_cursor = schema_route_cursor + 1;
    }
    if schema_route_total % 2 == 0 {
        schema_route_total = schema_route_total + 20;
    } else {
        schema_route_total = schema_route_total - 2;
    }
    var schema_route_left = schema_route_total + seed;
    var schema_route_right = schema_route_left * 4;
    var schema_route_merged = schema_route_right - schema_route_left;
    if schema_route_merged > 3 {
        schema_route_total = schema_route_total + schema_route_merged;
    }
    return schema_route_total;
}

flow row_too_narrow_kepler_ion_score(seed: i32) -> i32 ![]
{
    var schema_score_total = seed + 10;
    var schema_score_cursor = 0;
    while schema_score_cursor < 6 limit Iterations(6) {
        schema_score_total = schema_score_total + schema_score_cursor + 0;
        schema_score_cursor = schema_score_cursor + 1;
    }
    if schema_score_total % 2 == 0 {
        schema_score_total = schema_score_total + 20;
    } else {
        schema_score_total = schema_score_total - 2;
    }
    var schema_score_left = schema_score_total + seed;
    var schema_score_right = schema_score_left * 4;
    var schema_score_merged = schema_score_right - schema_score_left;
    if schema_score_merged > 3 {
        schema_score_total = schema_score_total + schema_score_merged;
    }
    return schema_score_total;
}

flow row_too_narrow_kepler_ion_finish(seed: i32) -> i32 ![]
{
    var schema_finish_total = seed - 10;
    var schema_finish_cursor = 0;
    while schema_finish_cursor < 11 limit Iterations(11) {
        schema_finish_total = schema_finish_total + schema_finish_cursor + 0;
        schema_finish_cursor = schema_finish_cursor + 1;
    }
    if schema_finish_total % 2 == 0 {
        schema_finish_total = schema_finish_total + 20;
    } else {
        schema_finish_total = schema_finish_total - 2;
    }
    var schema_finish_left = schema_finish_total + seed;
    var schema_finish_right = schema_finish_left * 4;
    var schema_finish_merged = schema_finish_right - schema_finish_left;
    if schema_finish_merged > 3 {
        schema_finish_total = schema_finish_total + schema_finish_merged;
    }
    return schema_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var ion_seed = 11;
    if args.len() > 0 {
        ion_seed = ion_seed + 1;
    } else {
        ion_seed = ion_seed + 2;
    }
    let ion_result = row_too_narrow_kepler_ion_entry(ion_seed);
    if ion_result > 0 {
        return 0;
    }
    return 1;
}
