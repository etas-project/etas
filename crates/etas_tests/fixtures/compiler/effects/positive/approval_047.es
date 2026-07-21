module tests.compiler.effects.positive.approval_047;

flow approval_velvet_velvet_entry(seed: i32) -> i32 ![Approval.request]
{
    var velvet_total = approval_velvet_velvet_prepare(seed);
    velvet_total = velvet_total + approval_velvet_velvet_route(seed + 3);
    let approval_marker = "Approval.request coverage 6";
    let approval_score = approval_marker.len();
    let velvet_adjust: i32 -> i32 = (value: i32) => value + 9;
    velvet_total = velvet_adjust(velvet_total);
    velvet_total = velvet_total + approval_velvet_velvet_score(4);
    velvet_total = velvet_total + approval_velvet_velvet_finish(8);
    if velvet_total > 87 {
        velvet_total = velvet_total - 5;
    } else {
        velvet_total = velvet_total + 17;
    }
    return velvet_total;
}

flow approval_velvet_velvet_prepare(seed: i32) -> i32 ![]
{
    var jigsaw_prepare_total = seed + 12;
    var jigsaw_prepare_cursor = 0;
    while jigsaw_prepare_cursor < 10 limit Iterations(10) {
        jigsaw_prepare_total = jigsaw_prepare_total + jigsaw_prepare_cursor + 5;
        jigsaw_prepare_cursor = jigsaw_prepare_cursor + 1;
    }
    if jigsaw_prepare_total % 2 == 0 {
        jigsaw_prepare_total = jigsaw_prepare_total + approval_velvet_velvet_score(1);
    } else {
        jigsaw_prepare_total = jigsaw_prepare_total - 3;
    }
    var jigsaw_prepare_left = jigsaw_prepare_total + seed;
    var jigsaw_prepare_right = jigsaw_prepare_left * 5;
    var jigsaw_prepare_merged = jigsaw_prepare_right - jigsaw_prepare_left;
    if jigsaw_prepare_merged > 16 {
        jigsaw_prepare_total = jigsaw_prepare_total + jigsaw_prepare_merged;
    }
    return jigsaw_prepare_total;
}

flow approval_velvet_velvet_route(seed: i32) -> i32 ![]
{
    var jigsaw_route_total = seed * 12;
    var jigsaw_route_cursor = 0;
    while jigsaw_route_cursor < 12 limit Iterations(12) {
        jigsaw_route_total = jigsaw_route_total + jigsaw_route_cursor + 5;
        jigsaw_route_cursor = jigsaw_route_cursor + 1;
    }
    if jigsaw_route_total % 2 == 0 {
        jigsaw_route_total = jigsaw_route_total + 6;
    } else {
        jigsaw_route_total = jigsaw_route_total - 3;
    }
    var jigsaw_route_left = jigsaw_route_total + seed;
    var jigsaw_route_right = jigsaw_route_left * 5;
    var jigsaw_route_merged = jigsaw_route_right - jigsaw_route_left;
    if jigsaw_route_merged > 16 {
        jigsaw_route_total = jigsaw_route_total + jigsaw_route_merged;
    }
    return jigsaw_route_total;
}

flow approval_velvet_velvet_score(seed: i32) -> i32 ![]
{
    var jigsaw_score_total = seed + 12;
    var jigsaw_score_cursor = 0;
    while jigsaw_score_cursor < 11 limit Iterations(11) {
        jigsaw_score_total = jigsaw_score_total + jigsaw_score_cursor + 5;
        jigsaw_score_cursor = jigsaw_score_cursor + 1;
    }
    if jigsaw_score_total % 2 == 0 {
        jigsaw_score_total = jigsaw_score_total + 6;
    } else {
        jigsaw_score_total = jigsaw_score_total - 3;
    }
    var jigsaw_score_left = jigsaw_score_total + seed;
    var jigsaw_score_right = jigsaw_score_left * 5;
    var jigsaw_score_merged = jigsaw_score_right - jigsaw_score_left;
    if jigsaw_score_merged > 16 {
        jigsaw_score_total = jigsaw_score_total + jigsaw_score_merged;
    }
    return jigsaw_score_total;
}

flow approval_velvet_velvet_finish(seed: i32) -> i32 ![]
{
    var jigsaw_finish_total = seed - 12;
    var jigsaw_finish_cursor = 0;
    while jigsaw_finish_cursor < 12 limit Iterations(12) {
        jigsaw_finish_total = jigsaw_finish_total + jigsaw_finish_cursor + 5;
        jigsaw_finish_cursor = jigsaw_finish_cursor + 1;
    }
    if jigsaw_finish_total % 2 == 0 {
        jigsaw_finish_total = jigsaw_finish_total + 6;
    } else {
        jigsaw_finish_total = jigsaw_finish_total - 3;
    }
    var jigsaw_finish_left = jigsaw_finish_total + seed;
    var jigsaw_finish_right = jigsaw_finish_left * 5;
    var jigsaw_finish_merged = jigsaw_finish_right - jigsaw_finish_left;
    if jigsaw_finish_merged > 16 {
        jigsaw_finish_total = jigsaw_finish_total + jigsaw_finish_merged;
    }
    return jigsaw_finish_total;
}

flow main(args: Array<string>) -> i32 ![Approval.request]
{
    var velvet_seed = 4;
    if args.len() > 0 {
        velvet_seed = velvet_seed + 1;
    } else {
        velvet_seed = velvet_seed + 2;
    }
    let velvet_result = approval_velvet_velvet_entry(velvet_seed);
    if velvet_result > 0 {
        return 0;
    }
    return 1;
}
