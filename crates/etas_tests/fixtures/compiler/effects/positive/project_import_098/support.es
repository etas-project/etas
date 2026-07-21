module support;


public flow project_support_foxtrot_garden_entry(seed: i32) -> i32 ![]
{
    var garden_total = project_support_foxtrot_garden_prepare(seed);
    garden_total = garden_total + project_support_foxtrot_garden_route(seed + 8);
    let support_marker = seed + 5;
    let garden_adjust: i32 -> i32 = (value: i32) => value + 11;
    garden_total = garden_adjust(garden_total);
    garden_total = garden_total + project_support_foxtrot_garden_score(2);
    garden_total = garden_total + project_support_foxtrot_garden_finish(5);
    if garden_total > 245 {
        garden_total = garden_total - 9;
    } else {
        garden_total = garden_total + 5;
    }
    return garden_total;
}

flow project_support_foxtrot_garden_prepare(seed: i32) -> i32 ![]
{
    var zircon_prepare_total = seed + 18;
    var zircon_prepare_cursor = 0;
    while zircon_prepare_cursor < 8 limit Iterations(8) {
        zircon_prepare_total = zircon_prepare_total + zircon_prepare_cursor + 2;
        zircon_prepare_cursor = zircon_prepare_cursor + 1;
    }
    if zircon_prepare_total % 2 == 0 {
        zircon_prepare_total = zircon_prepare_total + project_support_foxtrot_garden_score(1);
    } else {
        zircon_prepare_total = zircon_prepare_total - 1;
    }
    var zircon_prepare_left = zircon_prepare_total + seed;
    var zircon_prepare_right = zircon_prepare_left * 3;
    var zircon_prepare_merged = zircon_prepare_right - zircon_prepare_left;
    if zircon_prepare_merged > 19 {
        zircon_prepare_total = zircon_prepare_total + zircon_prepare_merged;
    }
    return zircon_prepare_total;
}

flow project_support_foxtrot_garden_route(seed: i32) -> i32 ![]
{
    var zircon_route_total = seed * 18;
    var zircon_route_cursor = 0;
    while zircon_route_cursor < 8 limit Iterations(8) {
        zircon_route_total = zircon_route_total + zircon_route_cursor + 2;
        zircon_route_cursor = zircon_route_cursor + 1;
    }
    if zircon_route_total % 2 == 0 {
        zircon_route_total = zircon_route_total + 26;
    } else {
        zircon_route_total = zircon_route_total - 1;
    }
    var zircon_route_left = zircon_route_total + seed;
    var zircon_route_right = zircon_route_left * 3;
    var zircon_route_merged = zircon_route_right - zircon_route_left;
    if zircon_route_merged > 19 {
        zircon_route_total = zircon_route_total + zircon_route_merged;
    }
    return zircon_route_total;
}

flow project_support_foxtrot_garden_score(seed: i32) -> i32 ![]
{
    var zircon_score_total = seed + 18;
    var zircon_score_cursor = 0;
    while zircon_score_cursor < 8 limit Iterations(8) {
        zircon_score_total = zircon_score_total + zircon_score_cursor + 2;
        zircon_score_cursor = zircon_score_cursor + 1;
    }
    if zircon_score_total % 2 == 0 {
        zircon_score_total = zircon_score_total + 26;
    } else {
        zircon_score_total = zircon_score_total - 1;
    }
    var zircon_score_left = zircon_score_total + seed;
    var zircon_score_right = zircon_score_left * 3;
    var zircon_score_merged = zircon_score_right - zircon_score_left;
    if zircon_score_merged > 19 {
        zircon_score_total = zircon_score_total + zircon_score_merged;
    }
    return zircon_score_total;
}

flow project_support_foxtrot_garden_finish(seed: i32) -> i32 ![]
{
    var zircon_finish_total = seed - 18;
    var zircon_finish_cursor = 0;
    while zircon_finish_cursor < 10 limit Iterations(10) {
        zircon_finish_total = zircon_finish_total + zircon_finish_cursor + 2;
        zircon_finish_cursor = zircon_finish_cursor + 1;
    }
    if zircon_finish_total % 2 == 0 {
        zircon_finish_total = zircon_finish_total + 26;
    } else {
        zircon_finish_total = zircon_finish_total - 1;
    }
    var zircon_finish_left = zircon_finish_total + seed;
    var zircon_finish_right = zircon_finish_left * 3;
    var zircon_finish_merged = zircon_finish_right - zircon_finish_left;
    if zircon_finish_merged > 19 {
        zircon_finish_total = zircon_finish_total + zircon_finish_merged;
    }
    return zircon_finish_total;
}
