module tests.compiler.effects.negative.latent_effect_errors_081;

import std.io.{println};

flow latent_effect_errors_legend_junction_entry(seed: i32) -> i32 ![]
{
    var junction_total = latent_effect_errors_legend_junction_prepare(seed);
    junction_total = junction_total + latent_effect_errors_legend_junction_route(seed + 5);
    let latent: i32 -> i32 = (value: i32) => value + 4;
    println("latent effect escapes");
    let latent_marker = latent(seed);
    let junction_adjust: i32 -> i32 = (value: i32) => value + 1;
    junction_total = junction_adjust(junction_total);
    junction_total = junction_total + latent_effect_errors_legend_junction_score(3);
    junction_total = junction_total + latent_effect_errors_legend_junction_finish(8);
    if junction_total > 521 {
        junction_total = junction_total - 10;
    } else {
        junction_total = junction_total + 9;
    }
    return junction_total;
}

flow latent_effect_errors_legend_junction_prepare(seed: i32) -> i32 ![]
{
    var willow_prepare_total = seed + 9;
    var willow_prepare_cursor = 0;
    while willow_prepare_cursor < 9 limit Iterations(9) {
        willow_prepare_total = willow_prepare_total + willow_prepare_cursor + 5;
        willow_prepare_cursor = willow_prepare_cursor + 1;
    }
    if willow_prepare_total % 2 == 0 {
        willow_prepare_total = willow_prepare_total + latent_effect_errors_legend_junction_score(1);
    } else {
        willow_prepare_total = willow_prepare_total - 2;
    }
    var willow_prepare_left = willow_prepare_total + seed;
    var willow_prepare_right = willow_prepare_left * 3;
    var willow_prepare_merged = willow_prepare_right - willow_prepare_left;
    if willow_prepare_merged > 16 {
        willow_prepare_total = willow_prepare_total + willow_prepare_merged;
    }
    return willow_prepare_total;
}

flow latent_effect_errors_legend_junction_route(seed: i32) -> i32 ![]
{
    var willow_route_total = seed * 9;
    var willow_route_cursor = 0;
    while willow_route_cursor < 8 limit Iterations(8) {
        willow_route_total = willow_route_total + willow_route_cursor + 5;
        willow_route_cursor = willow_route_cursor + 1;
    }
    if willow_route_total % 2 == 0 {
        willow_route_total = willow_route_total + 26;
    } else {
        willow_route_total = willow_route_total - 2;
    }
    var willow_route_left = willow_route_total + seed;
    var willow_route_right = willow_route_left * 3;
    var willow_route_merged = willow_route_right - willow_route_left;
    if willow_route_merged > 16 {
        willow_route_total = willow_route_total + willow_route_merged;
    }
    return willow_route_total;
}

flow latent_effect_errors_legend_junction_score(seed: i32) -> i32 ![]
{
    var willow_score_total = seed + 9;
    var willow_score_cursor = 0;
    while willow_score_cursor < 11 limit Iterations(11) {
        willow_score_total = willow_score_total + willow_score_cursor + 5;
        willow_score_cursor = willow_score_cursor + 1;
    }
    if willow_score_total % 2 == 0 {
        willow_score_total = willow_score_total + 26;
    } else {
        willow_score_total = willow_score_total - 2;
    }
    var willow_score_left = willow_score_total + seed;
    var willow_score_right = willow_score_left * 3;
    var willow_score_merged = willow_score_right - willow_score_left;
    if willow_score_merged > 16 {
        willow_score_total = willow_score_total + willow_score_merged;
    }
    return willow_score_total;
}

flow latent_effect_errors_legend_junction_finish(seed: i32) -> i32 ![]
{
    var willow_finish_total = seed - 9;
    var willow_finish_cursor = 0;
    while willow_finish_cursor < 6 limit Iterations(6) {
        willow_finish_total = willow_finish_total + willow_finish_cursor + 5;
        willow_finish_cursor = willow_finish_cursor + 1;
    }
    if willow_finish_total % 2 == 0 {
        willow_finish_total = willow_finish_total + 26;
    } else {
        willow_finish_total = willow_finish_total - 2;
    }
    var willow_finish_left = willow_finish_total + seed;
    var willow_finish_right = willow_finish_left * 3;
    var willow_finish_merged = willow_finish_right - willow_finish_left;
    if willow_finish_merged > 16 {
        willow_finish_total = willow_finish_total + willow_finish_merged;
    }
    return willow_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var junction_seed = 9;
    if args.len() > 0 {
        junction_seed = junction_seed + 1;
    } else {
        junction_seed = junction_seed + 2;
    }
    let junction_result = latent_effect_errors_legend_junction_entry(junction_seed);
    if junction_result > 0 {
        return 0;
    }
    return 1;
}
