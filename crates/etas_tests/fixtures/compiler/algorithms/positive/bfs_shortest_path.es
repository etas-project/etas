module tests.compiler.algorithms.bfs_shortest_path;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

type Edge = {
    from: i32,
    to: i32,
};

flow neighbors(edges: Array<Edge>, node: i32) -> Array<i32> {
    var out: Array<i32> = [];
    for edge in edges limit Iterations(1024) {
        if edge.from == node {
            out = out.push(edge.to);
        }
    }
    return out;
}

flow bfs_shortest_path(edges: Array<Edge>, start: i32, goal: i32) -> i32 {
    var queue: Array<i32> = [start];
    var distance = repeat_i32(-1, 5);
    distance[start] = 0;
    var head: usize = 0;

    while head < len(queue) limit Iterations(4096) {
        let current = queue[head];
        head = head + 1;

        if current == goal {
            return distance[current];
        }

        for next in neighbors(edges, current) limit Iterations(1024) {
            if distance[next] == -1 {
                distance[next] = distance[current] + 1;
                queue = queue.push(next);
            }
        }
    }

    return -1;
}

flow solve(input: string) -> string {
    let edges = [
        Edge { from = 0, to = 1 },
        Edge { from = 0, to = 2 },
        Edge { from = 1, to = 3 },
        Edge { from = 2, to = 3 },
        Edge { from = 3, to = 4 },
    ];
    let parts = parse_i32_list(input);
    return to_string(bfs_shortest_path(edges, parts[0], parts[1]));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
