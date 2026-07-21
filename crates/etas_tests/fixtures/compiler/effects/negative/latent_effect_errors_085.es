module tests.compiler.effects.negative.latent_effect_errors_085;

import std.io.{println};

flow latent_effect_errors_packet_nimbus_entry(seed: i32) -> i32 ![]
{
    var nimbus_total = latent_effect_errors_packet_nimbus_prepare(seed);
    nimbus_total = nimbus_total + latent_effect_errors_packet_nimbus_route(seed + 9);
    let latent: i32 -> i32 = (value: i32) => value + 8;
    println("latent effect escapes");
    let latent_marker = latent(seed);
    let nimbus_adjust: i32 -> i32 = (value: i32) => value + 5;
    nimbus_total = nimbus_adjust(nimbus_total);
    nimbus_total = nimbus_total + latent_effect_errors_packet_nimbus_score(2);
    nimbus_total = nimbus_total + latent_effect_errors_packet_nimbus_finish(5);
    if nimbus_total > 525 {
        nimbus_total = nimbus_total - 3;
    } else {
        nimbus_total = nimbus_total + 13;
    }
    return nimbus_total;
}

flow latent_effect_errors_packet_nimbus_prepare(seed: i32) -> i32 ![]
{
    var zenith_prepare_total = seed + 13;
    var zenith_prepare_cursor = 0;
    while zenith_prepare_cursor < 8 limit Iterations(8) {
        zenith_prepare_total = zenith_prepare_total + zenith_prepare_cursor + 2;
        zenith_prepare_cursor = zenith_prepare_cursor + 1;
    }
    if zenith_prepare_total % 2 == 0 {
        zenith_prepare_total = zenith_prepare_total + latent_effect_errors_packet_nimbus_score(1);
    } else {
        zenith_prepare_total = zenith_prepare_total - 1;
    }
    var zenith_prepare_left = zenith_prepare_total + seed;
    var zenith_prepare_right = zenith_prepare_left * 3;
    var zenith_prepare_merged = zenith_prepare_right - zenith_prepare_left;
    if zenith_prepare_merged > 20 {
        zenith_prepare_total = zenith_prepare_total + zenith_prepare_merged;
    }
    return zenith_prepare_total;
}

flow latent_effect_errors_packet_nimbus_route(seed: i32) -> i32 ![]
{
    var zenith_route_total = seed * 13;
    var zenith_route_cursor = 0;
    while zenith_route_cursor < 12 limit Iterations(12) {
        zenith_route_total = zenith_route_total + zenith_route_cursor + 2;
        zenith_route_cursor = zenith_route_cursor + 1;
    }
    if zenith_route_total % 2 == 0 {
        zenith_route_total = zenith_route_total + 7;
    } else {
        zenith_route_total = zenith_route_total - 1;
    }
    var zenith_route_left = zenith_route_total + seed;
    var zenith_route_right = zenith_route_left * 3;
    var zenith_route_merged = zenith_route_right - zenith_route_left;
    if zenith_route_merged > 20 {
        zenith_route_total = zenith_route_total + zenith_route_merged;
    }
    return zenith_route_total;
}

flow latent_effect_errors_packet_nimbus_score(seed: i32) -> i32 ![]
{
    var zenith_score_total = seed + 13;
    var zenith_score_cursor = 0;
    while zenith_score_cursor < 8 limit Iterations(8) {
        zenith_score_total = zenith_score_total + zenith_score_cursor + 2;
        zenith_score_cursor = zenith_score_cursor + 1;
    }
    if zenith_score_total % 2 == 0 {
        zenith_score_total = zenith_score_total + 7;
    } else {
        zenith_score_total = zenith_score_total - 1;
    }
    var zenith_score_left = zenith_score_total + seed;
    var zenith_score_right = zenith_score_left * 3;
    var zenith_score_merged = zenith_score_right - zenith_score_left;
    if zenith_score_merged > 20 {
        zenith_score_total = zenith_score_total + zenith_score_merged;
    }
    return zenith_score_total;
}

flow latent_effect_errors_packet_nimbus_finish(seed: i32) -> i32 ![]
{
    var zenith_finish_total = seed - 13;
    var zenith_finish_cursor = 0;
    while zenith_finish_cursor < 10 limit Iterations(10) {
        zenith_finish_total = zenith_finish_total + zenith_finish_cursor + 2;
        zenith_finish_cursor = zenith_finish_cursor + 1;
    }
    if zenith_finish_total % 2 == 0 {
        zenith_finish_total = zenith_finish_total + 7;
    } else {
        zenith_finish_total = zenith_finish_total - 1;
    }
    var zenith_finish_left = zenith_finish_total + seed;
    var zenith_finish_right = zenith_finish_left * 3;
    var zenith_finish_merged = zenith_finish_right - zenith_finish_left;
    if zenith_finish_merged > 20 {
        zenith_finish_total = zenith_finish_total + zenith_finish_merged;
    }
    return zenith_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var nimbus_seed = 2;
    if args.len() > 0 {
        nimbus_seed = nimbus_seed + 1;
    } else {
        nimbus_seed = nimbus_seed + 2;
    }
    let nimbus_result = latent_effect_errors_packet_nimbus_entry(nimbus_seed);
    if nimbus_result > 0 {
        return 0;
    }
    return 1;
}
