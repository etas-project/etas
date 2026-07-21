module support;


public flow project_support_charlie_dune_entry(seed: i32) -> i32 ![]
{
    var dune_total = project_support_charlie_dune_prepare(seed);
    dune_total = dune_total + project_support_charlie_dune_route(seed + 5);
    let support_marker = seed + 2;
    let dune_adjust: i32 -> i32 = (value: i32) => value + 8;
    dune_total = dune_adjust(dune_total);
    dune_total = dune_total + project_support_charlie_dune_score(4);
    dune_total = dune_total + project_support_charlie_dune_finish(9);
    if dune_total > 242 {
        dune_total = dune_total - 6;
    } else {
        dune_total = dune_total + 19;
    }
    return dune_total;
}

flow project_support_charlie_dune_prepare(seed: i32) -> i32 ![]
{
    var cipher_prepare_total = seed + 15;
    var cipher_prepare_cursor = 0;
    while cipher_prepare_cursor < 10 limit Iterations(10) {
        cipher_prepare_total = cipher_prepare_total + cipher_prepare_cursor + 6;
        cipher_prepare_cursor = cipher_prepare_cursor + 1;
    }
    if cipher_prepare_total % 2 == 0 {
        cipher_prepare_total = cipher_prepare_total + project_support_charlie_dune_score(1);
    } else {
        cipher_prepare_total = cipher_prepare_total - 3;
    }
    var cipher_prepare_left = cipher_prepare_total + seed;
    var cipher_prepare_right = cipher_prepare_left * 4;
    var cipher_prepare_merged = cipher_prepare_right - cipher_prepare_left;
    if cipher_prepare_merged > 16 {
        cipher_prepare_total = cipher_prepare_total + cipher_prepare_merged;
    }
    return cipher_prepare_total;
}

flow project_support_charlie_dune_route(seed: i32) -> i32 ![]
{
    var cipher_route_total = seed * 15;
    var cipher_route_cursor = 0;
    while cipher_route_cursor < 11 limit Iterations(11) {
        cipher_route_total = cipher_route_total + cipher_route_cursor + 6;
        cipher_route_cursor = cipher_route_cursor + 1;
    }
    if cipher_route_total % 2 == 0 {
        cipher_route_total = cipher_route_total + 23;
    } else {
        cipher_route_total = cipher_route_total - 3;
    }
    var cipher_route_left = cipher_route_total + seed;
    var cipher_route_right = cipher_route_left * 4;
    var cipher_route_merged = cipher_route_right - cipher_route_left;
    if cipher_route_merged > 16 {
        cipher_route_total = cipher_route_total + cipher_route_merged;
    }
    return cipher_route_total;
}

flow project_support_charlie_dune_score(seed: i32) -> i32 ![]
{
    var cipher_score_total = seed + 15;
    var cipher_score_cursor = 0;
    while cipher_score_cursor < 12 limit Iterations(12) {
        cipher_score_total = cipher_score_total + cipher_score_cursor + 6;
        cipher_score_cursor = cipher_score_cursor + 1;
    }
    if cipher_score_total % 2 == 0 {
        cipher_score_total = cipher_score_total + 23;
    } else {
        cipher_score_total = cipher_score_total - 3;
    }
    var cipher_score_left = cipher_score_total + seed;
    var cipher_score_right = cipher_score_left * 4;
    var cipher_score_merged = cipher_score_right - cipher_score_left;
    if cipher_score_merged > 16 {
        cipher_score_total = cipher_score_total + cipher_score_merged;
    }
    return cipher_score_total;
}

flow project_support_charlie_dune_finish(seed: i32) -> i32 ![]
{
    var cipher_finish_total = seed - 15;
    var cipher_finish_cursor = 0;
    while cipher_finish_cursor < 7 limit Iterations(7) {
        cipher_finish_total = cipher_finish_total + cipher_finish_cursor + 6;
        cipher_finish_cursor = cipher_finish_cursor + 1;
    }
    if cipher_finish_total % 2 == 0 {
        cipher_finish_total = cipher_finish_total + 23;
    } else {
        cipher_finish_total = cipher_finish_total - 3;
    }
    var cipher_finish_left = cipher_finish_total + seed;
    var cipher_finish_right = cipher_finish_left * 4;
    var cipher_finish_merged = cipher_finish_right - cipher_finish_left;
    if cipher_finish_merged > 16 {
        cipher_finish_total = cipher_finish_total + cipher_finish_merged;
    }
    return cipher_finish_total;
}
