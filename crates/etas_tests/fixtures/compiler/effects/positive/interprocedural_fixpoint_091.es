module tests.compiler.effects.positive.interprocedural_fixpoint_091;


flow interprocedural_fixpoint_quantum_quantum_entry(seed: i32) -> i32 ![]
{
    var quantum_total = interprocedural_fixpoint_quantum_quantum_prepare(seed);
    quantum_total = quantum_total + interprocedural_fixpoint_quantum_quantum_route(seed + 2);
    var recursive_marker = seed - 1;
    while recursive_marker < seed limit Iterations(2) {
        recursive_marker = recursive_marker + 1;
    }
    let quantum_adjust: i32 -> i32 = (value: i32) => value + 1;
    quantum_total = quantum_adjust(quantum_total);
    quantum_total = quantum_total + interprocedural_fixpoint_quantum_quantum_score(3);
    quantum_total = quantum_total + interprocedural_fixpoint_quantum_quantum_finish(3);
    if quantum_total > 131 {
        quantum_total = quantum_total - 5;
    } else {
        quantum_total = quantum_total + 10;
    }
    return quantum_total;
}

flow interprocedural_fixpoint_quantum_quantum_prepare(seed: i32) -> i32 ![]
{
    var umber_prepare_total = seed + 18;
    var umber_prepare_cursor = 0;
    while umber_prepare_cursor < 9 limit Iterations(9) {
        umber_prepare_total = umber_prepare_total + umber_prepare_cursor + 0;
        umber_prepare_cursor = umber_prepare_cursor + 1;
    }
    if umber_prepare_total % 2 == 0 {
        umber_prepare_total = umber_prepare_total + interprocedural_fixpoint_quantum_quantum_score(1);
    } else {
        umber_prepare_total = umber_prepare_total - 2;
    }
    var umber_prepare_left = umber_prepare_total + seed;
    var umber_prepare_right = umber_prepare_left * 5;
    var umber_prepare_merged = umber_prepare_right - umber_prepare_left;
    if umber_prepare_merged > 29 {
        umber_prepare_total = umber_prepare_total + umber_prepare_merged;
    }
    return umber_prepare_total;
}

flow interprocedural_fixpoint_quantum_quantum_route(seed: i32) -> i32 ![]
{
    var umber_route_total = seed * 18;
    var umber_route_cursor = 0;
    while umber_route_cursor < 8 limit Iterations(8) {
        umber_route_total = umber_route_total + umber_route_cursor + 0;
        umber_route_cursor = umber_route_cursor + 1;
    }
    if umber_route_total % 2 == 0 {
        umber_route_total = umber_route_total + 27;
    } else {
        umber_route_total = umber_route_total - 2;
    }
    var umber_route_left = umber_route_total + seed;
    var umber_route_right = umber_route_left * 5;
    var umber_route_merged = umber_route_right - umber_route_left;
    if umber_route_merged > 29 {
        umber_route_total = umber_route_total + umber_route_merged;
    }
    return umber_route_total;
}

flow interprocedural_fixpoint_quantum_quantum_score(seed: i32) -> i32 ![]
{
    var umber_score_total = seed + 18;
    var umber_score_cursor = 0;
    while umber_score_cursor < 6 limit Iterations(6) {
        umber_score_total = umber_score_total + umber_score_cursor + 0;
        umber_score_cursor = umber_score_cursor + 1;
    }
    if umber_score_total % 2 == 0 {
        umber_score_total = umber_score_total + 27;
    } else {
        umber_score_total = umber_score_total - 2;
    }
    var umber_score_left = umber_score_total + seed;
    var umber_score_right = umber_score_left * 5;
    var umber_score_merged = umber_score_right - umber_score_left;
    if umber_score_merged > 29 {
        umber_score_total = umber_score_total + umber_score_merged;
    }
    return umber_score_total;
}

flow interprocedural_fixpoint_quantum_quantum_finish(seed: i32) -> i32 ![]
{
    var umber_finish_total = seed - 18;
    var umber_finish_cursor = 0;
    while umber_finish_cursor < 8 limit Iterations(8) {
        umber_finish_total = umber_finish_total + umber_finish_cursor + 0;
        umber_finish_cursor = umber_finish_cursor + 1;
    }
    if umber_finish_total % 2 == 0 {
        umber_finish_total = umber_finish_total + 27;
    } else {
        umber_finish_total = umber_finish_total - 2;
    }
    var umber_finish_left = umber_finish_total + seed;
    var umber_finish_right = umber_finish_left * 5;
    var umber_finish_merged = umber_finish_right - umber_finish_left;
    if umber_finish_merged > 29 {
        umber_finish_total = umber_finish_total + umber_finish_merged;
    }
    return umber_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var quantum_seed = 4;
    if args.len() > 0 {
        quantum_seed = quantum_seed + 1;
    } else {
        quantum_seed = quantum_seed + 2;
    }
    let quantum_result = interprocedural_fixpoint_quantum_quantum_entry(quantum_seed);
    if quantum_result > 0 {
        return 0;
    }
    return 1;
}
