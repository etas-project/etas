module support;


public flow project_support_hotel_iron_entry(seed: i32) -> i32 ![]
{
    var iron_total = project_support_hotel_iron_prepare(seed);
    iron_total = iron_total + project_support_hotel_iron_route(seed + 1);
    let support_marker = seed + 7;
    let iron_adjust: i32 -> i32 = (value: i32) => value + 13;
    iron_total = iron_adjust(iron_total);
    iron_total = iron_total + project_support_hotel_iron_score(4);
    iron_total = iron_total + project_support_hotel_iron_finish(7);
    if iron_total > 247 {
        iron_total = iron_total - 11;
    } else {
        iron_total = iron_total + 7;
    }
    return iron_total;
}

flow project_support_hotel_iron_prepare(seed: i32) -> i32 ![]
{
    var needle_prepare_total = seed + 20;
    var needle_prepare_cursor = 0;
    while needle_prepare_cursor < 10 limit Iterations(10) {
        needle_prepare_total = needle_prepare_total + needle_prepare_cursor + 4;
        needle_prepare_cursor = needle_prepare_cursor + 1;
    }
    if needle_prepare_total % 2 == 0 {
        needle_prepare_total = needle_prepare_total + project_support_hotel_iron_score(1);
    } else {
        needle_prepare_total = needle_prepare_total - 3;
    }
    var needle_prepare_left = needle_prepare_total + seed;
    var needle_prepare_right = needle_prepare_left * 5;
    var needle_prepare_merged = needle_prepare_right - needle_prepare_left;
    if needle_prepare_merged > 21 {
        needle_prepare_total = needle_prepare_total + needle_prepare_merged;
    }
    return needle_prepare_total;
}

flow project_support_hotel_iron_route(seed: i32) -> i32 ![]
{
    var needle_route_total = seed * 20;
    var needle_route_cursor = 0;
    while needle_route_cursor < 10 limit Iterations(10) {
        needle_route_total = needle_route_total + needle_route_cursor + 4;
        needle_route_cursor = needle_route_cursor + 1;
    }
    if needle_route_total % 2 == 0 {
        needle_route_total = needle_route_total + 5;
    } else {
        needle_route_total = needle_route_total - 3;
    }
    var needle_route_left = needle_route_total + seed;
    var needle_route_right = needle_route_left * 5;
    var needle_route_merged = needle_route_right - needle_route_left;
    if needle_route_merged > 21 {
        needle_route_total = needle_route_total + needle_route_merged;
    }
    return needle_route_total;
}

flow project_support_hotel_iron_score(seed: i32) -> i32 ![]
{
    var needle_score_total = seed + 20;
    var needle_score_cursor = 0;
    while needle_score_cursor < 10 limit Iterations(10) {
        needle_score_total = needle_score_total + needle_score_cursor + 4;
        needle_score_cursor = needle_score_cursor + 1;
    }
    if needle_score_total % 2 == 0 {
        needle_score_total = needle_score_total + 5;
    } else {
        needle_score_total = needle_score_total - 3;
    }
    var needle_score_left = needle_score_total + seed;
    var needle_score_right = needle_score_left * 5;
    var needle_score_merged = needle_score_right - needle_score_left;
    if needle_score_merged > 21 {
        needle_score_total = needle_score_total + needle_score_merged;
    }
    return needle_score_total;
}

flow project_support_hotel_iron_finish(seed: i32) -> i32 ![]
{
    var needle_finish_total = seed - 20;
    var needle_finish_cursor = 0;
    while needle_finish_cursor < 12 limit Iterations(12) {
        needle_finish_total = needle_finish_total + needle_finish_cursor + 4;
        needle_finish_cursor = needle_finish_cursor + 1;
    }
    if needle_finish_total % 2 == 0 {
        needle_finish_total = needle_finish_total + 5;
    } else {
        needle_finish_total = needle_finish_total - 3;
    }
    var needle_finish_left = needle_finish_total + seed;
    var needle_finish_right = needle_finish_left * 5;
    var needle_finish_merged = needle_finish_right - needle_finish_left;
    if needle_finish_merged > 21 {
        needle_finish_total = needle_finish_total + needle_finish_merged;
    }
    return needle_finish_total;
}
