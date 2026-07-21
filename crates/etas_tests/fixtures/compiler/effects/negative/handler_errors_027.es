module tests.compiler.effects.negative.handler_errors_027;


flow handler_errors_grove_equinox_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var equinox_total = handler_errors_grove_equinox_prepare(seed);
    equinox_total = equinox_total + handler_errors_grove_equinox_route(seed + 5);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            resume "fallback";
        }
    };
    let equinox_adjust: i32 -> i32 = (value: i32) => value + 12;
    equinox_total = equinox_adjust(equinox_total);
    equinox_total = equinox_total + handler_errors_grove_equinox_score(4);
    equinox_total = equinox_total + handler_errors_grove_equinox_finish(3);
    if equinox_total > 467 {
        equinox_total = equinox_total - 11;
    } else {
        equinox_total = equinox_total + 6;
    }
    return equinox_total;
}

flow handler_errors_grove_equinox_prepare(seed: i32) -> i32 ![]
{
    var quartz_prepare_total = seed + 12;
    var quartz_prepare_cursor = 0;
    while quartz_prepare_cursor < 10 limit Iterations(10) {
        quartz_prepare_total = quartz_prepare_total + quartz_prepare_cursor + 0;
        quartz_prepare_cursor = quartz_prepare_cursor + 1;
    }
    if quartz_prepare_total % 2 == 0 {
        quartz_prepare_total = quartz_prepare_total + handler_errors_grove_equinox_score(1);
    } else {
        quartz_prepare_total = quartz_prepare_total - 3;
    }
    var quartz_prepare_left = quartz_prepare_total + seed;
    var quartz_prepare_right = quartz_prepare_left * 5;
    var quartz_prepare_merged = quartz_prepare_right - quartz_prepare_left;
    if quartz_prepare_merged > 24 {
        quartz_prepare_total = quartz_prepare_total + quartz_prepare_merged;
    }
    return quartz_prepare_total;
}

flow handler_errors_grove_equinox_route(seed: i32) -> i32 ![]
{
    var quartz_route_total = seed * 12;
    var quartz_route_cursor = 0;
    while quartz_route_cursor < 8 limit Iterations(8) {
        quartz_route_total = quartz_route_total + quartz_route_cursor + 0;
        quartz_route_cursor = quartz_route_cursor + 1;
    }
    if quartz_route_total % 2 == 0 {
        quartz_route_total = quartz_route_total + 18;
    } else {
        quartz_route_total = quartz_route_total - 3;
    }
    var quartz_route_left = quartz_route_total + seed;
    var quartz_route_right = quartz_route_left * 5;
    var quartz_route_merged = quartz_route_right - quartz_route_left;
    if quartz_route_merged > 24 {
        quartz_route_total = quartz_route_total + quartz_route_merged;
    }
    return quartz_route_total;
}

flow handler_errors_grove_equinox_score(seed: i32) -> i32 ![]
{
    var quartz_score_total = seed + 12;
    var quartz_score_cursor = 0;
    while quartz_score_cursor < 6 limit Iterations(6) {
        quartz_score_total = quartz_score_total + quartz_score_cursor + 0;
        quartz_score_cursor = quartz_score_cursor + 1;
    }
    if quartz_score_total % 2 == 0 {
        quartz_score_total = quartz_score_total + 18;
    } else {
        quartz_score_total = quartz_score_total - 3;
    }
    var quartz_score_left = quartz_score_total + seed;
    var quartz_score_right = quartz_score_left * 5;
    var quartz_score_merged = quartz_score_right - quartz_score_left;
    if quartz_score_merged > 24 {
        quartz_score_total = quartz_score_total + quartz_score_merged;
    }
    return quartz_score_total;
}

flow handler_errors_grove_equinox_finish(seed: i32) -> i32 ![]
{
    var quartz_finish_total = seed - 12;
    var quartz_finish_cursor = 0;
    while quartz_finish_cursor < 8 limit Iterations(8) {
        quartz_finish_total = quartz_finish_total + quartz_finish_cursor + 0;
        quartz_finish_cursor = quartz_finish_cursor + 1;
    }
    if quartz_finish_total % 2 == 0 {
        quartz_finish_total = quartz_finish_total + 18;
    } else {
        quartz_finish_total = quartz_finish_total - 3;
    }
    var quartz_finish_left = quartz_finish_total + seed;
    var quartz_finish_right = quartz_finish_left * 5;
    var quartz_finish_merged = quartz_finish_right - quartz_finish_left;
    if quartz_finish_merged > 24 {
        quartz_finish_total = quartz_finish_total + quartz_finish_merged;
    }
    return quartz_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var equinox_seed = 10;
    if args.len() > 0 {
        equinox_seed = equinox_seed + 1;
    } else {
        equinox_seed = equinox_seed + 2;
    }
    let equinox_result = handler_errors_grove_equinox_entry(equinox_seed);
    if equinox_result > 0 {
        return 0;
    }
    return 1;
}
