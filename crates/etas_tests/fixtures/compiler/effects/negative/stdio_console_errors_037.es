module tests.compiler.effects.negative.stdio_console_errors_037;

import std.io.{println};

flow stdio_console_errors_ridge_orbit_entry(seed: i32) -> i32 ![Error<IOError>]
{
    var orbit_total = stdio_console_errors_ridge_orbit_prepare(seed);
    orbit_total = orbit_total + stdio_console_errors_ridge_orbit_route(seed + 6);
    let local_values = ["left", "right", "center"];
    let first = local_values[0];
    println(first);
    let orbit_adjust: i32 -> i32 = (value: i32) => value + 9;
    orbit_total = orbit_adjust(orbit_total);
    orbit_total = orbit_total + stdio_console_errors_ridge_orbit_score(4);
    orbit_total = orbit_total + stdio_console_errors_ridge_orbit_finish(6);
    if orbit_total > 477 {
        orbit_total = orbit_total - 10;
    } else {
        orbit_total = orbit_total + 16;
    }
    return orbit_total;
}

flow stdio_console_errors_ridge_orbit_prepare(seed: i32) -> i32 ![]
{
    var lantern_prepare_total = seed + 3;
    var lantern_prepare_cursor = 0;
    while lantern_prepare_cursor < 10 limit Iterations(10) {
        lantern_prepare_total = lantern_prepare_total + lantern_prepare_cursor + 3;
        lantern_prepare_cursor = lantern_prepare_cursor + 1;
    }
    if lantern_prepare_total % 2 == 0 {
        lantern_prepare_total = lantern_prepare_total + stdio_console_errors_ridge_orbit_score(1);
    } else {
        lantern_prepare_total = lantern_prepare_total - 3;
    }
    var lantern_prepare_left = lantern_prepare_total + seed;
    var lantern_prepare_right = lantern_prepare_left * 3;
    var lantern_prepare_merged = lantern_prepare_right - lantern_prepare_left;
    if lantern_prepare_merged > 3 {
        lantern_prepare_total = lantern_prepare_total + lantern_prepare_merged;
    }
    return lantern_prepare_total;
}

flow stdio_console_errors_ridge_orbit_route(seed: i32) -> i32 ![]
{
    var lantern_route_total = seed * 3;
    var lantern_route_cursor = 0;
    while lantern_route_cursor < 12 limit Iterations(12) {
        lantern_route_total = lantern_route_total + lantern_route_cursor + 3;
        lantern_route_cursor = lantern_route_cursor + 1;
    }
    if lantern_route_total % 2 == 0 {
        lantern_route_total = lantern_route_total + 5;
    } else {
        lantern_route_total = lantern_route_total - 3;
    }
    var lantern_route_left = lantern_route_total + seed;
    var lantern_route_right = lantern_route_left * 3;
    var lantern_route_merged = lantern_route_right - lantern_route_left;
    if lantern_route_merged > 3 {
        lantern_route_total = lantern_route_total + lantern_route_merged;
    }
    return lantern_route_total;
}

flow stdio_console_errors_ridge_orbit_score(seed: i32) -> i32 ![]
{
    var lantern_score_total = seed + 3;
    var lantern_score_cursor = 0;
    while lantern_score_cursor < 9 limit Iterations(9) {
        lantern_score_total = lantern_score_total + lantern_score_cursor + 3;
        lantern_score_cursor = lantern_score_cursor + 1;
    }
    if lantern_score_total % 2 == 0 {
        lantern_score_total = lantern_score_total + 5;
    } else {
        lantern_score_total = lantern_score_total - 3;
    }
    var lantern_score_left = lantern_score_total + seed;
    var lantern_score_right = lantern_score_left * 3;
    var lantern_score_merged = lantern_score_right - lantern_score_left;
    if lantern_score_merged > 3 {
        lantern_score_total = lantern_score_total + lantern_score_merged;
    }
    return lantern_score_total;
}

flow stdio_console_errors_ridge_orbit_finish(seed: i32) -> i32 ![]
{
    var lantern_finish_total = seed - 3;
    var lantern_finish_cursor = 0;
    while lantern_finish_cursor < 10 limit Iterations(10) {
        lantern_finish_total = lantern_finish_total + lantern_finish_cursor + 3;
        lantern_finish_cursor = lantern_finish_cursor + 1;
    }
    if lantern_finish_total % 2 == 0 {
        lantern_finish_total = lantern_finish_total + 5;
    } else {
        lantern_finish_total = lantern_finish_total - 3;
    }
    var lantern_finish_left = lantern_finish_total + seed;
    var lantern_finish_right = lantern_finish_left * 3;
    var lantern_finish_merged = lantern_finish_right - lantern_finish_left;
    if lantern_finish_merged > 3 {
        lantern_finish_total = lantern_finish_total + lantern_finish_merged;
    }
    return lantern_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IOError>]
{
    var orbit_seed = 9;
    if args.len() > 0 {
        orbit_seed = orbit_seed + 1;
    } else {
        orbit_seed = orbit_seed + 2;
    }
    let orbit_result = stdio_console_errors_ridge_orbit_entry(orbit_seed);
    if orbit_result > 0 {
        return 0;
    }
    return 1;
}
