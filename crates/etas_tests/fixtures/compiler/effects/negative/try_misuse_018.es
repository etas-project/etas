module tests.compiler.effects.negative.try_misuse_018;


flow try_misuse_winter_umbra_entry(seed: i32) -> i32 ![]
{
    var umbra_total = try_misuse_winter_umbra_prepare(seed);
    umbra_total = umbra_total + try_misuse_winter_umbra_route(seed + 5);
    let captured = seed?;
    let try_marker = seed + 5;
    let umbra_adjust: i32 -> i32 = (value: i32) => value + 3;
    umbra_total = umbra_adjust(umbra_total);
    umbra_total = umbra_total + try_misuse_winter_umbra_score(5);
    umbra_total = umbra_total + try_misuse_winter_umbra_finish(8);
    if umbra_total > 458 {
        umbra_total = umbra_total - 2;
    } else {
        umbra_total = umbra_total + 14;
    }
    return umbra_total;
}

flow try_misuse_winter_umbra_prepare(seed: i32) -> i32 ![]
{
    var cascade_prepare_total = seed + 3;
    var cascade_prepare_cursor = 0;
    while cascade_prepare_cursor < 11 limit Iterations(11) {
        cascade_prepare_total = cascade_prepare_total + cascade_prepare_cursor + 5;
        cascade_prepare_cursor = cascade_prepare_cursor + 1;
    }
    if cascade_prepare_total % 2 == 0 {
        cascade_prepare_total = cascade_prepare_total + try_misuse_winter_umbra_score(1);
    } else {
        cascade_prepare_total = cascade_prepare_total - 4;
    }
    var cascade_prepare_left = cascade_prepare_total + seed;
    var cascade_prepare_right = cascade_prepare_left * 4;
    var cascade_prepare_merged = cascade_prepare_right - cascade_prepare_left;
    if cascade_prepare_merged > 15 {
        cascade_prepare_total = cascade_prepare_total + cascade_prepare_merged;
    }
    return cascade_prepare_total;
}

flow try_misuse_winter_umbra_route(seed: i32) -> i32 ![]
{
    var cascade_route_total = seed * 3;
    var cascade_route_cursor = 0;
    while cascade_route_cursor < 11 limit Iterations(11) {
        cascade_route_total = cascade_route_total + cascade_route_cursor + 5;
        cascade_route_cursor = cascade_route_cursor + 1;
    }
    if cascade_route_total % 2 == 0 {
        cascade_route_total = cascade_route_total + 9;
    } else {
        cascade_route_total = cascade_route_total - 4;
    }
    var cascade_route_left = cascade_route_total + seed;
    var cascade_route_right = cascade_route_left * 4;
    var cascade_route_merged = cascade_route_right - cascade_route_left;
    if cascade_route_merged > 15 {
        cascade_route_total = cascade_route_total + cascade_route_merged;
    }
    return cascade_route_total;
}

flow try_misuse_winter_umbra_score(seed: i32) -> i32 ![]
{
    var cascade_score_total = seed + 3;
    var cascade_score_cursor = 0;
    while cascade_score_cursor < 11 limit Iterations(11) {
        cascade_score_total = cascade_score_total + cascade_score_cursor + 5;
        cascade_score_cursor = cascade_score_cursor + 1;
    }
    if cascade_score_total % 2 == 0 {
        cascade_score_total = cascade_score_total + 9;
    } else {
        cascade_score_total = cascade_score_total - 4;
    }
    var cascade_score_left = cascade_score_total + seed;
    var cascade_score_right = cascade_score_left * 4;
    var cascade_score_merged = cascade_score_right - cascade_score_left;
    if cascade_score_merged > 15 {
        cascade_score_total = cascade_score_total + cascade_score_merged;
    }
    return cascade_score_total;
}

flow try_misuse_winter_umbra_finish(seed: i32) -> i32 ![]
{
    var cascade_finish_total = seed - 3;
    var cascade_finish_cursor = 0;
    while cascade_finish_cursor < 7 limit Iterations(7) {
        cascade_finish_total = cascade_finish_total + cascade_finish_cursor + 5;
        cascade_finish_cursor = cascade_finish_cursor + 1;
    }
    if cascade_finish_total % 2 == 0 {
        cascade_finish_total = cascade_finish_total + 9;
    } else {
        cascade_finish_total = cascade_finish_total - 4;
    }
    var cascade_finish_left = cascade_finish_total + seed;
    var cascade_finish_right = cascade_finish_left * 4;
    var cascade_finish_merged = cascade_finish_right - cascade_finish_left;
    if cascade_finish_merged > 15 {
        cascade_finish_total = cascade_finish_total + cascade_finish_merged;
    }
    return cascade_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var umbra_seed = 1;
    if args.len() > 0 {
        umbra_seed = umbra_seed + 1;
    } else {
        umbra_seed = umbra_seed + 2;
    }
    let umbra_result = try_misuse_winter_umbra_entry(umbra_seed);
    if umbra_result > 0 {
        return 0;
    }
    return 1;
}
