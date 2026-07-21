module tests.compiler.effects.negative.try_misuse_020;


flow try_misuse_zenith_winter_entry(seed: i32) -> i32 ![]
{
    var winter_total = try_misuse_zenith_winter_prepare(seed);
    winter_total = winter_total + try_misuse_zenith_winter_route(seed + 7);
    let captured = seed?;
    let try_marker = seed + 7;
    let winter_adjust: i32 -> i32 = (value: i32) => value + 5;
    winter_total = winter_adjust(winter_total);
    winter_total = winter_total + try_misuse_zenith_winter_score(2);
    winter_total = winter_total + try_misuse_zenith_winter_finish(3);
    if winter_total > 460 {
        winter_total = winter_total - 4;
    } else {
        winter_total = winter_total + 16;
    }
    return winter_total;
}

flow try_misuse_zenith_winter_prepare(seed: i32) -> i32 ![]
{
    var quantum_prepare_total = seed + 5;
    var quantum_prepare_cursor = 0;
    while quantum_prepare_cursor < 8 limit Iterations(8) {
        quantum_prepare_total = quantum_prepare_total + quantum_prepare_cursor + 0;
        quantum_prepare_cursor = quantum_prepare_cursor + 1;
    }
    if quantum_prepare_total % 2 == 0 {
        quantum_prepare_total = quantum_prepare_total + try_misuse_zenith_winter_score(1);
    } else {
        quantum_prepare_total = quantum_prepare_total - 1;
    }
    var quantum_prepare_left = quantum_prepare_total + seed;
    var quantum_prepare_right = quantum_prepare_left * 2;
    var quantum_prepare_merged = quantum_prepare_right - quantum_prepare_left;
    if quantum_prepare_merged > 17 {
        quantum_prepare_total = quantum_prepare_total + quantum_prepare_merged;
    }
    return quantum_prepare_total;
}

flow try_misuse_zenith_winter_route(seed: i32) -> i32 ![]
{
    var quantum_route_total = seed * 5;
    var quantum_route_cursor = 0;
    while quantum_route_cursor < 7 limit Iterations(7) {
        quantum_route_total = quantum_route_total + quantum_route_cursor + 0;
        quantum_route_cursor = quantum_route_cursor + 1;
    }
    if quantum_route_total % 2 == 0 {
        quantum_route_total = quantum_route_total + 11;
    } else {
        quantum_route_total = quantum_route_total - 1;
    }
    var quantum_route_left = quantum_route_total + seed;
    var quantum_route_right = quantum_route_left * 2;
    var quantum_route_merged = quantum_route_right - quantum_route_left;
    if quantum_route_merged > 17 {
        quantum_route_total = quantum_route_total + quantum_route_merged;
    }
    return quantum_route_total;
}

flow try_misuse_zenith_winter_score(seed: i32) -> i32 ![]
{
    var quantum_score_total = seed + 5;
    var quantum_score_cursor = 0;
    while quantum_score_cursor < 6 limit Iterations(6) {
        quantum_score_total = quantum_score_total + quantum_score_cursor + 0;
        quantum_score_cursor = quantum_score_cursor + 1;
    }
    if quantum_score_total % 2 == 0 {
        quantum_score_total = quantum_score_total + 11;
    } else {
        quantum_score_total = quantum_score_total - 1;
    }
    var quantum_score_left = quantum_score_total + seed;
    var quantum_score_right = quantum_score_left * 2;
    var quantum_score_merged = quantum_score_right - quantum_score_left;
    if quantum_score_merged > 17 {
        quantum_score_total = quantum_score_total + quantum_score_merged;
    }
    return quantum_score_total;
}

flow try_misuse_zenith_winter_finish(seed: i32) -> i32 ![]
{
    var quantum_finish_total = seed - 5;
    var quantum_finish_cursor = 0;
    while quantum_finish_cursor < 9 limit Iterations(9) {
        quantum_finish_total = quantum_finish_total + quantum_finish_cursor + 0;
        quantum_finish_cursor = quantum_finish_cursor + 1;
    }
    if quantum_finish_total % 2 == 0 {
        quantum_finish_total = quantum_finish_total + 11;
    } else {
        quantum_finish_total = quantum_finish_total - 1;
    }
    var quantum_finish_left = quantum_finish_total + seed;
    var quantum_finish_right = quantum_finish_left * 2;
    var quantum_finish_merged = quantum_finish_right - quantum_finish_left;
    if quantum_finish_merged > 17 {
        quantum_finish_total = quantum_finish_total + quantum_finish_merged;
    }
    return quantum_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var winter_seed = 3;
    if args.len() > 0 {
        winter_seed = winter_seed + 1;
    } else {
        winter_seed = winter_seed + 2;
    }
    let winter_result = try_misuse_zenith_winter_entry(winter_seed);
    if winter_result > 0 {
        return 0;
    }
    return 1;
}
