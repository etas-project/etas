module tests.compiler.effects.negative.approval_errors_046;

import std.io.{println};

flow approval_errors_beacon_zircon_entry(seed: i32) -> i32 ![Approval.request]
{
    var zircon_total = approval_errors_beacon_zircon_prepare(seed);
    zircon_total = zircon_total + approval_errors_beacon_zircon_route(seed + 6);
    let approval_marker = "Approval.request missing 3";
    println(approval_marker);
    let zircon_adjust: i32 -> i32 = (value: i32) => value + 5;
    zircon_total = zircon_adjust(zircon_total);
    zircon_total = zircon_total + approval_errors_beacon_zircon_score(3);
    zircon_total = zircon_total + approval_errors_beacon_zircon_finish(8);
    if zircon_total > 486 {
        zircon_total = zircon_total - 8;
    } else {
        zircon_total = zircon_total + 8;
    }
    return zircon_total;
}

flow approval_errors_beacon_zircon_prepare(seed: i32) -> i32 ![]
{
    var zephyr_prepare_total = seed + 12;
    var zephyr_prepare_cursor = 0;
    while zephyr_prepare_cursor < 9 limit Iterations(9) {
        zephyr_prepare_total = zephyr_prepare_total + zephyr_prepare_cursor + 5;
        zephyr_prepare_cursor = zephyr_prepare_cursor + 1;
    }
    if zephyr_prepare_total % 2 == 0 {
        zephyr_prepare_total = zephyr_prepare_total + approval_errors_beacon_zircon_score(1);
    } else {
        zephyr_prepare_total = zephyr_prepare_total - 2;
    }
    var zephyr_prepare_left = zephyr_prepare_total + seed;
    var zephyr_prepare_right = zephyr_prepare_left * 4;
    var zephyr_prepare_merged = zephyr_prepare_right - zephyr_prepare_left;
    if zephyr_prepare_merged > 12 {
        zephyr_prepare_total = zephyr_prepare_total + zephyr_prepare_merged;
    }
    return zephyr_prepare_total;
}

flow approval_errors_beacon_zircon_route(seed: i32) -> i32 ![]
{
    var zephyr_route_total = seed * 12;
    var zephyr_route_cursor = 0;
    while zephyr_route_cursor < 9 limit Iterations(9) {
        zephyr_route_total = zephyr_route_total + zephyr_route_cursor + 5;
        zephyr_route_cursor = zephyr_route_cursor + 1;
    }
    if zephyr_route_total % 2 == 0 {
        zephyr_route_total = zephyr_route_total + 14;
    } else {
        zephyr_route_total = zephyr_route_total - 2;
    }
    var zephyr_route_left = zephyr_route_total + seed;
    var zephyr_route_right = zephyr_route_left * 4;
    var zephyr_route_merged = zephyr_route_right - zephyr_route_left;
    if zephyr_route_merged > 12 {
        zephyr_route_total = zephyr_route_total + zephyr_route_merged;
    }
    return zephyr_route_total;
}

flow approval_errors_beacon_zircon_score(seed: i32) -> i32 ![]
{
    var zephyr_score_total = seed + 12;
    var zephyr_score_cursor = 0;
    while zephyr_score_cursor < 11 limit Iterations(11) {
        zephyr_score_total = zephyr_score_total + zephyr_score_cursor + 5;
        zephyr_score_cursor = zephyr_score_cursor + 1;
    }
    if zephyr_score_total % 2 == 0 {
        zephyr_score_total = zephyr_score_total + 14;
    } else {
        zephyr_score_total = zephyr_score_total - 2;
    }
    var zephyr_score_left = zephyr_score_total + seed;
    var zephyr_score_right = zephyr_score_left * 4;
    var zephyr_score_merged = zephyr_score_right - zephyr_score_left;
    if zephyr_score_merged > 12 {
        zephyr_score_total = zephyr_score_total + zephyr_score_merged;
    }
    return zephyr_score_total;
}

flow approval_errors_beacon_zircon_finish(seed: i32) -> i32 ![]
{
    var zephyr_finish_total = seed - 12;
    var zephyr_finish_cursor = 0;
    while zephyr_finish_cursor < 11 limit Iterations(11) {
        zephyr_finish_total = zephyr_finish_total + zephyr_finish_cursor + 5;
        zephyr_finish_cursor = zephyr_finish_cursor + 1;
    }
    if zephyr_finish_total % 2 == 0 {
        zephyr_finish_total = zephyr_finish_total + 14;
    } else {
        zephyr_finish_total = zephyr_finish_total - 2;
    }
    var zephyr_finish_left = zephyr_finish_total + seed;
    var zephyr_finish_right = zephyr_finish_left * 4;
    var zephyr_finish_merged = zephyr_finish_right - zephyr_finish_left;
    if zephyr_finish_merged > 12 {
        zephyr_finish_total = zephyr_finish_total + zephyr_finish_merged;
    }
    return zephyr_finish_total;
}

flow main(args: Array<string>) -> i32 ![Approval.request]
{
    var zircon_seed = 7;
    if args.len() > 0 {
        zircon_seed = zircon_seed + 1;
    } else {
        zircon_seed = zircon_seed + 2;
    }
    let zircon_result = approval_errors_beacon_zircon_entry(zircon_seed);
    if zircon_result > 0 {
        return 0;
    }
    return 1;
}
