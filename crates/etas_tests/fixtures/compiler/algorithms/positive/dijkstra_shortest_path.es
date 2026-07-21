module tests.compiler.algorithms.dijkstra_shortest_path;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, len_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

type Edge = {
    from: i32,
    to: i32,
    cost: i32,
};

flow relax(edges: Array<Edge>, dist: Array<i32>, visited: Array<bool>, current: i32) -> Array<i32> {
    var next_dist = dist;

    for edge in edges limit Iterations(1024) {
        if edge.from == current && !visited[edge.to] {
            let candidate = next_dist[current] + edge.cost;
            if candidate < next_dist[edge.to] {
                next_dist[edge.to] = candidate;
            }
        }
    }

    return next_dist;
}

flow next_unvisited(dist: Array<i32>, visited: Array<bool>) -> i32 {
    var best = -1;
    var i = 0;

    while i < len_i32_list(dist) limit Iterations(1024) {
        if !visited[i] && (best == -1 || dist[i] < dist[best]) {
            best = i;
        }
        i = i + 1;
    }

    return best;
}

flow dijkstra(edges: Array<Edge>, nodes: i32, start: i32, goal: i32) -> i32 {
    var dist = repeat_i32(1_000_000, nodes);
    var visited = repeat_bool(false, nodes);
    dist[start] = 0;

    var step = 0;
    while step < nodes limit Iterations(1024) {
        let current = next_unvisited(dist, visited);
        if current == -1 {
            break;
        }
        if current == goal {
            return dist[current];
        }
        visited[current] = true;
        dist = relax(edges, dist, visited, current);
        step = step + 1;
    }

    return dist[goal];
}

flow solve(input: string) -> string {
    let edges = [
        Edge { from = 0, to = 1, cost = 4 },
        Edge { from = 0, to = 2, cost = 1 },
        Edge { from = 2, to = 1, cost = 2 },
        Edge { from = 1, to = 3, cost = 1 },
        Edge { from = 2, to = 3, cost = 5 },
    ];
    let parts = parse_i32_list(input);
    return to_string(dijkstra(edges, 4, parts[0], parts[1]));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
