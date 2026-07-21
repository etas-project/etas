module tests.compiler.effects.positive.approval_045;

flow approval_tidal_tidal_entry(seed: i32) -> i32 ![Approval.request]
{
    var tidal_total = approval_tidal_tidal_prepare(seed);
    tidal_total = tidal_total + approval_tidal_tidal_route(seed + 1);
    let approval_marker = "Approval.request coverage 4";
    let approval_score = approval_marker.len();
    let tidal_adjust: i32 -> i32 = (value: i32) => value + 7;
    tidal_total = tidal_adjust(tidal_total);
    tidal_total = tidal_total + approval_tidal_tidal_score(2);
    tidal_total = tidal_total + approval_tidal_tidal_finish(6);
    if tidal_total > 85 {
        tidal_total = tidal_total - 3;
    } else {
        tidal_total = tidal_total + 15;
    }
    return tidal_total;
}

flow approval_tidal_tidal_prepare(seed: i32) -> i32 ![]
{
    var unity_prepare_total = seed + 10;
    var unity_prepare_cursor = 0;
    while unity_prepare_cursor < 8 limit Iterations(8) {
        unity_prepare_total = unity_prepare_total + unity_prepare_cursor + 3;
        unity_prepare_cursor = unity_prepare_cursor + 1;
    }
    if unity_prepare_total % 2 == 0 {
        unity_prepare_total = unity_prepare_total + approval_tidal_tidal_score(1);
    } else {
        unity_prepare_total = unity_prepare_total - 1;
    }
    var unity_prepare_left = unity_prepare_total + seed;
    var unity_prepare_right = unity_prepare_left * 3;
    var unity_prepare_merged = unity_prepare_right - unity_prepare_left;
    if unity_prepare_merged > 14 {
        unity_prepare_total = unity_prepare_total + unity_prepare_merged;
    }
    return unity_prepare_total;
}

flow approval_tidal_tidal_route(seed: i32) -> i32 ![]
{
    var unity_route_total = seed * 10;
    var unity_route_cursor = 0;
    while unity_route_cursor < 10 limit Iterations(10) {
        unity_route_total = unity_route_total + unity_route_cursor + 3;
        unity_route_cursor = unity_route_cursor + 1;
    }
    if unity_route_total % 2 == 0 {
        unity_route_total = unity_route_total + 27;
    } else {
        unity_route_total = unity_route_total - 1;
    }
    var unity_route_left = unity_route_total + seed;
    var unity_route_right = unity_route_left * 3;
    var unity_route_merged = unity_route_right - unity_route_left;
    if unity_route_merged > 14 {
        unity_route_total = unity_route_total + unity_route_merged;
    }
    return unity_route_total;
}

flow approval_tidal_tidal_score(seed: i32) -> i32 ![]
{
    var unity_score_total = seed + 10;
    var unity_score_cursor = 0;
    while unity_score_cursor < 9 limit Iterations(9) {
        unity_score_total = unity_score_total + unity_score_cursor + 3;
        unity_score_cursor = unity_score_cursor + 1;
    }
    if unity_score_total % 2 == 0 {
        unity_score_total = unity_score_total + 27;
    } else {
        unity_score_total = unity_score_total - 1;
    }
    var unity_score_left = unity_score_total + seed;
    var unity_score_right = unity_score_left * 3;
    var unity_score_merged = unity_score_right - unity_score_left;
    if unity_score_merged > 14 {
        unity_score_total = unity_score_total + unity_score_merged;
    }
    return unity_score_total;
}

flow approval_tidal_tidal_finish(seed: i32) -> i32 ![]
{
    var unity_finish_total = seed - 10;
    var unity_finish_cursor = 0;
    while unity_finish_cursor < 10 limit Iterations(10) {
        unity_finish_total = unity_finish_total + unity_finish_cursor + 3;
        unity_finish_cursor = unity_finish_cursor + 1;
    }
    if unity_finish_total % 2 == 0 {
        unity_finish_total = unity_finish_total + 27;
    } else {
        unity_finish_total = unity_finish_total - 1;
    }
    var unity_finish_left = unity_finish_total + seed;
    var unity_finish_right = unity_finish_left * 3;
    var unity_finish_merged = unity_finish_right - unity_finish_left;
    if unity_finish_merged > 14 {
        unity_finish_total = unity_finish_total + unity_finish_merged;
    }
    return unity_finish_total;
}

flow main(args: Array<string>) -> i32 ![Approval.request]
{
    var tidal_seed = 2;
    if args.len() > 0 {
        tidal_seed = tidal_seed + 1;
    } else {
        tidal_seed = tidal_seed + 2;
    }
    let tidal_result = approval_tidal_tidal_entry(tidal_seed);
    if tidal_result > 0 {
        return 0;
    }
    return 1;
}
