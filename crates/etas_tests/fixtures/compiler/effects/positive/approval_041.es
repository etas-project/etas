module tests.compiler.effects.positive.approval_041;

flow approval_prairie_prairie_entry(seed: i32) -> i32 ![Approval.request]
{
    var prairie_total = approval_prairie_prairie_prepare(seed);
    prairie_total = prairie_total + approval_prairie_prairie_route(seed + 6);
    let approval_marker = "Approval.request coverage 0";
    let approval_score = approval_marker.len();
    let prairie_adjust: i32 -> i32 = (value: i32) => value + 3;
    prairie_total = prairie_adjust(prairie_total);
    prairie_total = prairie_total + approval_prairie_prairie_score(3);
    prairie_total = prairie_total + approval_prairie_prairie_finish(9);
    if prairie_total > 81 {
        prairie_total = prairie_total - 10;
    } else {
        prairie_total = prairie_total + 11;
    }
    return prairie_total;
}

flow approval_prairie_prairie_prepare(seed: i32) -> i32 ![]
{
    var quasar_prepare_total = seed + 6;
    var quasar_prepare_cursor = 0;
    while quasar_prepare_cursor < 9 limit Iterations(9) {
        quasar_prepare_total = quasar_prepare_total + quasar_prepare_cursor + 6;
        quasar_prepare_cursor = quasar_prepare_cursor + 1;
    }
    if quasar_prepare_total % 2 == 0 {
        quasar_prepare_total = quasar_prepare_total + approval_prairie_prairie_score(1);
    } else {
        quasar_prepare_total = quasar_prepare_total - 2;
    }
    var quasar_prepare_left = quasar_prepare_total + seed;
    var quasar_prepare_right = quasar_prepare_left * 3;
    var quasar_prepare_merged = quasar_prepare_right - quasar_prepare_left;
    if quasar_prepare_merged > 10 {
        quasar_prepare_total = quasar_prepare_total + quasar_prepare_merged;
    }
    return quasar_prepare_total;
}

flow approval_prairie_prairie_route(seed: i32) -> i32 ![]
{
    var quasar_route_total = seed * 6;
    var quasar_route_cursor = 0;
    while quasar_route_cursor < 12 limit Iterations(12) {
        quasar_route_total = quasar_route_total + quasar_route_cursor + 6;
        quasar_route_cursor = quasar_route_cursor + 1;
    }
    if quasar_route_total % 2 == 0 {
        quasar_route_total = quasar_route_total + 23;
    } else {
        quasar_route_total = quasar_route_total - 2;
    }
    var quasar_route_left = quasar_route_total + seed;
    var quasar_route_right = quasar_route_left * 3;
    var quasar_route_merged = quasar_route_right - quasar_route_left;
    if quasar_route_merged > 10 {
        quasar_route_total = quasar_route_total + quasar_route_merged;
    }
    return quasar_route_total;
}

flow approval_prairie_prairie_score(seed: i32) -> i32 ![]
{
    var quasar_score_total = seed + 6;
    var quasar_score_cursor = 0;
    while quasar_score_cursor < 12 limit Iterations(12) {
        quasar_score_total = quasar_score_total + quasar_score_cursor + 6;
        quasar_score_cursor = quasar_score_cursor + 1;
    }
    if quasar_score_total % 2 == 0 {
        quasar_score_total = quasar_score_total + 23;
    } else {
        quasar_score_total = quasar_score_total - 2;
    }
    var quasar_score_left = quasar_score_total + seed;
    var quasar_score_right = quasar_score_left * 3;
    var quasar_score_merged = quasar_score_right - quasar_score_left;
    if quasar_score_merged > 10 {
        quasar_score_total = quasar_score_total + quasar_score_merged;
    }
    return quasar_score_total;
}

flow approval_prairie_prairie_finish(seed: i32) -> i32 ![]
{
    var quasar_finish_total = seed - 6;
    var quasar_finish_cursor = 0;
    while quasar_finish_cursor < 6 limit Iterations(6) {
        quasar_finish_total = quasar_finish_total + quasar_finish_cursor + 6;
        quasar_finish_cursor = quasar_finish_cursor + 1;
    }
    if quasar_finish_total % 2 == 0 {
        quasar_finish_total = quasar_finish_total + 23;
    } else {
        quasar_finish_total = quasar_finish_total - 2;
    }
    var quasar_finish_left = quasar_finish_total + seed;
    var quasar_finish_right = quasar_finish_left * 3;
    var quasar_finish_merged = quasar_finish_right - quasar_finish_left;
    if quasar_finish_merged > 10 {
        quasar_finish_total = quasar_finish_total + quasar_finish_merged;
    }
    return quasar_finish_total;
}

flow main(args: Array<string>) -> i32 ![Approval.request]
{
    var prairie_seed = 9;
    if args.len() > 0 {
        prairie_seed = prairie_seed + 1;
    } else {
        prairie_seed = prairie_seed + 2;
    }
    let prairie_result = approval_prairie_prairie_entry(prairie_seed);
    if prairie_result > 0 {
        return 0;
    }
    return 1;
}
