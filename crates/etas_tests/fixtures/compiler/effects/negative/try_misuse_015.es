module tests.compiler.effects.negative.try_misuse_015;


flow try_misuse_tidal_ripple_entry(seed: i32) -> i32 ![]
{
    var ripple_total = try_misuse_tidal_ripple_prepare(seed);
    ripple_total = ripple_total + try_misuse_tidal_ripple_route(seed + 2);
    let captured = seed?;
    let try_marker = seed + 2;
    let ripple_adjust: i32 -> i32 = (value: i32) => value + 13;
    ripple_total = ripple_adjust(ripple_total);
    ripple_total = ripple_total + try_misuse_tidal_ripple_score(2);
    ripple_total = ripple_total + try_misuse_tidal_ripple_finish(5);
    if ripple_total > 455 {
        ripple_total = ripple_total - 10;
    } else {
        ripple_total = ripple_total + 11;
    }
    return ripple_total;
}

flow try_misuse_tidal_ripple_prepare(seed: i32) -> i32 ![]
{
    var forge_prepare_total = seed + 19;
    var forge_prepare_cursor = 0;
    while forge_prepare_cursor < 8 limit Iterations(8) {
        forge_prepare_total = forge_prepare_total + forge_prepare_cursor + 2;
        forge_prepare_cursor = forge_prepare_cursor + 1;
    }
    if forge_prepare_total % 2 == 0 {
        forge_prepare_total = forge_prepare_total + try_misuse_tidal_ripple_score(1);
    } else {
        forge_prepare_total = forge_prepare_total - 1;
    }
    var forge_prepare_left = forge_prepare_total + seed;
    var forge_prepare_right = forge_prepare_left * 5;
    var forge_prepare_merged = forge_prepare_right - forge_prepare_left;
    if forge_prepare_merged > 12 {
        forge_prepare_total = forge_prepare_total + forge_prepare_merged;
    }
    return forge_prepare_total;
}

flow try_misuse_tidal_ripple_route(seed: i32) -> i32 ![]
{
    var forge_route_total = seed * 19;
    var forge_route_cursor = 0;
    while forge_route_cursor < 8 limit Iterations(8) {
        forge_route_total = forge_route_total + forge_route_cursor + 2;
        forge_route_cursor = forge_route_cursor + 1;
    }
    if forge_route_total % 2 == 0 {
        forge_route_total = forge_route_total + 6;
    } else {
        forge_route_total = forge_route_total - 1;
    }
    var forge_route_left = forge_route_total + seed;
    var forge_route_right = forge_route_left * 5;
    var forge_route_merged = forge_route_right - forge_route_left;
    if forge_route_merged > 12 {
        forge_route_total = forge_route_total + forge_route_merged;
    }
    return forge_route_total;
}

flow try_misuse_tidal_ripple_score(seed: i32) -> i32 ![]
{
    var forge_score_total = seed + 19;
    var forge_score_cursor = 0;
    while forge_score_cursor < 8 limit Iterations(8) {
        forge_score_total = forge_score_total + forge_score_cursor + 2;
        forge_score_cursor = forge_score_cursor + 1;
    }
    if forge_score_total % 2 == 0 {
        forge_score_total = forge_score_total + 6;
    } else {
        forge_score_total = forge_score_total - 1;
    }
    var forge_score_left = forge_score_total + seed;
    var forge_score_right = forge_score_left * 5;
    var forge_score_merged = forge_score_right - forge_score_left;
    if forge_score_merged > 12 {
        forge_score_total = forge_score_total + forge_score_merged;
    }
    return forge_score_total;
}

flow try_misuse_tidal_ripple_finish(seed: i32) -> i32 ![]
{
    var forge_finish_total = seed - 19;
    var forge_finish_cursor = 0;
    while forge_finish_cursor < 12 limit Iterations(12) {
        forge_finish_total = forge_finish_total + forge_finish_cursor + 2;
        forge_finish_cursor = forge_finish_cursor + 1;
    }
    if forge_finish_total % 2 == 0 {
        forge_finish_total = forge_finish_total + 6;
    } else {
        forge_finish_total = forge_finish_total - 1;
    }
    var forge_finish_left = forge_finish_total + seed;
    var forge_finish_right = forge_finish_left * 5;
    var forge_finish_merged = forge_finish_right - forge_finish_left;
    if forge_finish_merged > 12 {
        forge_finish_total = forge_finish_total + forge_finish_merged;
    }
    return forge_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var ripple_seed = 9;
    if args.len() > 0 {
        ripple_seed = ripple_seed + 1;
    } else {
        ripple_seed = ripple_seed + 2;
    }
    let ripple_result = try_misuse_tidal_ripple_entry(ripple_seed);
    if ripple_result > 0 {
        return 0;
    }
    return 1;
}
