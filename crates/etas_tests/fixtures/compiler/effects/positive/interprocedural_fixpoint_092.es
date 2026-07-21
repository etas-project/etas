module tests.compiler.effects.positive.interprocedural_fixpoint_092;


flow interprocedural_fixpoint_rocket_rocket_entry(seed: i32) -> i32 ![]
{
    var rocket_total = interprocedural_fixpoint_rocket_rocket_prepare(seed);
    rocket_total = rocket_total + interprocedural_fixpoint_rocket_rocket_route(seed + 3);
    var recursive_marker = seed - 1;
    while recursive_marker < seed limit Iterations(2) {
        recursive_marker = recursive_marker + 1;
    }
    let rocket_adjust: i32 -> i32 = (value: i32) => value + 2;
    rocket_total = rocket_adjust(rocket_total);
    rocket_total = rocket_total + interprocedural_fixpoint_rocket_rocket_score(4);
    rocket_total = rocket_total + interprocedural_fixpoint_rocket_rocket_finish(4);
    if rocket_total > 132 {
        rocket_total = rocket_total - 6;
    } else {
        rocket_total = rocket_total + 11;
    }
    return rocket_total;
}

flow interprocedural_fixpoint_rocket_rocket_prepare(seed: i32) -> i32 ![]
{
    var boreal_prepare_total = seed + 19;
    var boreal_prepare_cursor = 0;
    while boreal_prepare_cursor < 10 limit Iterations(10) {
        boreal_prepare_total = boreal_prepare_total + boreal_prepare_cursor + 1;
        boreal_prepare_cursor = boreal_prepare_cursor + 1;
    }
    if boreal_prepare_total % 2 == 0 {
        boreal_prepare_total = boreal_prepare_total + interprocedural_fixpoint_rocket_rocket_score(1);
    } else {
        boreal_prepare_total = boreal_prepare_total - 3;
    }
    var boreal_prepare_left = boreal_prepare_total + seed;
    var boreal_prepare_right = boreal_prepare_left * 2;
    var boreal_prepare_merged = boreal_prepare_right - boreal_prepare_left;
    if boreal_prepare_merged > 30 {
        boreal_prepare_total = boreal_prepare_total + boreal_prepare_merged;
    }
    return boreal_prepare_total;
}

flow interprocedural_fixpoint_rocket_rocket_route(seed: i32) -> i32 ![]
{
    var boreal_route_total = seed * 19;
    var boreal_route_cursor = 0;
    while boreal_route_cursor < 9 limit Iterations(9) {
        boreal_route_total = boreal_route_total + boreal_route_cursor + 1;
        boreal_route_cursor = boreal_route_cursor + 1;
    }
    if boreal_route_total % 2 == 0 {
        boreal_route_total = boreal_route_total + 5;
    } else {
        boreal_route_total = boreal_route_total - 3;
    }
    var boreal_route_left = boreal_route_total + seed;
    var boreal_route_right = boreal_route_left * 2;
    var boreal_route_merged = boreal_route_right - boreal_route_left;
    if boreal_route_merged > 30 {
        boreal_route_total = boreal_route_total + boreal_route_merged;
    }
    return boreal_route_total;
}

flow interprocedural_fixpoint_rocket_rocket_score(seed: i32) -> i32 ![]
{
    var boreal_score_total = seed + 19;
    var boreal_score_cursor = 0;
    while boreal_score_cursor < 7 limit Iterations(7) {
        boreal_score_total = boreal_score_total + boreal_score_cursor + 1;
        boreal_score_cursor = boreal_score_cursor + 1;
    }
    if boreal_score_total % 2 == 0 {
        boreal_score_total = boreal_score_total + 5;
    } else {
        boreal_score_total = boreal_score_total - 3;
    }
    var boreal_score_left = boreal_score_total + seed;
    var boreal_score_right = boreal_score_left * 2;
    var boreal_score_merged = boreal_score_right - boreal_score_left;
    if boreal_score_merged > 30 {
        boreal_score_total = boreal_score_total + boreal_score_merged;
    }
    return boreal_score_total;
}

flow interprocedural_fixpoint_rocket_rocket_finish(seed: i32) -> i32 ![]
{
    var boreal_finish_total = seed - 19;
    var boreal_finish_cursor = 0;
    while boreal_finish_cursor < 9 limit Iterations(9) {
        boreal_finish_total = boreal_finish_total + boreal_finish_cursor + 1;
        boreal_finish_cursor = boreal_finish_cursor + 1;
    }
    if boreal_finish_total % 2 == 0 {
        boreal_finish_total = boreal_finish_total + 5;
    } else {
        boreal_finish_total = boreal_finish_total - 3;
    }
    var boreal_finish_left = boreal_finish_total + seed;
    var boreal_finish_right = boreal_finish_left * 2;
    var boreal_finish_merged = boreal_finish_right - boreal_finish_left;
    if boreal_finish_merged > 30 {
        boreal_finish_total = boreal_finish_total + boreal_finish_merged;
    }
    return boreal_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var rocket_seed = 5;
    if args.len() > 0 {
        rocket_seed = rocket_seed + 1;
    } else {
        rocket_seed = rocket_seed + 2;
    }
    let rocket_result = interprocedural_fixpoint_rocket_rocket_entry(rocket_seed);
    if rocket_result > 0 {
        return 0;
    }
    return 1;
}
