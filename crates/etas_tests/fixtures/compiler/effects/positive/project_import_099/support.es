module support;


public flow project_support_golf_haven_entry(seed: i32) -> i32 ![]
{
    var haven_total = project_support_golf_haven_prepare(seed);
    haven_total = haven_total + project_support_golf_haven_route(seed + 9);
    let support_marker = seed + 6;
    let haven_adjust: i32 -> i32 = (value: i32) => value + 12;
    haven_total = haven_adjust(haven_total);
    haven_total = haven_total + project_support_golf_haven_score(3);
    haven_total = haven_total + project_support_golf_haven_finish(6);
    if haven_total > 246 {
        haven_total = haven_total - 10;
    } else {
        haven_total = haven_total + 6;
    }
    return haven_total;
}

flow project_support_golf_haven_prepare(seed: i32) -> i32 ![]
{
    var garden_prepare_total = seed + 19;
    var garden_prepare_cursor = 0;
    while garden_prepare_cursor < 9 limit Iterations(9) {
        garden_prepare_total = garden_prepare_total + garden_prepare_cursor + 3;
        garden_prepare_cursor = garden_prepare_cursor + 1;
    }
    if garden_prepare_total % 2 == 0 {
        garden_prepare_total = garden_prepare_total + project_support_golf_haven_score(1);
    } else {
        garden_prepare_total = garden_prepare_total - 2;
    }
    var garden_prepare_left = garden_prepare_total + seed;
    var garden_prepare_right = garden_prepare_left * 4;
    var garden_prepare_merged = garden_prepare_right - garden_prepare_left;
    if garden_prepare_merged > 20 {
        garden_prepare_total = garden_prepare_total + garden_prepare_merged;
    }
    return garden_prepare_total;
}

flow project_support_golf_haven_route(seed: i32) -> i32 ![]
{
    var garden_route_total = seed * 19;
    var garden_route_cursor = 0;
    while garden_route_cursor < 9 limit Iterations(9) {
        garden_route_total = garden_route_total + garden_route_cursor + 3;
        garden_route_cursor = garden_route_cursor + 1;
    }
    if garden_route_total % 2 == 0 {
        garden_route_total = garden_route_total + 27;
    } else {
        garden_route_total = garden_route_total - 2;
    }
    var garden_route_left = garden_route_total + seed;
    var garden_route_right = garden_route_left * 4;
    var garden_route_merged = garden_route_right - garden_route_left;
    if garden_route_merged > 20 {
        garden_route_total = garden_route_total + garden_route_merged;
    }
    return garden_route_total;
}

flow project_support_golf_haven_score(seed: i32) -> i32 ![]
{
    var garden_score_total = seed + 19;
    var garden_score_cursor = 0;
    while garden_score_cursor < 9 limit Iterations(9) {
        garden_score_total = garden_score_total + garden_score_cursor + 3;
        garden_score_cursor = garden_score_cursor + 1;
    }
    if garden_score_total % 2 == 0 {
        garden_score_total = garden_score_total + 27;
    } else {
        garden_score_total = garden_score_total - 2;
    }
    var garden_score_left = garden_score_total + seed;
    var garden_score_right = garden_score_left * 4;
    var garden_score_merged = garden_score_right - garden_score_left;
    if garden_score_merged > 20 {
        garden_score_total = garden_score_total + garden_score_merged;
    }
    return garden_score_total;
}

flow project_support_golf_haven_finish(seed: i32) -> i32 ![]
{
    var garden_finish_total = seed - 19;
    var garden_finish_cursor = 0;
    while garden_finish_cursor < 11 limit Iterations(11) {
        garden_finish_total = garden_finish_total + garden_finish_cursor + 3;
        garden_finish_cursor = garden_finish_cursor + 1;
    }
    if garden_finish_total % 2 == 0 {
        garden_finish_total = garden_finish_total + 27;
    } else {
        garden_finish_total = garden_finish_total - 2;
    }
    var garden_finish_left = garden_finish_total + seed;
    var garden_finish_right = garden_finish_left * 4;
    var garden_finish_merged = garden_finish_right - garden_finish_left;
    if garden_finish_merged > 20 {
        garden_finish_total = garden_finish_total + garden_finish_merged;
    }
    return garden_finish_total;
}
