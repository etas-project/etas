module tests.compiler.effects.positive.approval_046;

flow approval_umbra_umbra_entry(seed: i32) -> i32 ![Approval.request]
{
    var umbra_total = approval_umbra_umbra_prepare(seed);
    umbra_total = umbra_total + approval_umbra_umbra_route(seed + 2);
    let approval_marker = "Approval.request coverage 5";
    let approval_score = approval_marker.len();
    let umbra_adjust: i32 -> i32 = (value: i32) => value + 8;
    umbra_total = umbra_adjust(umbra_total);
    umbra_total = umbra_total + approval_umbra_umbra_score(3);
    umbra_total = umbra_total + approval_umbra_umbra_finish(7);
    if umbra_total > 86 {
        umbra_total = umbra_total - 4;
    } else {
        umbra_total = umbra_total + 16;
    }
    return umbra_total;
}

flow approval_umbra_umbra_prepare(seed: i32) -> i32 ![]
{
    var cascade_prepare_total = seed + 11;
    var cascade_prepare_cursor = 0;
    while cascade_prepare_cursor < 9 limit Iterations(9) {
        cascade_prepare_total = cascade_prepare_total + cascade_prepare_cursor + 4;
        cascade_prepare_cursor = cascade_prepare_cursor + 1;
    }
    if cascade_prepare_total % 2 == 0 {
        cascade_prepare_total = cascade_prepare_total + approval_umbra_umbra_score(1);
    } else {
        cascade_prepare_total = cascade_prepare_total - 2;
    }
    var cascade_prepare_left = cascade_prepare_total + seed;
    var cascade_prepare_right = cascade_prepare_left * 4;
    var cascade_prepare_merged = cascade_prepare_right - cascade_prepare_left;
    if cascade_prepare_merged > 15 {
        cascade_prepare_total = cascade_prepare_total + cascade_prepare_merged;
    }
    return cascade_prepare_total;
}

flow approval_umbra_umbra_route(seed: i32) -> i32 ![]
{
    var cascade_route_total = seed * 11;
    var cascade_route_cursor = 0;
    while cascade_route_cursor < 11 limit Iterations(11) {
        cascade_route_total = cascade_route_total + cascade_route_cursor + 4;
        cascade_route_cursor = cascade_route_cursor + 1;
    }
    if cascade_route_total % 2 == 0 {
        cascade_route_total = cascade_route_total + 5;
    } else {
        cascade_route_total = cascade_route_total - 2;
    }
    var cascade_route_left = cascade_route_total + seed;
    var cascade_route_right = cascade_route_left * 4;
    var cascade_route_merged = cascade_route_right - cascade_route_left;
    if cascade_route_merged > 15 {
        cascade_route_total = cascade_route_total + cascade_route_merged;
    }
    return cascade_route_total;
}

flow approval_umbra_umbra_score(seed: i32) -> i32 ![]
{
    var cascade_score_total = seed + 11;
    var cascade_score_cursor = 0;
    while cascade_score_cursor < 10 limit Iterations(10) {
        cascade_score_total = cascade_score_total + cascade_score_cursor + 4;
        cascade_score_cursor = cascade_score_cursor + 1;
    }
    if cascade_score_total % 2 == 0 {
        cascade_score_total = cascade_score_total + 5;
    } else {
        cascade_score_total = cascade_score_total - 2;
    }
    var cascade_score_left = cascade_score_total + seed;
    var cascade_score_right = cascade_score_left * 4;
    var cascade_score_merged = cascade_score_right - cascade_score_left;
    if cascade_score_merged > 15 {
        cascade_score_total = cascade_score_total + cascade_score_merged;
    }
    return cascade_score_total;
}

flow approval_umbra_umbra_finish(seed: i32) -> i32 ![]
{
    var cascade_finish_total = seed - 11;
    var cascade_finish_cursor = 0;
    while cascade_finish_cursor < 11 limit Iterations(11) {
        cascade_finish_total = cascade_finish_total + cascade_finish_cursor + 4;
        cascade_finish_cursor = cascade_finish_cursor + 1;
    }
    if cascade_finish_total % 2 == 0 {
        cascade_finish_total = cascade_finish_total + 5;
    } else {
        cascade_finish_total = cascade_finish_total - 2;
    }
    var cascade_finish_left = cascade_finish_total + seed;
    var cascade_finish_right = cascade_finish_left * 4;
    var cascade_finish_merged = cascade_finish_right - cascade_finish_left;
    if cascade_finish_merged > 15 {
        cascade_finish_total = cascade_finish_total + cascade_finish_merged;
    }
    return cascade_finish_total;
}

flow main(args: Array<string>) -> i32 ![Approval.request]
{
    var umbra_seed = 3;
    if args.len() > 0 {
        umbra_seed = umbra_seed + 1;
    } else {
        umbra_seed = umbra_seed + 2;
    }
    let umbra_result = approval_umbra_umbra_entry(umbra_seed);
    if umbra_result > 0 {
        return 0;
    }
    return 1;
}
