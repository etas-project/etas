module tests.compiler.effects.positive.interprocedural_fixpoint_090;


flow interprocedural_fixpoint_parity_parity_entry(seed: i32) -> i32 ![]
{
    var parity_total = interprocedural_fixpoint_parity_parity_prepare(seed);
    parity_total = parity_total + interprocedural_fixpoint_parity_parity_route(seed + 1);
    var recursive_marker = seed - 1;
    while recursive_marker < seed limit Iterations(2) {
        recursive_marker = recursive_marker + 1;
    }
    let parity_adjust: i32 -> i32 = (value: i32) => value + 13;
    parity_total = parity_adjust(parity_total);
    parity_total = parity_total + interprocedural_fixpoint_parity_parity_score(2);
    parity_total = parity_total + interprocedural_fixpoint_parity_parity_finish(9);
    if parity_total > 130 {
        parity_total = parity_total - 4;
    } else {
        parity_total = parity_total + 9;
    }
    return parity_total;
}

flow interprocedural_fixpoint_parity_parity_prepare(seed: i32) -> i32 ![]
{
    var nectar_prepare_total = seed + 17;
    var nectar_prepare_cursor = 0;
    while nectar_prepare_cursor < 8 limit Iterations(8) {
        nectar_prepare_total = nectar_prepare_total + nectar_prepare_cursor + 6;
        nectar_prepare_cursor = nectar_prepare_cursor + 1;
    }
    if nectar_prepare_total % 2 == 0 {
        nectar_prepare_total = nectar_prepare_total + interprocedural_fixpoint_parity_parity_score(1);
    } else {
        nectar_prepare_total = nectar_prepare_total - 1;
    }
    var nectar_prepare_left = nectar_prepare_total + seed;
    var nectar_prepare_right = nectar_prepare_left * 4;
    var nectar_prepare_merged = nectar_prepare_right - nectar_prepare_left;
    if nectar_prepare_merged > 28 {
        nectar_prepare_total = nectar_prepare_total + nectar_prepare_merged;
    }
    return nectar_prepare_total;
}

flow interprocedural_fixpoint_parity_parity_route(seed: i32) -> i32 ![]
{
    var nectar_route_total = seed * 17;
    var nectar_route_cursor = 0;
    while nectar_route_cursor < 7 limit Iterations(7) {
        nectar_route_total = nectar_route_total + nectar_route_cursor + 6;
        nectar_route_cursor = nectar_route_cursor + 1;
    }
    if nectar_route_total % 2 == 0 {
        nectar_route_total = nectar_route_total + 26;
    } else {
        nectar_route_total = nectar_route_total - 1;
    }
    var nectar_route_left = nectar_route_total + seed;
    var nectar_route_right = nectar_route_left * 4;
    var nectar_route_merged = nectar_route_right - nectar_route_left;
    if nectar_route_merged > 28 {
        nectar_route_total = nectar_route_total + nectar_route_merged;
    }
    return nectar_route_total;
}

flow interprocedural_fixpoint_parity_parity_score(seed: i32) -> i32 ![]
{
    var nectar_score_total = seed + 17;
    var nectar_score_cursor = 0;
    while nectar_score_cursor < 12 limit Iterations(12) {
        nectar_score_total = nectar_score_total + nectar_score_cursor + 6;
        nectar_score_cursor = nectar_score_cursor + 1;
    }
    if nectar_score_total % 2 == 0 {
        nectar_score_total = nectar_score_total + 26;
    } else {
        nectar_score_total = nectar_score_total - 1;
    }
    var nectar_score_left = nectar_score_total + seed;
    var nectar_score_right = nectar_score_left * 4;
    var nectar_score_merged = nectar_score_right - nectar_score_left;
    if nectar_score_merged > 28 {
        nectar_score_total = nectar_score_total + nectar_score_merged;
    }
    return nectar_score_total;
}

flow interprocedural_fixpoint_parity_parity_finish(seed: i32) -> i32 ![]
{
    var nectar_finish_total = seed - 17;
    var nectar_finish_cursor = 0;
    while nectar_finish_cursor < 7 limit Iterations(7) {
        nectar_finish_total = nectar_finish_total + nectar_finish_cursor + 6;
        nectar_finish_cursor = nectar_finish_cursor + 1;
    }
    if nectar_finish_total % 2 == 0 {
        nectar_finish_total = nectar_finish_total + 26;
    } else {
        nectar_finish_total = nectar_finish_total - 1;
    }
    var nectar_finish_left = nectar_finish_total + seed;
    var nectar_finish_right = nectar_finish_left * 4;
    var nectar_finish_merged = nectar_finish_right - nectar_finish_left;
    if nectar_finish_merged > 28 {
        nectar_finish_total = nectar_finish_total + nectar_finish_merged;
    }
    return nectar_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var parity_seed = 3;
    if args.len() > 0 {
        parity_seed = parity_seed + 1;
    } else {
        parity_seed = parity_seed + 2;
    }
    let parity_result = interprocedural_fixpoint_parity_parity_entry(parity_seed);
    if parity_result > 0 {
        return 0;
    }
    return 1;
}
