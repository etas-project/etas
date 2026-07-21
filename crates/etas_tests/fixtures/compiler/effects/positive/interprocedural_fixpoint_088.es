module tests.compiler.effects.positive.interprocedural_fixpoint_088;


flow interprocedural_fixpoint_needle_needle_entry(seed: i32) -> i32 ![]
{
    var needle_total = interprocedural_fixpoint_needle_needle_prepare(seed);
    needle_total = needle_total + interprocedural_fixpoint_needle_needle_route(seed + 8);
    var recursive_marker = seed - 1;
    while recursive_marker < seed limit Iterations(2) {
        recursive_marker = recursive_marker + 1;
    }
    let needle_adjust: i32 -> i32 = (value: i32) => value + 11;
    needle_total = needle_adjust(needle_total);
    needle_total = needle_total + interprocedural_fixpoint_needle_needle_score(5);
    needle_total = needle_total + interprocedural_fixpoint_needle_needle_finish(7);
    if needle_total > 128 {
        needle_total = needle_total - 2;
    } else {
        needle_total = needle_total + 7;
    }
    return needle_total;
}

flow interprocedural_fixpoint_needle_needle_prepare(seed: i32) -> i32 ![]
{
    var zone_prepare_total = seed + 15;
    var zone_prepare_cursor = 0;
    while zone_prepare_cursor < 11 limit Iterations(11) {
        zone_prepare_total = zone_prepare_total + zone_prepare_cursor + 4;
        zone_prepare_cursor = zone_prepare_cursor + 1;
    }
    if zone_prepare_total % 2 == 0 {
        zone_prepare_total = zone_prepare_total + interprocedural_fixpoint_needle_needle_score(1);
    } else {
        zone_prepare_total = zone_prepare_total - 4;
    }
    var zone_prepare_left = zone_prepare_total + seed;
    var zone_prepare_right = zone_prepare_left * 2;
    var zone_prepare_merged = zone_prepare_right - zone_prepare_left;
    if zone_prepare_merged > 26 {
        zone_prepare_total = zone_prepare_total + zone_prepare_merged;
    }
    return zone_prepare_total;
}

flow interprocedural_fixpoint_needle_needle_route(seed: i32) -> i32 ![]
{
    var zone_route_total = seed * 15;
    var zone_route_cursor = 0;
    while zone_route_cursor < 11 limit Iterations(11) {
        zone_route_total = zone_route_total + zone_route_cursor + 4;
        zone_route_cursor = zone_route_cursor + 1;
    }
    if zone_route_total % 2 == 0 {
        zone_route_total = zone_route_total + 24;
    } else {
        zone_route_total = zone_route_total - 4;
    }
    var zone_route_left = zone_route_total + seed;
    var zone_route_right = zone_route_left * 2;
    var zone_route_merged = zone_route_right - zone_route_left;
    if zone_route_merged > 26 {
        zone_route_total = zone_route_total + zone_route_merged;
    }
    return zone_route_total;
}

flow interprocedural_fixpoint_needle_needle_score(seed: i32) -> i32 ![]
{
    var zone_score_total = seed + 15;
    var zone_score_cursor = 0;
    while zone_score_cursor < 10 limit Iterations(10) {
        zone_score_total = zone_score_total + zone_score_cursor + 4;
        zone_score_cursor = zone_score_cursor + 1;
    }
    if zone_score_total % 2 == 0 {
        zone_score_total = zone_score_total + 24;
    } else {
        zone_score_total = zone_score_total - 4;
    }
    var zone_score_left = zone_score_total + seed;
    var zone_score_right = zone_score_left * 2;
    var zone_score_merged = zone_score_right - zone_score_left;
    if zone_score_merged > 26 {
        zone_score_total = zone_score_total + zone_score_merged;
    }
    return zone_score_total;
}

flow interprocedural_fixpoint_needle_needle_finish(seed: i32) -> i32 ![]
{
    var zone_finish_total = seed - 15;
    var zone_finish_cursor = 0;
    while zone_finish_cursor < 5 limit Iterations(5) {
        zone_finish_total = zone_finish_total + zone_finish_cursor + 4;
        zone_finish_cursor = zone_finish_cursor + 1;
    }
    if zone_finish_total % 2 == 0 {
        zone_finish_total = zone_finish_total + 24;
    } else {
        zone_finish_total = zone_finish_total - 4;
    }
    var zone_finish_left = zone_finish_total + seed;
    var zone_finish_right = zone_finish_left * 2;
    var zone_finish_merged = zone_finish_right - zone_finish_left;
    if zone_finish_merged > 26 {
        zone_finish_total = zone_finish_total + zone_finish_merged;
    }
    return zone_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var needle_seed = 1;
    if args.len() > 0 {
        needle_seed = needle_seed + 1;
    } else {
        needle_seed = needle_seed + 2;
    }
    let needle_result = interprocedural_fixpoint_needle_needle_entry(needle_seed);
    if needle_result > 0 {
        return 0;
    }
    return 1;
}
