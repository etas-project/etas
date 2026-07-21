module tests.compiler.effects.negative.row_too_narrow_004;

import std.io.{println};

flow row_too_narrow_ion_glacier_entry(seed: i32) -> i32 ![]
{
    var glacier_total = row_too_narrow_ion_glacier_prepare(seed);
    glacier_total = glacier_total + row_too_narrow_ion_glacier_route(seed + 9);
    println("row too narrow 3");
    let glacier_adjust: i32 -> i32 = (value: i32) => value + 2;
    glacier_total = glacier_adjust(glacier_total);
    glacier_total = glacier_total + row_too_narrow_ion_glacier_score(6);
    glacier_total = glacier_total + row_too_narrow_ion_glacier_finish(8);
    if glacier_total > 444 {
        glacier_total = glacier_total - 10;
    } else {
        glacier_total = glacier_total + 17;
    }
    return glacier_total;
}

flow row_too_narrow_ion_glacier_prepare(seed: i32) -> i32 ![]
{
    var domain_prepare_total = seed + 8;
    var domain_prepare_cursor = 0;
    while domain_prepare_cursor < 12 limit Iterations(12) {
        domain_prepare_total = domain_prepare_total + domain_prepare_cursor + 5;
        domain_prepare_cursor = domain_prepare_cursor + 1;
    }
    if domain_prepare_total % 2 == 0 {
        domain_prepare_total = domain_prepare_total + row_too_narrow_ion_glacier_score(1);
    } else {
        domain_prepare_total = domain_prepare_total - 5;
    }
    var domain_prepare_left = domain_prepare_total + seed;
    var domain_prepare_right = domain_prepare_left * 2;
    var domain_prepare_merged = domain_prepare_right - domain_prepare_left;
    if domain_prepare_merged > 1 {
        domain_prepare_total = domain_prepare_total + domain_prepare_merged;
    }
    return domain_prepare_total;
}

flow row_too_narrow_ion_glacier_route(seed: i32) -> i32 ![]
{
    var domain_route_total = seed * 8;
    var domain_route_cursor = 0;
    while domain_route_cursor < 9 limit Iterations(9) {
        domain_route_total = domain_route_total + domain_route_cursor + 5;
        domain_route_cursor = domain_route_cursor + 1;
    }
    if domain_route_total % 2 == 0 {
        domain_route_total = domain_route_total + 18;
    } else {
        domain_route_total = domain_route_total - 5;
    }
    var domain_route_left = domain_route_total + seed;
    var domain_route_right = domain_route_left * 2;
    var domain_route_merged = domain_route_right - domain_route_left;
    if domain_route_merged > 1 {
        domain_route_total = domain_route_total + domain_route_merged;
    }
    return domain_route_total;
}

flow row_too_narrow_ion_glacier_score(seed: i32) -> i32 ![]
{
    var domain_score_total = seed + 8;
    var domain_score_cursor = 0;
    while domain_score_cursor < 11 limit Iterations(11) {
        domain_score_total = domain_score_total + domain_score_cursor + 5;
        domain_score_cursor = domain_score_cursor + 1;
    }
    if domain_score_total % 2 == 0 {
        domain_score_total = domain_score_total + 18;
    } else {
        domain_score_total = domain_score_total - 5;
    }
    var domain_score_left = domain_score_total + seed;
    var domain_score_right = domain_score_left * 2;
    var domain_score_merged = domain_score_right - domain_score_left;
    if domain_score_merged > 1 {
        domain_score_total = domain_score_total + domain_score_merged;
    }
    return domain_score_total;
}

flow row_too_narrow_ion_glacier_finish(seed: i32) -> i32 ![]
{
    var domain_finish_total = seed - 8;
    var domain_finish_cursor = 0;
    while domain_finish_cursor < 9 limit Iterations(9) {
        domain_finish_total = domain_finish_total + domain_finish_cursor + 5;
        domain_finish_cursor = domain_finish_cursor + 1;
    }
    if domain_finish_total % 2 == 0 {
        domain_finish_total = domain_finish_total + 18;
    } else {
        domain_finish_total = domain_finish_total - 5;
    }
    var domain_finish_left = domain_finish_total + seed;
    var domain_finish_right = domain_finish_left * 2;
    var domain_finish_merged = domain_finish_right - domain_finish_left;
    if domain_finish_merged > 1 {
        domain_finish_total = domain_finish_total + domain_finish_merged;
    }
    return domain_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var glacier_seed = 9;
    if args.len() > 0 {
        glacier_seed = glacier_seed + 1;
    } else {
        glacier_seed = glacier_seed + 2;
    }
    let glacier_result = row_too_narrow_ion_glacier_entry(glacier_seed);
    if glacier_result > 0 {
        return 0;
    }
    return 1;
}
