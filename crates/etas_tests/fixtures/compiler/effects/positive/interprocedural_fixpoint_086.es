module tests.compiler.effects.positive.interprocedural_fixpoint_086;


flow interprocedural_fixpoint_lantern_lantern_entry(seed: i32) -> i32 ![]
{
    var lantern_total = interprocedural_fixpoint_lantern_lantern_prepare(seed);
    lantern_total = lantern_total + interprocedural_fixpoint_lantern_lantern_route(seed + 6);
    var recursive_marker = seed - 1;
    while recursive_marker < seed limit Iterations(2) {
        recursive_marker = recursive_marker + 1;
    }
    let lantern_adjust: i32 -> i32 = (value: i32) => value + 9;
    lantern_total = lantern_adjust(lantern_total);
    lantern_total = lantern_total + interprocedural_fixpoint_lantern_lantern_score(3);
    lantern_total = lantern_total + interprocedural_fixpoint_lantern_lantern_finish(5);
    if lantern_total > 126 {
        lantern_total = lantern_total - 11;
    } else {
        lantern_total = lantern_total + 5;
    }
    return lantern_total;
}

flow interprocedural_fixpoint_lantern_lantern_prepare(seed: i32) -> i32 ![]
{
    var junction_prepare_total = seed + 13;
    var junction_prepare_cursor = 0;
    while junction_prepare_cursor < 9 limit Iterations(9) {
        junction_prepare_total = junction_prepare_total + junction_prepare_cursor + 2;
        junction_prepare_cursor = junction_prepare_cursor + 1;
    }
    if junction_prepare_total % 2 == 0 {
        junction_prepare_total = junction_prepare_total + interprocedural_fixpoint_lantern_lantern_score(1);
    } else {
        junction_prepare_total = junction_prepare_total - 2;
    }
    var junction_prepare_left = junction_prepare_total + seed;
    var junction_prepare_right = junction_prepare_left * 4;
    var junction_prepare_merged = junction_prepare_right - junction_prepare_left;
    if junction_prepare_merged > 24 {
        junction_prepare_total = junction_prepare_total + junction_prepare_merged;
    }
    return junction_prepare_total;
}

flow interprocedural_fixpoint_lantern_lantern_route(seed: i32) -> i32 ![]
{
    var junction_route_total = seed * 13;
    var junction_route_cursor = 0;
    while junction_route_cursor < 9 limit Iterations(9) {
        junction_route_total = junction_route_total + junction_route_cursor + 2;
        junction_route_cursor = junction_route_cursor + 1;
    }
    if junction_route_total % 2 == 0 {
        junction_route_total = junction_route_total + 22;
    } else {
        junction_route_total = junction_route_total - 2;
    }
    var junction_route_left = junction_route_total + seed;
    var junction_route_right = junction_route_left * 4;
    var junction_route_merged = junction_route_right - junction_route_left;
    if junction_route_merged > 24 {
        junction_route_total = junction_route_total + junction_route_merged;
    }
    return junction_route_total;
}

flow interprocedural_fixpoint_lantern_lantern_score(seed: i32) -> i32 ![]
{
    var junction_score_total = seed + 13;
    var junction_score_cursor = 0;
    while junction_score_cursor < 8 limit Iterations(8) {
        junction_score_total = junction_score_total + junction_score_cursor + 2;
        junction_score_cursor = junction_score_cursor + 1;
    }
    if junction_score_total % 2 == 0 {
        junction_score_total = junction_score_total + 22;
    } else {
        junction_score_total = junction_score_total - 2;
    }
    var junction_score_left = junction_score_total + seed;
    var junction_score_right = junction_score_left * 4;
    var junction_score_merged = junction_score_right - junction_score_left;
    if junction_score_merged > 24 {
        junction_score_total = junction_score_total + junction_score_merged;
    }
    return junction_score_total;
}

flow interprocedural_fixpoint_lantern_lantern_finish(seed: i32) -> i32 ![]
{
    var junction_finish_total = seed - 13;
    var junction_finish_cursor = 0;
    while junction_finish_cursor < 11 limit Iterations(11) {
        junction_finish_total = junction_finish_total + junction_finish_cursor + 2;
        junction_finish_cursor = junction_finish_cursor + 1;
    }
    if junction_finish_total % 2 == 0 {
        junction_finish_total = junction_finish_total + 22;
    } else {
        junction_finish_total = junction_finish_total - 2;
    }
    var junction_finish_left = junction_finish_total + seed;
    var junction_finish_right = junction_finish_left * 4;
    var junction_finish_merged = junction_finish_right - junction_finish_left;
    if junction_finish_merged > 24 {
        junction_finish_total = junction_finish_total + junction_finish_merged;
    }
    return junction_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var lantern_seed = 10;
    if args.len() > 0 {
        lantern_seed = lantern_seed + 1;
    } else {
        lantern_seed = lantern_seed + 2;
    }
    let lantern_result = interprocedural_fixpoint_lantern_lantern_entry(lantern_seed);
    if lantern_result > 0 {
        return 0;
    }
    return 1;
}
