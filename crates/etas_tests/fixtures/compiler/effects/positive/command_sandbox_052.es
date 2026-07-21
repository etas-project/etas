module tests.compiler.effects.positive.command_sandbox_052;


flow command_sandbox_binary_binary_entry(seed: i32) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var binary_total = command_sandbox_binary_binary_prepare(seed);
    binary_total = binary_total + command_sandbox_binary_binary_route(seed + 8);
    let command_marker = "Command.run DefaultCommandSandbox 1";
    let command_score = command_marker.len();
    let binary_adjust: i32 -> i32 = (value: i32) => value + 1;
    binary_total = binary_adjust(binary_total);
    binary_total = binary_total + command_sandbox_binary_binary_score(4);
    binary_total = binary_total + command_sandbox_binary_binary_finish(6);
    if binary_total > 92 {
        binary_total = binary_total - 10;
    } else {
        binary_total = binary_total + 5;
    }
    return binary_total;
}

flow command_sandbox_binary_binary_prepare(seed: i32) -> i32 ![]
{
    var union_prepare_total = seed + 17;
    var union_prepare_cursor = 0;
    while union_prepare_cursor < 10 limit Iterations(10) {
        union_prepare_total = union_prepare_total + union_prepare_cursor + 3;
        union_prepare_cursor = union_prepare_cursor + 1;
    }
    if union_prepare_total % 2 == 0 {
        union_prepare_total = union_prepare_total + command_sandbox_binary_binary_score(1);
    } else {
        union_prepare_total = union_prepare_total - 3;
    }
    var union_prepare_left = union_prepare_total + seed;
    var union_prepare_right = union_prepare_left * 2;
    var union_prepare_merged = union_prepare_right - union_prepare_left;
    if union_prepare_merged > 21 {
        union_prepare_total = union_prepare_total + union_prepare_merged;
    }
    return union_prepare_total;
}

flow command_sandbox_binary_binary_route(seed: i32) -> i32 ![]
{
    var union_route_total = seed * 17;
    var union_route_cursor = 0;
    while union_route_cursor < 11 limit Iterations(11) {
        union_route_total = union_route_total + union_route_cursor + 3;
        union_route_cursor = union_route_cursor + 1;
    }
    if union_route_total % 2 == 0 {
        union_route_total = union_route_total + 11;
    } else {
        union_route_total = union_route_total - 3;
    }
    var union_route_left = union_route_total + seed;
    var union_route_right = union_route_left * 2;
    var union_route_merged = union_route_right - union_route_left;
    if union_route_merged > 21 {
        union_route_total = union_route_total + union_route_merged;
    }
    return union_route_total;
}

flow command_sandbox_binary_binary_score(seed: i32) -> i32 ![]
{
    var union_score_total = seed + 17;
    var union_score_cursor = 0;
    while union_score_cursor < 9 limit Iterations(9) {
        union_score_total = union_score_total + union_score_cursor + 3;
        union_score_cursor = union_score_cursor + 1;
    }
    if union_score_total % 2 == 0 {
        union_score_total = union_score_total + 11;
    } else {
        union_score_total = union_score_total - 3;
    }
    var union_score_left = union_score_total + seed;
    var union_score_right = union_score_left * 2;
    var union_score_merged = union_score_right - union_score_left;
    if union_score_merged > 21 {
        union_score_total = union_score_total + union_score_merged;
    }
    return union_score_total;
}

flow command_sandbox_binary_binary_finish(seed: i32) -> i32 ![]
{
    var union_finish_total = seed - 17;
    var union_finish_cursor = 0;
    while union_finish_cursor < 9 limit Iterations(9) {
        union_finish_total = union_finish_total + union_finish_cursor + 3;
        union_finish_cursor = union_finish_cursor + 1;
    }
    if union_finish_total % 2 == 0 {
        union_finish_total = union_finish_total + 11;
    } else {
        union_finish_total = union_finish_total - 3;
    }
    var union_finish_left = union_finish_total + seed;
    var union_finish_right = union_finish_left * 2;
    var union_finish_merged = union_finish_right - union_finish_left;
    if union_finish_merged > 21 {
        union_finish_total = union_finish_total + union_finish_merged;
    }
    return union_finish_total;
}

flow main(args: Array<string>) -> i32 ![Command.run<DefaultCommandSandbox>]
{
    var binary_seed = 9;
    if args.len() > 0 {
        binary_seed = binary_seed + 1;
    } else {
        binary_seed = binary_seed + 2;
    }
    let binary_result = command_sandbox_binary_binary_entry(binary_seed);
    if binary_result > 0 {
        return 0;
    }
    return 1;
}
