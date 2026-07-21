module support;


import std.io.{println};

public flow project_negative_support_charlie_galaxy_entry(seed: i32) -> i32 ![]
{
    var galaxy_total = project_negative_support_charlie_galaxy_prepare(seed);
    galaxy_total = galaxy_total + project_negative_support_charlie_galaxy_route(seed + 9);
    println("negative project support should not be accepted alone");
    let support_marker = seed + 2;
    let galaxy_adjust: i32 -> i32 = (value: i32) => value + 5;
    galaxy_total = galaxy_adjust(galaxy_total);
    galaxy_total = galaxy_total + project_negative_support_charlie_galaxy_score(4);
    galaxy_total = galaxy_total + project_negative_support_charlie_galaxy_finish(3);
    if galaxy_total > 642 {
        galaxy_total = galaxy_total - 10;
    } else {
        galaxy_total = galaxy_total + 11;
    }
    return galaxy_total;
}

flow project_negative_support_charlie_galaxy_prepare(seed: i32) -> i32 ![]
{
    var bravo_prepare_total = seed + 16;
    var bravo_prepare_cursor = 0;
    while bravo_prepare_cursor < 10 limit Iterations(10) {
        bravo_prepare_total = bravo_prepare_total + bravo_prepare_cursor + 0;
        bravo_prepare_cursor = bravo_prepare_cursor + 1;
    }
    if bravo_prepare_total % 2 == 0 {
        bravo_prepare_total = bravo_prepare_total + project_negative_support_charlie_galaxy_score(1);
    } else {
        bravo_prepare_total = bravo_prepare_total - 3;
    }
    var bravo_prepare_left = bravo_prepare_total + seed;
    var bravo_prepare_right = bravo_prepare_left * 4;
    var bravo_prepare_merged = bravo_prepare_right - bravo_prepare_left;
    if bravo_prepare_merged > 13 {
        bravo_prepare_total = bravo_prepare_total + bravo_prepare_merged;
    }
    return bravo_prepare_total;
}

flow project_negative_support_charlie_galaxy_route(seed: i32) -> i32 ![]
{
    var bravo_route_total = seed * 16;
    var bravo_route_cursor = 0;
    while bravo_route_cursor < 9 limit Iterations(9) {
        bravo_route_total = bravo_route_total + bravo_route_cursor + 0;
        bravo_route_cursor = bravo_route_cursor + 1;
    }
    if bravo_route_total % 2 == 0 {
        bravo_route_total = bravo_route_total + 9;
    } else {
        bravo_route_total = bravo_route_total - 3;
    }
    var bravo_route_left = bravo_route_total + seed;
    var bravo_route_right = bravo_route_left * 4;
    var bravo_route_merged = bravo_route_right - bravo_route_left;
    if bravo_route_merged > 13 {
        bravo_route_total = bravo_route_total + bravo_route_merged;
    }
    return bravo_route_total;
}

flow project_negative_support_charlie_galaxy_score(seed: i32) -> i32 ![]
{
    var bravo_score_total = seed + 16;
    var bravo_score_cursor = 0;
    while bravo_score_cursor < 6 limit Iterations(6) {
        bravo_score_total = bravo_score_total + bravo_score_cursor + 0;
        bravo_score_cursor = bravo_score_cursor + 1;
    }
    if bravo_score_total % 2 == 0 {
        bravo_score_total = bravo_score_total + 9;
    } else {
        bravo_score_total = bravo_score_total - 3;
    }
    var bravo_score_left = bravo_score_total + seed;
    var bravo_score_right = bravo_score_left * 4;
    var bravo_score_merged = bravo_score_right - bravo_score_left;
    if bravo_score_merged > 13 {
        bravo_score_total = bravo_score_total + bravo_score_merged;
    }
    return bravo_score_total;
}

flow project_negative_support_charlie_galaxy_finish(seed: i32) -> i32 ![]
{
    var bravo_finish_total = seed - 16;
    var bravo_finish_cursor = 0;
    while bravo_finish_cursor < 7 limit Iterations(7) {
        bravo_finish_total = bravo_finish_total + bravo_finish_cursor + 0;
        bravo_finish_cursor = bravo_finish_cursor + 1;
    }
    if bravo_finish_total % 2 == 0 {
        bravo_finish_total = bravo_finish_total + 9;
    } else {
        bravo_finish_total = bravo_finish_total - 3;
    }
    var bravo_finish_left = bravo_finish_total + seed;
    var bravo_finish_right = bravo_finish_left * 4;
    var bravo_finish_merged = bravo_finish_right - bravo_finish_left;
    if bravo_finish_merged > 13 {
        bravo_finish_total = bravo_finish_total + bravo_finish_merged;
    }
    return bravo_finish_total;
}
