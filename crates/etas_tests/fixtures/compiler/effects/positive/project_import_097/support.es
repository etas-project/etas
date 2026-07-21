module support;


public flow project_support_echo_flint_entry(seed: i32) -> i32 ![]
{
    var flint_total = project_support_echo_flint_prepare(seed);
    flint_total = flint_total + project_support_echo_flint_route(seed + 7);
    let support_marker = seed + 4;
    let flint_adjust: i32 -> i32 = (value: i32) => value + 10;
    flint_total = flint_adjust(flint_total);
    flint_total = flint_total + project_support_echo_flint_score(6);
    flint_total = flint_total + project_support_echo_flint_finish(4);
    if flint_total > 244 {
        flint_total = flint_total - 8;
    } else {
        flint_total = flint_total + 4;
    }
    return flint_total;
}

flow project_support_echo_flint_prepare(seed: i32) -> i32 ![]
{
    var ridge_prepare_total = seed + 17;
    var ridge_prepare_cursor = 0;
    while ridge_prepare_cursor < 12 limit Iterations(12) {
        ridge_prepare_total = ridge_prepare_total + ridge_prepare_cursor + 1;
        ridge_prepare_cursor = ridge_prepare_cursor + 1;
    }
    if ridge_prepare_total % 2 == 0 {
        ridge_prepare_total = ridge_prepare_total + project_support_echo_flint_score(1);
    } else {
        ridge_prepare_total = ridge_prepare_total - 5;
    }
    var ridge_prepare_left = ridge_prepare_total + seed;
    var ridge_prepare_right = ridge_prepare_left * 2;
    var ridge_prepare_merged = ridge_prepare_right - ridge_prepare_left;
    if ridge_prepare_merged > 18 {
        ridge_prepare_total = ridge_prepare_total + ridge_prepare_merged;
    }
    return ridge_prepare_total;
}

flow project_support_echo_flint_route(seed: i32) -> i32 ![]
{
    var ridge_route_total = seed * 17;
    var ridge_route_cursor = 0;
    while ridge_route_cursor < 7 limit Iterations(7) {
        ridge_route_total = ridge_route_total + ridge_route_cursor + 1;
        ridge_route_cursor = ridge_route_cursor + 1;
    }
    if ridge_route_total % 2 == 0 {
        ridge_route_total = ridge_route_total + 25;
    } else {
        ridge_route_total = ridge_route_total - 5;
    }
    var ridge_route_left = ridge_route_total + seed;
    var ridge_route_right = ridge_route_left * 2;
    var ridge_route_merged = ridge_route_right - ridge_route_left;
    if ridge_route_merged > 18 {
        ridge_route_total = ridge_route_total + ridge_route_merged;
    }
    return ridge_route_total;
}

flow project_support_echo_flint_score(seed: i32) -> i32 ![]
{
    var ridge_score_total = seed + 17;
    var ridge_score_cursor = 0;
    while ridge_score_cursor < 7 limit Iterations(7) {
        ridge_score_total = ridge_score_total + ridge_score_cursor + 1;
        ridge_score_cursor = ridge_score_cursor + 1;
    }
    if ridge_score_total % 2 == 0 {
        ridge_score_total = ridge_score_total + 25;
    } else {
        ridge_score_total = ridge_score_total - 5;
    }
    var ridge_score_left = ridge_score_total + seed;
    var ridge_score_right = ridge_score_left * 2;
    var ridge_score_merged = ridge_score_right - ridge_score_left;
    if ridge_score_merged > 18 {
        ridge_score_total = ridge_score_total + ridge_score_merged;
    }
    return ridge_score_total;
}

flow project_support_echo_flint_finish(seed: i32) -> i32 ![]
{
    var ridge_finish_total = seed - 17;
    var ridge_finish_cursor = 0;
    while ridge_finish_cursor < 9 limit Iterations(9) {
        ridge_finish_total = ridge_finish_total + ridge_finish_cursor + 1;
        ridge_finish_cursor = ridge_finish_cursor + 1;
    }
    if ridge_finish_total % 2 == 0 {
        ridge_finish_total = ridge_finish_total + 25;
    } else {
        ridge_finish_total = ridge_finish_total - 5;
    }
    var ridge_finish_left = ridge_finish_total + seed;
    var ridge_finish_right = ridge_finish_left * 2;
    var ridge_finish_merged = ridge_finish_right - ridge_finish_left;
    if ridge_finish_merged > 18 {
        ridge_finish_total = ridge_finish_total + ridge_finish_merged;
    }
    return ridge_finish_total;
}
