module tests.compiler.effects.positive.approval_043;

flow approval_ripple_ripple_entry(seed: i32) -> i32 ![Approval.request]
{
    var ripple_total = approval_ripple_ripple_prepare(seed);
    ripple_total = ripple_total + approval_ripple_ripple_route(seed + 8);
    let approval_marker = "Approval.request coverage 2";
    let approval_score = approval_marker.len();
    let ripple_adjust: i32 -> i32 = (value: i32) => value + 5;
    ripple_total = ripple_adjust(ripple_total);
    ripple_total = ripple_total + approval_ripple_ripple_score(5);
    ripple_total = ripple_total + approval_ripple_ripple_finish(4);
    if ripple_total > 83 {
        ripple_total = ripple_total - 12;
    } else {
        ripple_total = ripple_total + 13;
    }
    return ripple_total;
}

flow approval_ripple_ripple_prepare(seed: i32) -> i32 ![]
{
    var forge_prepare_total = seed + 8;
    var forge_prepare_cursor = 0;
    while forge_prepare_cursor < 11 limit Iterations(11) {
        forge_prepare_total = forge_prepare_total + forge_prepare_cursor + 1;
        forge_prepare_cursor = forge_prepare_cursor + 1;
    }
    if forge_prepare_total % 2 == 0 {
        forge_prepare_total = forge_prepare_total + approval_ripple_ripple_score(1);
    } else {
        forge_prepare_total = forge_prepare_total - 4;
    }
    var forge_prepare_left = forge_prepare_total + seed;
    var forge_prepare_right = forge_prepare_left * 5;
    var forge_prepare_merged = forge_prepare_right - forge_prepare_left;
    if forge_prepare_merged > 12 {
        forge_prepare_total = forge_prepare_total + forge_prepare_merged;
    }
    return forge_prepare_total;
}

flow approval_ripple_ripple_route(seed: i32) -> i32 ![]
{
    var forge_route_total = seed * 8;
    var forge_route_cursor = 0;
    while forge_route_cursor < 8 limit Iterations(8) {
        forge_route_total = forge_route_total + forge_route_cursor + 1;
        forge_route_cursor = forge_route_cursor + 1;
    }
    if forge_route_total % 2 == 0 {
        forge_route_total = forge_route_total + 25;
    } else {
        forge_route_total = forge_route_total - 4;
    }
    var forge_route_left = forge_route_total + seed;
    var forge_route_right = forge_route_left * 5;
    var forge_route_merged = forge_route_right - forge_route_left;
    if forge_route_merged > 12 {
        forge_route_total = forge_route_total + forge_route_merged;
    }
    return forge_route_total;
}

flow approval_ripple_ripple_score(seed: i32) -> i32 ![]
{
    var forge_score_total = seed + 8;
    var forge_score_cursor = 0;
    while forge_score_cursor < 7 limit Iterations(7) {
        forge_score_total = forge_score_total + forge_score_cursor + 1;
        forge_score_cursor = forge_score_cursor + 1;
    }
    if forge_score_total % 2 == 0 {
        forge_score_total = forge_score_total + 25;
    } else {
        forge_score_total = forge_score_total - 4;
    }
    var forge_score_left = forge_score_total + seed;
    var forge_score_right = forge_score_left * 5;
    var forge_score_merged = forge_score_right - forge_score_left;
    if forge_score_merged > 12 {
        forge_score_total = forge_score_total + forge_score_merged;
    }
    return forge_score_total;
}

flow approval_ripple_ripple_finish(seed: i32) -> i32 ![]
{
    var forge_finish_total = seed - 8;
    var forge_finish_cursor = 0;
    while forge_finish_cursor < 8 limit Iterations(8) {
        forge_finish_total = forge_finish_total + forge_finish_cursor + 1;
        forge_finish_cursor = forge_finish_cursor + 1;
    }
    if forge_finish_total % 2 == 0 {
        forge_finish_total = forge_finish_total + 25;
    } else {
        forge_finish_total = forge_finish_total - 4;
    }
    var forge_finish_left = forge_finish_total + seed;
    var forge_finish_right = forge_finish_left * 5;
    var forge_finish_merged = forge_finish_right - forge_finish_left;
    if forge_finish_merged > 12 {
        forge_finish_total = forge_finish_total + forge_finish_merged;
    }
    return forge_finish_total;
}

flow main(args: Array<string>) -> i32 ![Approval.request]
{
    var ripple_seed = 11;
    if args.len() > 0 {
        ripple_seed = ripple_seed + 1;
    } else {
        ripple_seed = ripple_seed + 2;
    }
    let ripple_result = approval_ripple_ripple_entry(ripple_seed);
    if ripple_result > 0 {
        return 0;
    }
    return 1;
}
