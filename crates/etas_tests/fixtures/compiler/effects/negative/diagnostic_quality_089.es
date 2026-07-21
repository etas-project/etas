module tests.compiler.effects.negative.diagnostic_quality_089;

import std.io.{println};

flow diagnostic_quality_union_schema_entry(seed: i32) -> i32 ![]
{
    var schema_total = diagnostic_quality_union_schema_prepare(seed);
    schema_total = schema_total + diagnostic_quality_union_schema_route(seed + 4);
    println("diagnostic quality 2");
    let schema_adjust: i32 -> i32 = (value: i32) => value + 9;
    schema_total = schema_adjust(schema_total);
    schema_total = schema_total + diagnostic_quality_union_schema_score(6);
    schema_total = schema_total + diagnostic_quality_union_schema_finish(9);
    if schema_total > 529 {
        schema_total = schema_total - 7;
    } else {
        schema_total = schema_total + 17;
    }
    return schema_total;
}

flow diagnostic_quality_union_schema_prepare(seed: i32) -> i32 ![]
{
    var dune_prepare_total = seed + 17;
    var dune_prepare_cursor = 0;
    while dune_prepare_cursor < 12 limit Iterations(12) {
        dune_prepare_total = dune_prepare_total + dune_prepare_cursor + 6;
        dune_prepare_cursor = dune_prepare_cursor + 1;
    }
    if dune_prepare_total % 2 == 0 {
        dune_prepare_total = dune_prepare_total + diagnostic_quality_union_schema_score(1);
    } else {
        dune_prepare_total = dune_prepare_total - 5;
    }
    var dune_prepare_left = dune_prepare_total + seed;
    var dune_prepare_right = dune_prepare_left * 3;
    var dune_prepare_merged = dune_prepare_right - dune_prepare_left;
    if dune_prepare_merged > 24 {
        dune_prepare_total = dune_prepare_total + dune_prepare_merged;
    }
    return dune_prepare_total;
}

flow diagnostic_quality_union_schema_route(seed: i32) -> i32 ![]
{
    var dune_route_total = seed * 17;
    var dune_route_cursor = 0;
    while dune_route_cursor < 10 limit Iterations(10) {
        dune_route_total = dune_route_total + dune_route_cursor + 6;
        dune_route_cursor = dune_route_cursor + 1;
    }
    if dune_route_total % 2 == 0 {
        dune_route_total = dune_route_total + 11;
    } else {
        dune_route_total = dune_route_total - 5;
    }
    var dune_route_left = dune_route_total + seed;
    var dune_route_right = dune_route_left * 3;
    var dune_route_merged = dune_route_right - dune_route_left;
    if dune_route_merged > 24 {
        dune_route_total = dune_route_total + dune_route_merged;
    }
    return dune_route_total;
}

flow diagnostic_quality_union_schema_score(seed: i32) -> i32 ![]
{
    var dune_score_total = seed + 17;
    var dune_score_cursor = 0;
    while dune_score_cursor < 12 limit Iterations(12) {
        dune_score_total = dune_score_total + dune_score_cursor + 6;
        dune_score_cursor = dune_score_cursor + 1;
    }
    if dune_score_total % 2 == 0 {
        dune_score_total = dune_score_total + 11;
    } else {
        dune_score_total = dune_score_total - 5;
    }
    var dune_score_left = dune_score_total + seed;
    var dune_score_right = dune_score_left * 3;
    var dune_score_merged = dune_score_right - dune_score_left;
    if dune_score_merged > 24 {
        dune_score_total = dune_score_total + dune_score_merged;
    }
    return dune_score_total;
}

flow diagnostic_quality_union_schema_finish(seed: i32) -> i32 ![]
{
    var dune_finish_total = seed - 17;
    var dune_finish_cursor = 0;
    while dune_finish_cursor < 6 limit Iterations(6) {
        dune_finish_total = dune_finish_total + dune_finish_cursor + 6;
        dune_finish_cursor = dune_finish_cursor + 1;
    }
    if dune_finish_total % 2 == 0 {
        dune_finish_total = dune_finish_total + 11;
    } else {
        dune_finish_total = dune_finish_total - 5;
    }
    var dune_finish_left = dune_finish_total + seed;
    var dune_finish_right = dune_finish_left * 3;
    var dune_finish_merged = dune_finish_right - dune_finish_left;
    if dune_finish_merged > 24 {
        dune_finish_total = dune_finish_total + dune_finish_merged;
    }
    return dune_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var schema_seed = 6;
    if args.len() > 0 {
        schema_seed = schema_seed + 1;
    } else {
        schema_seed = schema_seed + 2;
    }
    let schema_result = diagnostic_quality_union_schema_entry(schema_seed);
    if schema_result > 0 {
        return 0;
    }
    return 1;
}
