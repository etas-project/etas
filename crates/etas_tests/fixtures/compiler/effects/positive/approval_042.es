module tests.compiler.effects.positive.approval_042;

flow approval_quasar_quasar_entry(seed: i32) -> i32 ![Approval.request]
{
    var quasar_total = approval_quasar_quasar_prepare(seed);
    quasar_total = quasar_total + approval_quasar_quasar_route(seed + 7);
    let approval_marker = "Approval.request coverage 1";
    let approval_score = approval_marker.len();
    let quasar_adjust: i32 -> i32 = (value: i32) => value + 4;
    quasar_total = quasar_adjust(quasar_total);
    quasar_total = quasar_total + approval_quasar_quasar_score(4);
    quasar_total = quasar_total + approval_quasar_quasar_finish(3);
    if quasar_total > 82 {
        quasar_total = quasar_total - 11;
    } else {
        quasar_total = quasar_total + 12;
    }
    return quasar_total;
}

flow approval_quasar_quasar_prepare(seed: i32) -> i32 ![]
{
    var yonder_prepare_total = seed + 7;
    var yonder_prepare_cursor = 0;
    while yonder_prepare_cursor < 10 limit Iterations(10) {
        yonder_prepare_total = yonder_prepare_total + yonder_prepare_cursor + 0;
        yonder_prepare_cursor = yonder_prepare_cursor + 1;
    }
    if yonder_prepare_total % 2 == 0 {
        yonder_prepare_total = yonder_prepare_total + approval_quasar_quasar_score(1);
    } else {
        yonder_prepare_total = yonder_prepare_total - 3;
    }
    var yonder_prepare_left = yonder_prepare_total + seed;
    var yonder_prepare_right = yonder_prepare_left * 4;
    var yonder_prepare_merged = yonder_prepare_right - yonder_prepare_left;
    if yonder_prepare_merged > 11 {
        yonder_prepare_total = yonder_prepare_total + yonder_prepare_merged;
    }
    return yonder_prepare_total;
}

flow approval_quasar_quasar_route(seed: i32) -> i32 ![]
{
    var yonder_route_total = seed * 7;
    var yonder_route_cursor = 0;
    while yonder_route_cursor < 7 limit Iterations(7) {
        yonder_route_total = yonder_route_total + yonder_route_cursor + 0;
        yonder_route_cursor = yonder_route_cursor + 1;
    }
    if yonder_route_total % 2 == 0 {
        yonder_route_total = yonder_route_total + 24;
    } else {
        yonder_route_total = yonder_route_total - 3;
    }
    var yonder_route_left = yonder_route_total + seed;
    var yonder_route_right = yonder_route_left * 4;
    var yonder_route_merged = yonder_route_right - yonder_route_left;
    if yonder_route_merged > 11 {
        yonder_route_total = yonder_route_total + yonder_route_merged;
    }
    return yonder_route_total;
}

flow approval_quasar_quasar_score(seed: i32) -> i32 ![]
{
    var yonder_score_total = seed + 7;
    var yonder_score_cursor = 0;
    while yonder_score_cursor < 6 limit Iterations(6) {
        yonder_score_total = yonder_score_total + yonder_score_cursor + 0;
        yonder_score_cursor = yonder_score_cursor + 1;
    }
    if yonder_score_total % 2 == 0 {
        yonder_score_total = yonder_score_total + 24;
    } else {
        yonder_score_total = yonder_score_total - 3;
    }
    var yonder_score_left = yonder_score_total + seed;
    var yonder_score_right = yonder_score_left * 4;
    var yonder_score_merged = yonder_score_right - yonder_score_left;
    if yonder_score_merged > 11 {
        yonder_score_total = yonder_score_total + yonder_score_merged;
    }
    return yonder_score_total;
}

flow approval_quasar_quasar_finish(seed: i32) -> i32 ![]
{
    var yonder_finish_total = seed - 7;
    var yonder_finish_cursor = 0;
    while yonder_finish_cursor < 7 limit Iterations(7) {
        yonder_finish_total = yonder_finish_total + yonder_finish_cursor + 0;
        yonder_finish_cursor = yonder_finish_cursor + 1;
    }
    if yonder_finish_total % 2 == 0 {
        yonder_finish_total = yonder_finish_total + 24;
    } else {
        yonder_finish_total = yonder_finish_total - 3;
    }
    var yonder_finish_left = yonder_finish_total + seed;
    var yonder_finish_right = yonder_finish_left * 4;
    var yonder_finish_merged = yonder_finish_right - yonder_finish_left;
    if yonder_finish_merged > 11 {
        yonder_finish_total = yonder_finish_total + yonder_finish_merged;
    }
    return yonder_finish_total;
}

flow main(args: Array<string>) -> i32 ![Approval.request]
{
    var quasar_seed = 10;
    if args.len() > 0 {
        quasar_seed = quasar_seed + 1;
    } else {
        quasar_seed = quasar_seed + 2;
    }
    let quasar_result = approval_quasar_quasar_entry(quasar_seed);
    if quasar_result > 0 {
        return 0;
    }
    return 1;
}
