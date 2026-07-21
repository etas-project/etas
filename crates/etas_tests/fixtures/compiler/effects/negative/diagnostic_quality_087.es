module tests.compiler.effects.negative.diagnostic_quality_087;

import std.io.{println};

flow diagnostic_quality_schema_packet_entry(seed: i32) -> i32 ![]
{
    var packet_total = diagnostic_quality_schema_packet_prepare(seed);
    packet_total = packet_total + diagnostic_quality_schema_packet_route(seed + 2);
    println("diagnostic quality 0");
    let packet_adjust: i32 -> i32 = (value: i32) => value + 7;
    packet_total = packet_adjust(packet_total);
    packet_total = packet_total + diagnostic_quality_schema_packet_score(4);
    packet_total = packet_total + diagnostic_quality_schema_packet_finish(7);
    if packet_total > 527 {
        packet_total = packet_total - 5;
    } else {
        packet_total = packet_total + 15;
    }
    return packet_total;
}

flow diagnostic_quality_schema_packet_prepare(seed: i32) -> i32 ![]
{
    var north_prepare_total = seed + 15;
    var north_prepare_cursor = 0;
    while north_prepare_cursor < 10 limit Iterations(10) {
        north_prepare_total = north_prepare_total + north_prepare_cursor + 4;
        north_prepare_cursor = north_prepare_cursor + 1;
    }
    if north_prepare_total % 2 == 0 {
        north_prepare_total = north_prepare_total + diagnostic_quality_schema_packet_score(1);
    } else {
        north_prepare_total = north_prepare_total - 3;
    }
    var north_prepare_left = north_prepare_total + seed;
    var north_prepare_right = north_prepare_left * 5;
    var north_prepare_merged = north_prepare_right - north_prepare_left;
    if north_prepare_merged > 22 {
        north_prepare_total = north_prepare_total + north_prepare_merged;
    }
    return north_prepare_total;
}

flow diagnostic_quality_schema_packet_route(seed: i32) -> i32 ![]
{
    var north_route_total = seed * 15;
    var north_route_cursor = 0;
    while north_route_cursor < 8 limit Iterations(8) {
        north_route_total = north_route_total + north_route_cursor + 4;
        north_route_cursor = north_route_cursor + 1;
    }
    if north_route_total % 2 == 0 {
        north_route_total = north_route_total + 9;
    } else {
        north_route_total = north_route_total - 3;
    }
    var north_route_left = north_route_total + seed;
    var north_route_right = north_route_left * 5;
    var north_route_merged = north_route_right - north_route_left;
    if north_route_merged > 22 {
        north_route_total = north_route_total + north_route_merged;
    }
    return north_route_total;
}

flow diagnostic_quality_schema_packet_score(seed: i32) -> i32 ![]
{
    var north_score_total = seed + 15;
    var north_score_cursor = 0;
    while north_score_cursor < 10 limit Iterations(10) {
        north_score_total = north_score_total + north_score_cursor + 4;
        north_score_cursor = north_score_cursor + 1;
    }
    if north_score_total % 2 == 0 {
        north_score_total = north_score_total + 9;
    } else {
        north_score_total = north_score_total - 3;
    }
    var north_score_left = north_score_total + seed;
    var north_score_right = north_score_left * 5;
    var north_score_merged = north_score_right - north_score_left;
    if north_score_merged > 22 {
        north_score_total = north_score_total + north_score_merged;
    }
    return north_score_total;
}

flow diagnostic_quality_schema_packet_finish(seed: i32) -> i32 ![]
{
    var north_finish_total = seed - 15;
    var north_finish_cursor = 0;
    while north_finish_cursor < 12 limit Iterations(12) {
        north_finish_total = north_finish_total + north_finish_cursor + 4;
        north_finish_cursor = north_finish_cursor + 1;
    }
    if north_finish_total % 2 == 0 {
        north_finish_total = north_finish_total + 9;
    } else {
        north_finish_total = north_finish_total - 3;
    }
    var north_finish_left = north_finish_total + seed;
    var north_finish_right = north_finish_left * 5;
    var north_finish_merged = north_finish_right - north_finish_left;
    if north_finish_merged > 22 {
        north_finish_total = north_finish_total + north_finish_merged;
    }
    return north_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var packet_seed = 4;
    if args.len() > 0 {
        packet_seed = packet_seed + 1;
    } else {
        packet_seed = packet_seed + 2;
    }
    let packet_result = diagnostic_quality_schema_packet_entry(packet_seed);
    if packet_result > 0 {
        return 0;
    }
    return 1;
}
