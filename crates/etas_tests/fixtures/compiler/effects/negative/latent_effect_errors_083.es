module tests.compiler.effects.negative.latent_effect_errors_083;

import std.io.{println};

flow latent_effect_errors_nimbus_legend_entry(seed: i32) -> i32 ![]
{
    var legend_total = latent_effect_errors_nimbus_legend_prepare(seed);
    legend_total = legend_total + latent_effect_errors_nimbus_legend_route(seed + 7);
    let latent: i32 -> i32 = (value: i32) => value + 6;
    println("latent effect escapes");
    let latent_marker = latent(seed);
    let legend_adjust: i32 -> i32 = (value: i32) => value + 3;
    legend_total = legend_adjust(legend_total);
    legend_total = legend_total + latent_effect_errors_nimbus_legend_score(5);
    legend_total = legend_total + latent_effect_errors_nimbus_legend_finish(3);
    if legend_total > 523 {
        legend_total = legend_total - 12;
    } else {
        legend_total = legend_total + 11;
    }
    return legend_total;
}

flow latent_effect_errors_nimbus_legend_prepare(seed: i32) -> i32 ![]
{
    var kepler_prepare_total = seed + 11;
    var kepler_prepare_cursor = 0;
    while kepler_prepare_cursor < 11 limit Iterations(11) {
        kepler_prepare_total = kepler_prepare_total + kepler_prepare_cursor + 0;
        kepler_prepare_cursor = kepler_prepare_cursor + 1;
    }
    if kepler_prepare_total % 2 == 0 {
        kepler_prepare_total = kepler_prepare_total + latent_effect_errors_nimbus_legend_score(1);
    } else {
        kepler_prepare_total = kepler_prepare_total - 4;
    }
    var kepler_prepare_left = kepler_prepare_total + seed;
    var kepler_prepare_right = kepler_prepare_left * 5;
    var kepler_prepare_merged = kepler_prepare_right - kepler_prepare_left;
    if kepler_prepare_merged > 18 {
        kepler_prepare_total = kepler_prepare_total + kepler_prepare_merged;
    }
    return kepler_prepare_total;
}

flow latent_effect_errors_nimbus_legend_route(seed: i32) -> i32 ![]
{
    var kepler_route_total = seed * 11;
    var kepler_route_cursor = 0;
    while kepler_route_cursor < 10 limit Iterations(10) {
        kepler_route_total = kepler_route_total + kepler_route_cursor + 0;
        kepler_route_cursor = kepler_route_cursor + 1;
    }
    if kepler_route_total % 2 == 0 {
        kepler_route_total = kepler_route_total + 5;
    } else {
        kepler_route_total = kepler_route_total - 4;
    }
    var kepler_route_left = kepler_route_total + seed;
    var kepler_route_right = kepler_route_left * 5;
    var kepler_route_merged = kepler_route_right - kepler_route_left;
    if kepler_route_merged > 18 {
        kepler_route_total = kepler_route_total + kepler_route_merged;
    }
    return kepler_route_total;
}

flow latent_effect_errors_nimbus_legend_score(seed: i32) -> i32 ![]
{
    var kepler_score_total = seed + 11;
    var kepler_score_cursor = 0;
    while kepler_score_cursor < 6 limit Iterations(6) {
        kepler_score_total = kepler_score_total + kepler_score_cursor + 0;
        kepler_score_cursor = kepler_score_cursor + 1;
    }
    if kepler_score_total % 2 == 0 {
        kepler_score_total = kepler_score_total + 5;
    } else {
        kepler_score_total = kepler_score_total - 4;
    }
    var kepler_score_left = kepler_score_total + seed;
    var kepler_score_right = kepler_score_left * 5;
    var kepler_score_merged = kepler_score_right - kepler_score_left;
    if kepler_score_merged > 18 {
        kepler_score_total = kepler_score_total + kepler_score_merged;
    }
    return kepler_score_total;
}

flow latent_effect_errors_nimbus_legend_finish(seed: i32) -> i32 ![]
{
    var kepler_finish_total = seed - 11;
    var kepler_finish_cursor = 0;
    while kepler_finish_cursor < 8 limit Iterations(8) {
        kepler_finish_total = kepler_finish_total + kepler_finish_cursor + 0;
        kepler_finish_cursor = kepler_finish_cursor + 1;
    }
    if kepler_finish_total % 2 == 0 {
        kepler_finish_total = kepler_finish_total + 5;
    } else {
        kepler_finish_total = kepler_finish_total - 4;
    }
    var kepler_finish_left = kepler_finish_total + seed;
    var kepler_finish_right = kepler_finish_left * 5;
    var kepler_finish_merged = kepler_finish_right - kepler_finish_left;
    if kepler_finish_merged > 18 {
        kepler_finish_total = kepler_finish_total + kepler_finish_merged;
    }
    return kepler_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var legend_seed = 11;
    if args.len() > 0 {
        legend_seed = legend_seed + 1;
    } else {
        legend_seed = legend_seed + 2;
    }
    let legend_result = latent_effect_errors_nimbus_legend_entry(legend_seed);
    if legend_result > 0 {
        return 0;
    }
    return 1;
}
