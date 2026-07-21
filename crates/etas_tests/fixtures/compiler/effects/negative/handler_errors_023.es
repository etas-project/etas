module tests.compiler.effects.negative.handler_errors_023;


flow handler_errors_cipher_atlas_entry(seed: i32) -> i32 ![Error<IndexError>]
{
    var atlas_total = handler_errors_cipher_atlas_prepare(seed);
    atlas_total = atlas_total + handler_errors_cipher_atlas_route(seed + 1);
    let local_values = ["left", "right", "center"];
    let handled = handle {
        local_values[0]
    } with {
        Error<IndexError>.raise(err) => {
            resume "fallback";
        }
    };
    let atlas_adjust: i32 -> i32 = (value: i32) => value + 8;
    atlas_total = atlas_adjust(atlas_total);
    atlas_total = atlas_total + handler_errors_cipher_atlas_score(5);
    atlas_total = atlas_total + handler_errors_cipher_atlas_finish(6);
    if atlas_total > 463 {
        atlas_total = atlas_total - 7;
    } else {
        atlas_total = atlas_total + 19;
    }
    return atlas_total;
}

flow handler_errors_cipher_atlas_prepare(seed: i32) -> i32 ![]
{
    var monsoon_prepare_total = seed + 8;
    var monsoon_prepare_cursor = 0;
    while monsoon_prepare_cursor < 11 limit Iterations(11) {
        monsoon_prepare_total = monsoon_prepare_total + monsoon_prepare_cursor + 3;
        monsoon_prepare_cursor = monsoon_prepare_cursor + 1;
    }
    if monsoon_prepare_total % 2 == 0 {
        monsoon_prepare_total = monsoon_prepare_total + handler_errors_cipher_atlas_score(1);
    } else {
        monsoon_prepare_total = monsoon_prepare_total - 4;
    }
    var monsoon_prepare_left = monsoon_prepare_total + seed;
    var monsoon_prepare_right = monsoon_prepare_left * 5;
    var monsoon_prepare_merged = monsoon_prepare_right - monsoon_prepare_left;
    if monsoon_prepare_merged > 20 {
        monsoon_prepare_total = monsoon_prepare_total + monsoon_prepare_merged;
    }
    return monsoon_prepare_total;
}

flow handler_errors_cipher_atlas_route(seed: i32) -> i32 ![]
{
    var monsoon_route_total = seed * 8;
    var monsoon_route_cursor = 0;
    while monsoon_route_cursor < 10 limit Iterations(10) {
        monsoon_route_total = monsoon_route_total + monsoon_route_cursor + 3;
        monsoon_route_cursor = monsoon_route_cursor + 1;
    }
    if monsoon_route_total % 2 == 0 {
        monsoon_route_total = monsoon_route_total + 14;
    } else {
        monsoon_route_total = monsoon_route_total - 4;
    }
    var monsoon_route_left = monsoon_route_total + seed;
    var monsoon_route_right = monsoon_route_left * 5;
    var monsoon_route_merged = monsoon_route_right - monsoon_route_left;
    if monsoon_route_merged > 20 {
        monsoon_route_total = monsoon_route_total + monsoon_route_merged;
    }
    return monsoon_route_total;
}

flow handler_errors_cipher_atlas_score(seed: i32) -> i32 ![]
{
    var monsoon_score_total = seed + 8;
    var monsoon_score_cursor = 0;
    while monsoon_score_cursor < 9 limit Iterations(9) {
        monsoon_score_total = monsoon_score_total + monsoon_score_cursor + 3;
        monsoon_score_cursor = monsoon_score_cursor + 1;
    }
    if monsoon_score_total % 2 == 0 {
        monsoon_score_total = monsoon_score_total + 14;
    } else {
        monsoon_score_total = monsoon_score_total - 4;
    }
    var monsoon_score_left = monsoon_score_total + seed;
    var monsoon_score_right = monsoon_score_left * 5;
    var monsoon_score_merged = monsoon_score_right - monsoon_score_left;
    if monsoon_score_merged > 20 {
        monsoon_score_total = monsoon_score_total + monsoon_score_merged;
    }
    return monsoon_score_total;
}

flow handler_errors_cipher_atlas_finish(seed: i32) -> i32 ![]
{
    var monsoon_finish_total = seed - 8;
    var monsoon_finish_cursor = 0;
    while monsoon_finish_cursor < 12 limit Iterations(12) {
        monsoon_finish_total = monsoon_finish_total + monsoon_finish_cursor + 3;
        monsoon_finish_cursor = monsoon_finish_cursor + 1;
    }
    if monsoon_finish_total % 2 == 0 {
        monsoon_finish_total = monsoon_finish_total + 14;
    } else {
        monsoon_finish_total = monsoon_finish_total - 4;
    }
    var monsoon_finish_left = monsoon_finish_total + seed;
    var monsoon_finish_right = monsoon_finish_left * 5;
    var monsoon_finish_merged = monsoon_finish_right - monsoon_finish_left;
    if monsoon_finish_merged > 20 {
        monsoon_finish_total = monsoon_finish_total + monsoon_finish_merged;
    }
    return monsoon_finish_total;
}

flow main(args: Array<string>) -> i32 ![Error<IndexError>]
{
    var atlas_seed = 6;
    if args.len() > 0 {
        atlas_seed = atlas_seed + 1;
    } else {
        atlas_seed = atlas_seed + 2;
    }
    let atlas_result = handler_errors_cipher_atlas_entry(atlas_seed);
    if atlas_result > 0 {
        return 0;
    }
    return 1;
}
