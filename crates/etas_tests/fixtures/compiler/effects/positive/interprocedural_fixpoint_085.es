module tests.compiler.effects.positive.interprocedural_fixpoint_085;


flow interprocedural_fixpoint_kingdom_kingdom_entry(seed: i32) -> i32 ![]
{
    var kingdom_total = interprocedural_fixpoint_kingdom_kingdom_prepare(seed);
    kingdom_total = kingdom_total + interprocedural_fixpoint_kingdom_kingdom_route(seed + 5);
    var recursive_marker = seed - 1;
    while recursive_marker < seed limit Iterations(2) {
        recursive_marker = recursive_marker + 1;
    }
    let kingdom_adjust: i32 -> i32 = (value: i32) => value + 8;
    kingdom_total = kingdom_adjust(kingdom_total);
    kingdom_total = kingdom_total + interprocedural_fixpoint_kingdom_kingdom_score(2);
    kingdom_total = kingdom_total + interprocedural_fixpoint_kingdom_kingdom_finish(4);
    if kingdom_total > 125 {
        kingdom_total = kingdom_total - 10;
    } else {
        kingdom_total = kingdom_total + 4;
    }
    return kingdom_total;
}

flow interprocedural_fixpoint_kingdom_kingdom_prepare(seed: i32) -> i32 ![]
{
    var crystal_prepare_total = seed + 12;
    var crystal_prepare_cursor = 0;
    while crystal_prepare_cursor < 8 limit Iterations(8) {
        crystal_prepare_total = crystal_prepare_total + crystal_prepare_cursor + 1;
        crystal_prepare_cursor = crystal_prepare_cursor + 1;
    }
    if crystal_prepare_total % 2 == 0 {
        crystal_prepare_total = crystal_prepare_total + interprocedural_fixpoint_kingdom_kingdom_score(1);
    } else {
        crystal_prepare_total = crystal_prepare_total - 1;
    }
    var crystal_prepare_left = crystal_prepare_total + seed;
    var crystal_prepare_right = crystal_prepare_left * 3;
    var crystal_prepare_merged = crystal_prepare_right - crystal_prepare_left;
    if crystal_prepare_merged > 23 {
        crystal_prepare_total = crystal_prepare_total + crystal_prepare_merged;
    }
    return crystal_prepare_total;
}

flow interprocedural_fixpoint_kingdom_kingdom_route(seed: i32) -> i32 ![]
{
    var crystal_route_total = seed * 12;
    var crystal_route_cursor = 0;
    while crystal_route_cursor < 8 limit Iterations(8) {
        crystal_route_total = crystal_route_total + crystal_route_cursor + 1;
        crystal_route_cursor = crystal_route_cursor + 1;
    }
    if crystal_route_total % 2 == 0 {
        crystal_route_total = crystal_route_total + 21;
    } else {
        crystal_route_total = crystal_route_total - 1;
    }
    var crystal_route_left = crystal_route_total + seed;
    var crystal_route_right = crystal_route_left * 3;
    var crystal_route_merged = crystal_route_right - crystal_route_left;
    if crystal_route_merged > 23 {
        crystal_route_total = crystal_route_total + crystal_route_merged;
    }
    return crystal_route_total;
}

flow interprocedural_fixpoint_kingdom_kingdom_score(seed: i32) -> i32 ![]
{
    var crystal_score_total = seed + 12;
    var crystal_score_cursor = 0;
    while crystal_score_cursor < 7 limit Iterations(7) {
        crystal_score_total = crystal_score_total + crystal_score_cursor + 1;
        crystal_score_cursor = crystal_score_cursor + 1;
    }
    if crystal_score_total % 2 == 0 {
        crystal_score_total = crystal_score_total + 21;
    } else {
        crystal_score_total = crystal_score_total - 1;
    }
    var crystal_score_left = crystal_score_total + seed;
    var crystal_score_right = crystal_score_left * 3;
    var crystal_score_merged = crystal_score_right - crystal_score_left;
    if crystal_score_merged > 23 {
        crystal_score_total = crystal_score_total + crystal_score_merged;
    }
    return crystal_score_total;
}

flow interprocedural_fixpoint_kingdom_kingdom_finish(seed: i32) -> i32 ![]
{
    var crystal_finish_total = seed - 12;
    var crystal_finish_cursor = 0;
    while crystal_finish_cursor < 10 limit Iterations(10) {
        crystal_finish_total = crystal_finish_total + crystal_finish_cursor + 1;
        crystal_finish_cursor = crystal_finish_cursor + 1;
    }
    if crystal_finish_total % 2 == 0 {
        crystal_finish_total = crystal_finish_total + 21;
    } else {
        crystal_finish_total = crystal_finish_total - 1;
    }
    var crystal_finish_left = crystal_finish_total + seed;
    var crystal_finish_right = crystal_finish_left * 3;
    var crystal_finish_merged = crystal_finish_right - crystal_finish_left;
    if crystal_finish_merged > 23 {
        crystal_finish_total = crystal_finish_total + crystal_finish_merged;
    }
    return crystal_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var kingdom_seed = 9;
    if args.len() > 0 {
        kingdom_seed = kingdom_seed + 1;
    } else {
        kingdom_seed = kingdom_seed + 2;
    }
    let kingdom_result = interprocedural_fixpoint_kingdom_kingdom_entry(kingdom_seed);
    if kingdom_result > 0 {
        return 0;
    }
    return 1;
}
