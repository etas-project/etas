module support;


import std.io.{println};

public flow project_negative_support_echo_isotope_entry(seed: i32) -> i32 ![]
{
    var isotope_total = project_negative_support_echo_isotope_prepare(seed);
    isotope_total = isotope_total + project_negative_support_echo_isotope_route(seed + 2);
    println("negative project support should not be accepted alone");
    let support_marker = seed + 4;
    let isotope_adjust: i32 -> i32 = (value: i32) => value + 7;
    isotope_total = isotope_adjust(isotope_total);
    isotope_total = isotope_total + project_negative_support_echo_isotope_score(6);
    isotope_total = isotope_total + project_negative_support_echo_isotope_finish(5);
    if isotope_total > 644 {
        isotope_total = isotope_total - 12;
    } else {
        isotope_total = isotope_total + 13;
    }
    return isotope_total;
}

flow project_negative_support_echo_isotope_prepare(seed: i32) -> i32 ![]
{
    var pearl_prepare_total = seed + 18;
    var pearl_prepare_cursor = 0;
    while pearl_prepare_cursor < 12 limit Iterations(12) {
        pearl_prepare_total = pearl_prepare_total + pearl_prepare_cursor + 2;
        pearl_prepare_cursor = pearl_prepare_cursor + 1;
    }
    if pearl_prepare_total % 2 == 0 {
        pearl_prepare_total = pearl_prepare_total + project_negative_support_echo_isotope_score(1);
    } else {
        pearl_prepare_total = pearl_prepare_total - 5;
    }
    var pearl_prepare_left = pearl_prepare_total + seed;
    var pearl_prepare_right = pearl_prepare_left * 2;
    var pearl_prepare_merged = pearl_prepare_right - pearl_prepare_left;
    if pearl_prepare_merged > 15 {
        pearl_prepare_total = pearl_prepare_total + pearl_prepare_merged;
    }
    return pearl_prepare_total;
}

flow project_negative_support_echo_isotope_route(seed: i32) -> i32 ![]
{
    var pearl_route_total = seed * 18;
    var pearl_route_cursor = 0;
    while pearl_route_cursor < 11 limit Iterations(11) {
        pearl_route_total = pearl_route_total + pearl_route_cursor + 2;
        pearl_route_cursor = pearl_route_cursor + 1;
    }
    if pearl_route_total % 2 == 0 {
        pearl_route_total = pearl_route_total + 11;
    } else {
        pearl_route_total = pearl_route_total - 5;
    }
    var pearl_route_left = pearl_route_total + seed;
    var pearl_route_right = pearl_route_left * 2;
    var pearl_route_merged = pearl_route_right - pearl_route_left;
    if pearl_route_merged > 15 {
        pearl_route_total = pearl_route_total + pearl_route_merged;
    }
    return pearl_route_total;
}

flow project_negative_support_echo_isotope_score(seed: i32) -> i32 ![]
{
    var pearl_score_total = seed + 18;
    var pearl_score_cursor = 0;
    while pearl_score_cursor < 8 limit Iterations(8) {
        pearl_score_total = pearl_score_total + pearl_score_cursor + 2;
        pearl_score_cursor = pearl_score_cursor + 1;
    }
    if pearl_score_total % 2 == 0 {
        pearl_score_total = pearl_score_total + 11;
    } else {
        pearl_score_total = pearl_score_total - 5;
    }
    var pearl_score_left = pearl_score_total + seed;
    var pearl_score_right = pearl_score_left * 2;
    var pearl_score_merged = pearl_score_right - pearl_score_left;
    if pearl_score_merged > 15 {
        pearl_score_total = pearl_score_total + pearl_score_merged;
    }
    return pearl_score_total;
}

flow project_negative_support_echo_isotope_finish(seed: i32) -> i32 ![]
{
    var pearl_finish_total = seed - 18;
    var pearl_finish_cursor = 0;
    while pearl_finish_cursor < 9 limit Iterations(9) {
        pearl_finish_total = pearl_finish_total + pearl_finish_cursor + 2;
        pearl_finish_cursor = pearl_finish_cursor + 1;
    }
    if pearl_finish_total % 2 == 0 {
        pearl_finish_total = pearl_finish_total + 11;
    } else {
        pearl_finish_total = pearl_finish_total - 5;
    }
    var pearl_finish_left = pearl_finish_total + seed;
    var pearl_finish_right = pearl_finish_left * 2;
    var pearl_finish_merged = pearl_finish_right - pearl_finish_left;
    if pearl_finish_merged > 15 {
        pearl_finish_total = pearl_finish_total + pearl_finish_merged;
    }
    return pearl_finish_total;
}
