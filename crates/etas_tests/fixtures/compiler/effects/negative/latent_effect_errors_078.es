module tests.compiler.effects.negative.latent_effect_errors_078;

import std.io.{println};

flow latent_effect_errors_isotope_galaxy_entry(seed: i32) -> i32 ![]
{
    var galaxy_total = latent_effect_errors_isotope_galaxy_prepare(seed);
    galaxy_total = galaxy_total + latent_effect_errors_isotope_galaxy_route(seed + 2);
    let latent: i32 -> i32 = (value: i32) => value + 1;
    println("latent effect escapes");
    let latent_marker = latent(seed);
    let galaxy_adjust: i32 -> i32 = (value: i32) => value + 11;
    galaxy_total = galaxy_adjust(galaxy_total);
    galaxy_total = galaxy_total + latent_effect_errors_isotope_galaxy_score(5);
    galaxy_total = galaxy_total + latent_effect_errors_isotope_galaxy_finish(5);
    if galaxy_total > 518 {
        galaxy_total = galaxy_total - 7;
    } else {
        galaxy_total = galaxy_total + 6;
    }
    return galaxy_total;
}

flow latent_effect_errors_isotope_galaxy_prepare(seed: i32) -> i32 ![]
{
    var bravo_prepare_total = seed + 6;
    var bravo_prepare_cursor = 0;
    while bravo_prepare_cursor < 11 limit Iterations(11) {
        bravo_prepare_total = bravo_prepare_total + bravo_prepare_cursor + 2;
        bravo_prepare_cursor = bravo_prepare_cursor + 1;
    }
    if bravo_prepare_total % 2 == 0 {
        bravo_prepare_total = bravo_prepare_total + latent_effect_errors_isotope_galaxy_score(1);
    } else {
        bravo_prepare_total = bravo_prepare_total - 4;
    }
    var bravo_prepare_left = bravo_prepare_total + seed;
    var bravo_prepare_right = bravo_prepare_left * 4;
    var bravo_prepare_merged = bravo_prepare_right - bravo_prepare_left;
    if bravo_prepare_merged > 13 {
        bravo_prepare_total = bravo_prepare_total + bravo_prepare_merged;
    }
    return bravo_prepare_total;
}

flow latent_effect_errors_isotope_galaxy_route(seed: i32) -> i32 ![]
{
    var bravo_route_total = seed * 6;
    var bravo_route_cursor = 0;
    while bravo_route_cursor < 11 limit Iterations(11) {
        bravo_route_total = bravo_route_total + bravo_route_cursor + 2;
        bravo_route_cursor = bravo_route_cursor + 1;
    }
    if bravo_route_total % 2 == 0 {
        bravo_route_total = bravo_route_total + 23;
    } else {
        bravo_route_total = bravo_route_total - 4;
    }
    var bravo_route_left = bravo_route_total + seed;
    var bravo_route_right = bravo_route_left * 4;
    var bravo_route_merged = bravo_route_right - bravo_route_left;
    if bravo_route_merged > 13 {
        bravo_route_total = bravo_route_total + bravo_route_merged;
    }
    return bravo_route_total;
}

flow latent_effect_errors_isotope_galaxy_score(seed: i32) -> i32 ![]
{
    var bravo_score_total = seed + 6;
    var bravo_score_cursor = 0;
    while bravo_score_cursor < 8 limit Iterations(8) {
        bravo_score_total = bravo_score_total + bravo_score_cursor + 2;
        bravo_score_cursor = bravo_score_cursor + 1;
    }
    if bravo_score_total % 2 == 0 {
        bravo_score_total = bravo_score_total + 23;
    } else {
        bravo_score_total = bravo_score_total - 4;
    }
    var bravo_score_left = bravo_score_total + seed;
    var bravo_score_right = bravo_score_left * 4;
    var bravo_score_merged = bravo_score_right - bravo_score_left;
    if bravo_score_merged > 13 {
        bravo_score_total = bravo_score_total + bravo_score_merged;
    }
    return bravo_score_total;
}

flow latent_effect_errors_isotope_galaxy_finish(seed: i32) -> i32 ![]
{
    var bravo_finish_total = seed - 6;
    var bravo_finish_cursor = 0;
    while bravo_finish_cursor < 11 limit Iterations(11) {
        bravo_finish_total = bravo_finish_total + bravo_finish_cursor + 2;
        bravo_finish_cursor = bravo_finish_cursor + 1;
    }
    if bravo_finish_total % 2 == 0 {
        bravo_finish_total = bravo_finish_total + 23;
    } else {
        bravo_finish_total = bravo_finish_total - 4;
    }
    var bravo_finish_left = bravo_finish_total + seed;
    var bravo_finish_right = bravo_finish_left * 4;
    var bravo_finish_merged = bravo_finish_right - bravo_finish_left;
    if bravo_finish_merged > 13 {
        bravo_finish_total = bravo_finish_total + bravo_finish_merged;
    }
    return bravo_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var galaxy_seed = 6;
    if args.len() > 0 {
        galaxy_seed = galaxy_seed + 1;
    } else {
        galaxy_seed = galaxy_seed + 2;
    }
    let galaxy_result = latent_effect_errors_isotope_galaxy_entry(galaxy_seed);
    if galaxy_result > 0 {
        return 0;
    }
    return 1;
}
