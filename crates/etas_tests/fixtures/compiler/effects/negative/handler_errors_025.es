module tests.compiler.effects.negative.handler_errors_025;


flow handler_errors_equinox_cipher_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var cipher_total = handler_errors_equinox_cipher_prepare(seed);
    cipher_total = cipher_total + handler_errors_equinox_cipher_route(seed + 3);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            resume "fallback";
        }
    };
    let cipher_adjust: i32 -> i32 = (value: i32) => value + 10;
    cipher_total = cipher_adjust(cipher_total);
    cipher_total = cipher_total + handler_errors_equinox_cipher_score(2);
    cipher_total = cipher_total + handler_errors_equinox_cipher_finish(8);
    if cipher_total > 465 {
        cipher_total = cipher_total - 9;
    } else {
        cipher_total = cipher_total + 4;
    }
    return cipher_total;
}

flow handler_errors_equinox_cipher_prepare(seed: i32) -> i32 ![]
{
    var charlie_prepare_total = seed + 10;
    var charlie_prepare_cursor = 0;
    while charlie_prepare_cursor < 8 limit Iterations(8) {
        charlie_prepare_total = charlie_prepare_total + charlie_prepare_cursor + 5;
        charlie_prepare_cursor = charlie_prepare_cursor + 1;
    }
    if charlie_prepare_total % 2 == 0 {
        charlie_prepare_total = charlie_prepare_total + handler_errors_equinox_cipher_score(1);
    } else {
        charlie_prepare_total = charlie_prepare_total - 1;
    }
    var charlie_prepare_left = charlie_prepare_total + seed;
    var charlie_prepare_right = charlie_prepare_left * 3;
    var charlie_prepare_merged = charlie_prepare_right - charlie_prepare_left;
    if charlie_prepare_merged > 22 {
        charlie_prepare_total = charlie_prepare_total + charlie_prepare_merged;
    }
    return charlie_prepare_total;
}

flow handler_errors_equinox_cipher_route(seed: i32) -> i32 ![]
{
    var charlie_route_total = seed * 10;
    var charlie_route_cursor = 0;
    while charlie_route_cursor < 12 limit Iterations(12) {
        charlie_route_total = charlie_route_total + charlie_route_cursor + 5;
        charlie_route_cursor = charlie_route_cursor + 1;
    }
    if charlie_route_total % 2 == 0 {
        charlie_route_total = charlie_route_total + 16;
    } else {
        charlie_route_total = charlie_route_total - 1;
    }
    var charlie_route_left = charlie_route_total + seed;
    var charlie_route_right = charlie_route_left * 3;
    var charlie_route_merged = charlie_route_right - charlie_route_left;
    if charlie_route_merged > 22 {
        charlie_route_total = charlie_route_total + charlie_route_merged;
    }
    return charlie_route_total;
}

flow handler_errors_equinox_cipher_score(seed: i32) -> i32 ![]
{
    var charlie_score_total = seed + 10;
    var charlie_score_cursor = 0;
    while charlie_score_cursor < 11 limit Iterations(11) {
        charlie_score_total = charlie_score_total + charlie_score_cursor + 5;
        charlie_score_cursor = charlie_score_cursor + 1;
    }
    if charlie_score_total % 2 == 0 {
        charlie_score_total = charlie_score_total + 16;
    } else {
        charlie_score_total = charlie_score_total - 1;
    }
    var charlie_score_left = charlie_score_total + seed;
    var charlie_score_right = charlie_score_left * 3;
    var charlie_score_merged = charlie_score_right - charlie_score_left;
    if charlie_score_merged > 22 {
        charlie_score_total = charlie_score_total + charlie_score_merged;
    }
    return charlie_score_total;
}

flow handler_errors_equinox_cipher_finish(seed: i32) -> i32 ![]
{
    var charlie_finish_total = seed - 10;
    var charlie_finish_cursor = 0;
    while charlie_finish_cursor < 6 limit Iterations(6) {
        charlie_finish_total = charlie_finish_total + charlie_finish_cursor + 5;
        charlie_finish_cursor = charlie_finish_cursor + 1;
    }
    if charlie_finish_total % 2 == 0 {
        charlie_finish_total = charlie_finish_total + 16;
    } else {
        charlie_finish_total = charlie_finish_total - 1;
    }
    var charlie_finish_left = charlie_finish_total + seed;
    var charlie_finish_right = charlie_finish_left * 3;
    var charlie_finish_merged = charlie_finish_right - charlie_finish_left;
    if charlie_finish_merged > 22 {
        charlie_finish_total = charlie_finish_total + charlie_finish_merged;
    }
    return charlie_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var cipher_seed = 8;
    if args.len() > 0 {
        cipher_seed = cipher_seed + 1;
    } else {
        cipher_seed = cipher_seed + 2;
    }
    let cipher_result = handler_errors_equinox_cipher_entry(cipher_seed);
    if cipher_result > 0 {
        return 0;
    }
    return 1;
}
