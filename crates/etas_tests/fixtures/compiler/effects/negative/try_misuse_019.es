module tests.compiler.effects.negative.try_misuse_019;


flow try_misuse_yonder_velvet_entry(seed: i32) -> i32 ![]
{
    var velvet_total = try_misuse_yonder_velvet_prepare(seed);
    velvet_total = velvet_total + try_misuse_yonder_velvet_route(seed + 6);
    let captured = seed?;
    let try_marker = seed + 6;
    let velvet_adjust: i32 -> i32 = (value: i32) => value + 4;
    velvet_total = velvet_adjust(velvet_total);
    velvet_total = velvet_total + try_misuse_yonder_velvet_score(6);
    velvet_total = velvet_total + try_misuse_yonder_velvet_finish(9);
    if velvet_total > 459 {
        velvet_total = velvet_total - 3;
    } else {
        velvet_total = velvet_total + 15;
    }
    return velvet_total;
}

flow try_misuse_yonder_velvet_prepare(seed: i32) -> i32 ![]
{
    var jigsaw_prepare_total = seed + 4;
    var jigsaw_prepare_cursor = 0;
    while jigsaw_prepare_cursor < 12 limit Iterations(12) {
        jigsaw_prepare_total = jigsaw_prepare_total + jigsaw_prepare_cursor + 6;
        jigsaw_prepare_cursor = jigsaw_prepare_cursor + 1;
    }
    if jigsaw_prepare_total % 2 == 0 {
        jigsaw_prepare_total = jigsaw_prepare_total + try_misuse_yonder_velvet_score(1);
    } else {
        jigsaw_prepare_total = jigsaw_prepare_total - 5;
    }
    var jigsaw_prepare_left = jigsaw_prepare_total + seed;
    var jigsaw_prepare_right = jigsaw_prepare_left * 5;
    var jigsaw_prepare_merged = jigsaw_prepare_right - jigsaw_prepare_left;
    if jigsaw_prepare_merged > 16 {
        jigsaw_prepare_total = jigsaw_prepare_total + jigsaw_prepare_merged;
    }
    return jigsaw_prepare_total;
}

flow try_misuse_yonder_velvet_route(seed: i32) -> i32 ![]
{
    var jigsaw_route_total = seed * 4;
    var jigsaw_route_cursor = 0;
    while jigsaw_route_cursor < 12 limit Iterations(12) {
        jigsaw_route_total = jigsaw_route_total + jigsaw_route_cursor + 6;
        jigsaw_route_cursor = jigsaw_route_cursor + 1;
    }
    if jigsaw_route_total % 2 == 0 {
        jigsaw_route_total = jigsaw_route_total + 10;
    } else {
        jigsaw_route_total = jigsaw_route_total - 5;
    }
    var jigsaw_route_left = jigsaw_route_total + seed;
    var jigsaw_route_right = jigsaw_route_left * 5;
    var jigsaw_route_merged = jigsaw_route_right - jigsaw_route_left;
    if jigsaw_route_merged > 16 {
        jigsaw_route_total = jigsaw_route_total + jigsaw_route_merged;
    }
    return jigsaw_route_total;
}

flow try_misuse_yonder_velvet_score(seed: i32) -> i32 ![]
{
    var jigsaw_score_total = seed + 4;
    var jigsaw_score_cursor = 0;
    while jigsaw_score_cursor < 12 limit Iterations(12) {
        jigsaw_score_total = jigsaw_score_total + jigsaw_score_cursor + 6;
        jigsaw_score_cursor = jigsaw_score_cursor + 1;
    }
    if jigsaw_score_total % 2 == 0 {
        jigsaw_score_total = jigsaw_score_total + 10;
    } else {
        jigsaw_score_total = jigsaw_score_total - 5;
    }
    var jigsaw_score_left = jigsaw_score_total + seed;
    var jigsaw_score_right = jigsaw_score_left * 5;
    var jigsaw_score_merged = jigsaw_score_right - jigsaw_score_left;
    if jigsaw_score_merged > 16 {
        jigsaw_score_total = jigsaw_score_total + jigsaw_score_merged;
    }
    return jigsaw_score_total;
}

flow try_misuse_yonder_velvet_finish(seed: i32) -> i32 ![]
{
    var jigsaw_finish_total = seed - 4;
    var jigsaw_finish_cursor = 0;
    while jigsaw_finish_cursor < 8 limit Iterations(8) {
        jigsaw_finish_total = jigsaw_finish_total + jigsaw_finish_cursor + 6;
        jigsaw_finish_cursor = jigsaw_finish_cursor + 1;
    }
    if jigsaw_finish_total % 2 == 0 {
        jigsaw_finish_total = jigsaw_finish_total + 10;
    } else {
        jigsaw_finish_total = jigsaw_finish_total - 5;
    }
    var jigsaw_finish_left = jigsaw_finish_total + seed;
    var jigsaw_finish_right = jigsaw_finish_left * 5;
    var jigsaw_finish_merged = jigsaw_finish_right - jigsaw_finish_left;
    if jigsaw_finish_merged > 16 {
        jigsaw_finish_total = jigsaw_finish_total + jigsaw_finish_merged;
    }
    return jigsaw_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var velvet_seed = 2;
    if args.len() > 0 {
        velvet_seed = velvet_seed + 1;
    } else {
        velvet_seed = velvet_seed + 2;
    }
    let velvet_result = try_misuse_yonder_velvet_entry(velvet_seed);
    if velvet_result > 0 {
        return 0;
    }
    return 1;
}
