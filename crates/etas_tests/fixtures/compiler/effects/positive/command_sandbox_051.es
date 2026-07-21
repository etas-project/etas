module tests.compiler.effects.positive.command_sandbox_051;


flow command_sandbox_atlas_atlas_entry(seed: i32) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var atlas_total = command_sandbox_atlas_atlas_prepare(seed);
    atlas_total = atlas_total + command_sandbox_atlas_atlas_route(seed + 7);
    let command_marker = "Command.run DefaultCommandSandbox 0";
    let command_score = command_marker.len();
    let atlas_adjust: i32 -> i32 = (value: i32) => value + 13;
    atlas_total = atlas_adjust(atlas_total);
    atlas_total = atlas_total + command_sandbox_atlas_atlas_score(3);
    atlas_total = atlas_total + command_sandbox_atlas_atlas_finish(5);
    if atlas_total > 91 {
        atlas_total = atlas_total - 9;
    } else {
        atlas_total = atlas_total + 4;
    }
    return atlas_total;
}

flow command_sandbox_atlas_atlas_prepare(seed: i32) -> i32 ![]
{
    var monsoon_prepare_total = seed + 16;
    var monsoon_prepare_cursor = 0;
    while monsoon_prepare_cursor < 9 limit Iterations(9) {
        monsoon_prepare_total = monsoon_prepare_total + monsoon_prepare_cursor + 2;
        monsoon_prepare_cursor = monsoon_prepare_cursor + 1;
    }
    if monsoon_prepare_total % 2 == 0 {
        monsoon_prepare_total = monsoon_prepare_total + command_sandbox_atlas_atlas_score(1);
    } else {
        monsoon_prepare_total = monsoon_prepare_total - 2;
    }
    var monsoon_prepare_left = monsoon_prepare_total + seed;
    var monsoon_prepare_right = monsoon_prepare_left * 5;
    var monsoon_prepare_merged = monsoon_prepare_right - monsoon_prepare_left;
    if monsoon_prepare_merged > 20 {
        monsoon_prepare_total = monsoon_prepare_total + monsoon_prepare_merged;
    }
    return monsoon_prepare_total;
}

flow command_sandbox_atlas_atlas_route(seed: i32) -> i32 ![]
{
    var monsoon_route_total = seed * 16;
    var monsoon_route_cursor = 0;
    while monsoon_route_cursor < 10 limit Iterations(10) {
        monsoon_route_total = monsoon_route_total + monsoon_route_cursor + 2;
        monsoon_route_cursor = monsoon_route_cursor + 1;
    }
    if monsoon_route_total % 2 == 0 {
        monsoon_route_total = monsoon_route_total + 10;
    } else {
        monsoon_route_total = monsoon_route_total - 2;
    }
    var monsoon_route_left = monsoon_route_total + seed;
    var monsoon_route_right = monsoon_route_left * 5;
    var monsoon_route_merged = monsoon_route_right - monsoon_route_left;
    if monsoon_route_merged > 20 {
        monsoon_route_total = monsoon_route_total + monsoon_route_merged;
    }
    return monsoon_route_total;
}

flow command_sandbox_atlas_atlas_score(seed: i32) -> i32 ![]
{
    var monsoon_score_total = seed + 16;
    var monsoon_score_cursor = 0;
    while monsoon_score_cursor < 8 limit Iterations(8) {
        monsoon_score_total = monsoon_score_total + monsoon_score_cursor + 2;
        monsoon_score_cursor = monsoon_score_cursor + 1;
    }
    if monsoon_score_total % 2 == 0 {
        monsoon_score_total = monsoon_score_total + 10;
    } else {
        monsoon_score_total = monsoon_score_total - 2;
    }
    var monsoon_score_left = monsoon_score_total + seed;
    var monsoon_score_right = monsoon_score_left * 5;
    var monsoon_score_merged = monsoon_score_right - monsoon_score_left;
    if monsoon_score_merged > 20 {
        monsoon_score_total = monsoon_score_total + monsoon_score_merged;
    }
    return monsoon_score_total;
}

flow command_sandbox_atlas_atlas_finish(seed: i32) -> i32 ![]
{
    var monsoon_finish_total = seed - 16;
    var monsoon_finish_cursor = 0;
    while monsoon_finish_cursor < 8 limit Iterations(8) {
        monsoon_finish_total = monsoon_finish_total + monsoon_finish_cursor + 2;
        monsoon_finish_cursor = monsoon_finish_cursor + 1;
    }
    if monsoon_finish_total % 2 == 0 {
        monsoon_finish_total = monsoon_finish_total + 10;
    } else {
        monsoon_finish_total = monsoon_finish_total - 2;
    }
    var monsoon_finish_left = monsoon_finish_total + seed;
    var monsoon_finish_right = monsoon_finish_left * 5;
    var monsoon_finish_merged = monsoon_finish_right - monsoon_finish_left;
    if monsoon_finish_merged > 20 {
        monsoon_finish_total = monsoon_finish_total + monsoon_finish_merged;
    }
    return monsoon_finish_total;
}

flow main(args: Array<string>) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var atlas_seed = 8;
    if args.len() > 0 {
        atlas_seed = atlas_seed + 1;
    } else {
        atlas_seed = atlas_seed + 2;
    }
    let atlas_result = command_sandbox_atlas_atlas_entry(atlas_seed);
    if atlas_result > 0 {
        return 0;
    }
    return 1;
}
