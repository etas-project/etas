module tests.compiler.effects.negative.try_misuse_021;


flow try_misuse_atlas_yonder_entry(seed: i32) -> i32 ![]
{
    var yonder_total = try_misuse_atlas_yonder_prepare(seed);
    yonder_total = yonder_total + try_misuse_atlas_yonder_route(seed + 8);
    let captured = seed?;
    let try_marker = seed + 8;
    let yonder_adjust: i32 -> i32 = (value: i32) => value + 6;
    yonder_total = yonder_adjust(yonder_total);
    yonder_total = yonder_total + try_misuse_atlas_yonder_score(3);
    yonder_total = yonder_total + try_misuse_atlas_yonder_finish(4);
    if yonder_total > 461 {
        yonder_total = yonder_total - 5;
    } else {
        yonder_total = yonder_total + 17;
    }
    return yonder_total;
}

flow try_misuse_atlas_yonder_prepare(seed: i32) -> i32 ![]
{
    var yard_prepare_total = seed + 6;
    var yard_prepare_cursor = 0;
    while yard_prepare_cursor < 9 limit Iterations(9) {
        yard_prepare_total = yard_prepare_total + yard_prepare_cursor + 1;
        yard_prepare_cursor = yard_prepare_cursor + 1;
    }
    if yard_prepare_total % 2 == 0 {
        yard_prepare_total = yard_prepare_total + try_misuse_atlas_yonder_score(1);
    } else {
        yard_prepare_total = yard_prepare_total - 2;
    }
    var yard_prepare_left = yard_prepare_total + seed;
    var yard_prepare_right = yard_prepare_left * 3;
    var yard_prepare_merged = yard_prepare_right - yard_prepare_left;
    if yard_prepare_merged > 18 {
        yard_prepare_total = yard_prepare_total + yard_prepare_merged;
    }
    return yard_prepare_total;
}

flow try_misuse_atlas_yonder_route(seed: i32) -> i32 ![]
{
    var yard_route_total = seed * 6;
    var yard_route_cursor = 0;
    while yard_route_cursor < 8 limit Iterations(8) {
        yard_route_total = yard_route_total + yard_route_cursor + 1;
        yard_route_cursor = yard_route_cursor + 1;
    }
    if yard_route_total % 2 == 0 {
        yard_route_total = yard_route_total + 12;
    } else {
        yard_route_total = yard_route_total - 2;
    }
    var yard_route_left = yard_route_total + seed;
    var yard_route_right = yard_route_left * 3;
    var yard_route_merged = yard_route_right - yard_route_left;
    if yard_route_merged > 18 {
        yard_route_total = yard_route_total + yard_route_merged;
    }
    return yard_route_total;
}

flow try_misuse_atlas_yonder_score(seed: i32) -> i32 ![]
{
    var yard_score_total = seed + 6;
    var yard_score_cursor = 0;
    while yard_score_cursor < 7 limit Iterations(7) {
        yard_score_total = yard_score_total + yard_score_cursor + 1;
        yard_score_cursor = yard_score_cursor + 1;
    }
    if yard_score_total % 2 == 0 {
        yard_score_total = yard_score_total + 12;
    } else {
        yard_score_total = yard_score_total - 2;
    }
    var yard_score_left = yard_score_total + seed;
    var yard_score_right = yard_score_left * 3;
    var yard_score_merged = yard_score_right - yard_score_left;
    if yard_score_merged > 18 {
        yard_score_total = yard_score_total + yard_score_merged;
    }
    return yard_score_total;
}

flow try_misuse_atlas_yonder_finish(seed: i32) -> i32 ![]
{
    var yard_finish_total = seed - 6;
    var yard_finish_cursor = 0;
    while yard_finish_cursor < 10 limit Iterations(10) {
        yard_finish_total = yard_finish_total + yard_finish_cursor + 1;
        yard_finish_cursor = yard_finish_cursor + 1;
    }
    if yard_finish_total % 2 == 0 {
        yard_finish_total = yard_finish_total + 12;
    } else {
        yard_finish_total = yard_finish_total - 2;
    }
    var yard_finish_left = yard_finish_total + seed;
    var yard_finish_right = yard_finish_left * 3;
    var yard_finish_merged = yard_finish_right - yard_finish_left;
    if yard_finish_merged > 18 {
        yard_finish_total = yard_finish_total + yard_finish_merged;
    }
    return yard_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var yonder_seed = 4;
    if args.len() > 0 {
        yonder_seed = yonder_seed + 1;
    } else {
        yonder_seed = yonder_seed + 2;
    }
    let yonder_result = try_misuse_atlas_yonder_entry(yonder_seed);
    if yonder_result > 0 {
        return 0;
    }
    return 1;
}
