module tests.compiler.effects.positive.approval_050;

flow approval_zenith_zenith_entry(seed: i32) -> i32 ![Approval.request]
{
    var zenith_total = approval_zenith_zenith_prepare(seed);
    zenith_total = zenith_total + approval_zenith_zenith_route(seed + 6);
    let approval_marker = "Approval.request coverage 9";
    let approval_score = approval_marker.len();
    let zenith_adjust: i32 -> i32 = (value: i32) => value + 12;
    zenith_total = zenith_adjust(zenith_total);
    zenith_total = zenith_total + approval_zenith_zenith_score(2);
    zenith_total = zenith_total + approval_zenith_zenith_finish(4);
    if zenith_total > 90 {
        zenith_total = zenith_total - 8;
    } else {
        zenith_total = zenith_total + 20;
    }
    return zenith_total;
}

flow approval_zenith_zenith_prepare(seed: i32) -> i32 ![]
{
    var forest_prepare_total = seed + 15;
    var forest_prepare_cursor = 0;
    while forest_prepare_cursor < 8 limit Iterations(8) {
        forest_prepare_total = forest_prepare_total + forest_prepare_cursor + 1;
        forest_prepare_cursor = forest_prepare_cursor + 1;
    }
    if forest_prepare_total % 2 == 0 {
        forest_prepare_total = forest_prepare_total + approval_zenith_zenith_score(1);
    } else {
        forest_prepare_total = forest_prepare_total - 1;
    }
    var forest_prepare_left = forest_prepare_total + seed;
    var forest_prepare_right = forest_prepare_left * 4;
    var forest_prepare_merged = forest_prepare_right - forest_prepare_left;
    if forest_prepare_merged > 19 {
        forest_prepare_total = forest_prepare_total + forest_prepare_merged;
    }
    return forest_prepare_total;
}

flow approval_zenith_zenith_route(seed: i32) -> i32 ![]
{
    var forest_route_total = seed * 15;
    var forest_route_cursor = 0;
    while forest_route_cursor < 9 limit Iterations(9) {
        forest_route_total = forest_route_total + forest_route_cursor + 1;
        forest_route_cursor = forest_route_cursor + 1;
    }
    if forest_route_total % 2 == 0 {
        forest_route_total = forest_route_total + 9;
    } else {
        forest_route_total = forest_route_total - 1;
    }
    var forest_route_left = forest_route_total + seed;
    var forest_route_right = forest_route_left * 4;
    var forest_route_merged = forest_route_right - forest_route_left;
    if forest_route_merged > 19 {
        forest_route_total = forest_route_total + forest_route_merged;
    }
    return forest_route_total;
}

flow approval_zenith_zenith_score(seed: i32) -> i32 ![]
{
    var forest_score_total = seed + 15;
    var forest_score_cursor = 0;
    while forest_score_cursor < 7 limit Iterations(7) {
        forest_score_total = forest_score_total + forest_score_cursor + 1;
        forest_score_cursor = forest_score_cursor + 1;
    }
    if forest_score_total % 2 == 0 {
        forest_score_total = forest_score_total + 9;
    } else {
        forest_score_total = forest_score_total - 1;
    }
    var forest_score_left = forest_score_total + seed;
    var forest_score_right = forest_score_left * 4;
    var forest_score_merged = forest_score_right - forest_score_left;
    if forest_score_merged > 19 {
        forest_score_total = forest_score_total + forest_score_merged;
    }
    return forest_score_total;
}

flow approval_zenith_zenith_finish(seed: i32) -> i32 ![]
{
    var forest_finish_total = seed - 15;
    var forest_finish_cursor = 0;
    while forest_finish_cursor < 7 limit Iterations(7) {
        forest_finish_total = forest_finish_total + forest_finish_cursor + 1;
        forest_finish_cursor = forest_finish_cursor + 1;
    }
    if forest_finish_total % 2 == 0 {
        forest_finish_total = forest_finish_total + 9;
    } else {
        forest_finish_total = forest_finish_total - 1;
    }
    var forest_finish_left = forest_finish_total + seed;
    var forest_finish_right = forest_finish_left * 4;
    var forest_finish_merged = forest_finish_right - forest_finish_left;
    if forest_finish_merged > 19 {
        forest_finish_total = forest_finish_total + forest_finish_merged;
    }
    return forest_finish_total;
}

flow main(args: Array<string>) -> i32 ![Approval.request]
{
    var zenith_seed = 7;
    if args.len() > 0 {
        zenith_seed = zenith_seed + 1;
    } else {
        zenith_seed = zenith_seed + 2;
    }
    let zenith_result = approval_zenith_zenith_entry(zenith_seed);
    if zenith_result > 0 {
        return 0;
    }
    return 1;
}
