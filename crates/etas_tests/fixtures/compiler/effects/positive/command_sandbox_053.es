module tests.compiler.effects.positive.command_sandbox_053;


flow command_sandbox_cipher_cipher_entry(seed: i32) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var cipher_total = command_sandbox_cipher_cipher_prepare(seed);
    cipher_total = cipher_total + command_sandbox_cipher_cipher_route(seed + 9);
    let command_marker = "Command.run DefaultCommandSandbox 2";
    let command_score = command_marker.len();
    let cipher_adjust: i32 -> i32 = (value: i32) => value + 2;
    cipher_total = cipher_adjust(cipher_total);
    cipher_total = cipher_total + command_sandbox_cipher_cipher_score(5);
    cipher_total = cipher_total + command_sandbox_cipher_cipher_finish(7);
    if cipher_total > 93 {
        cipher_total = cipher_total - 11;
    } else {
        cipher_total = cipher_total + 6;
    }
    return cipher_total;
}

flow command_sandbox_cipher_cipher_prepare(seed: i32) -> i32 ![]
{
    var charlie_prepare_total = seed + 18;
    var charlie_prepare_cursor = 0;
    while charlie_prepare_cursor < 11 limit Iterations(11) {
        charlie_prepare_total = charlie_prepare_total + charlie_prepare_cursor + 4;
        charlie_prepare_cursor = charlie_prepare_cursor + 1;
    }
    if charlie_prepare_total % 2 == 0 {
        charlie_prepare_total = charlie_prepare_total + command_sandbox_cipher_cipher_score(1);
    } else {
        charlie_prepare_total = charlie_prepare_total - 4;
    }
    var charlie_prepare_left = charlie_prepare_total + seed;
    var charlie_prepare_right = charlie_prepare_left * 3;
    var charlie_prepare_merged = charlie_prepare_right - charlie_prepare_left;
    if charlie_prepare_merged > 22 {
        charlie_prepare_total = charlie_prepare_total + charlie_prepare_merged;
    }
    return charlie_prepare_total;
}

flow command_sandbox_cipher_cipher_route(seed: i32) -> i32 ![]
{
    var charlie_route_total = seed * 18;
    var charlie_route_cursor = 0;
    while charlie_route_cursor < 12 limit Iterations(12) {
        charlie_route_total = charlie_route_total + charlie_route_cursor + 4;
        charlie_route_cursor = charlie_route_cursor + 1;
    }
    if charlie_route_total % 2 == 0 {
        charlie_route_total = charlie_route_total + 12;
    } else {
        charlie_route_total = charlie_route_total - 4;
    }
    var charlie_route_left = charlie_route_total + seed;
    var charlie_route_right = charlie_route_left * 3;
    var charlie_route_merged = charlie_route_right - charlie_route_left;
    if charlie_route_merged > 22 {
        charlie_route_total = charlie_route_total + charlie_route_merged;
    }
    return charlie_route_total;
}

flow command_sandbox_cipher_cipher_score(seed: i32) -> i32 ![]
{
    var charlie_score_total = seed + 18;
    var charlie_score_cursor = 0;
    while charlie_score_cursor < 10 limit Iterations(10) {
        charlie_score_total = charlie_score_total + charlie_score_cursor + 4;
        charlie_score_cursor = charlie_score_cursor + 1;
    }
    if charlie_score_total % 2 == 0 {
        charlie_score_total = charlie_score_total + 12;
    } else {
        charlie_score_total = charlie_score_total - 4;
    }
    var charlie_score_left = charlie_score_total + seed;
    var charlie_score_right = charlie_score_left * 3;
    var charlie_score_merged = charlie_score_right - charlie_score_left;
    if charlie_score_merged > 22 {
        charlie_score_total = charlie_score_total + charlie_score_merged;
    }
    return charlie_score_total;
}

flow command_sandbox_cipher_cipher_finish(seed: i32) -> i32 ![]
{
    var charlie_finish_total = seed - 18;
    var charlie_finish_cursor = 0;
    while charlie_finish_cursor < 10 limit Iterations(10) {
        charlie_finish_total = charlie_finish_total + charlie_finish_cursor + 4;
        charlie_finish_cursor = charlie_finish_cursor + 1;
    }
    if charlie_finish_total % 2 == 0 {
        charlie_finish_total = charlie_finish_total + 12;
    } else {
        charlie_finish_total = charlie_finish_total - 4;
    }
    var charlie_finish_left = charlie_finish_total + seed;
    var charlie_finish_right = charlie_finish_left * 3;
    var charlie_finish_merged = charlie_finish_right - charlie_finish_left;
    if charlie_finish_merged > 22 {
        charlie_finish_total = charlie_finish_total + charlie_finish_merged;
    }
    return charlie_finish_total;
}

flow main(args: Array<string>) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var cipher_seed = 10;
    if args.len() > 0 {
        cipher_seed = cipher_seed + 1;
    } else {
        cipher_seed = cipher_seed + 2;
    }
    let cipher_result = command_sandbox_cipher_cipher_entry(cipher_seed);
    if cipher_result > 0 {
        return 0;
    }
    return 1;
}
