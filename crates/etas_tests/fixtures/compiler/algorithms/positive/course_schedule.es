module tests.compiler.algorithms.course_schedule;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

type Edge = {
    from: i32,
    to: i32,
};

flow can_finish(courses: i32, edges: Array<Edge>) -> bool {
    var indegree = repeat_i32(0, courses);
    for edge in edges limit Iterations(1024) {
        indegree[edge.to] = indegree[edge.to] + 1;
    }

    var queue: Array<i32> = [];
    var course = 0;
    while course < courses limit Iterations(1024) {
        if indegree[course] == 0 {
            queue = queue.push(course);
        }
        course = course + 1;
    }

    var visited = 0;
    var head: usize = 0;
    while head < len(queue) limit Iterations(4096) {
        let current = queue[head];
        head = head + 1;
        visited = visited + 1;

        for edge in edges limit Iterations(1024) {
            if edge.from == current {
                indegree[edge.to] = indegree[edge.to] - 1;
                if indegree[edge.to] == 0 {
                    queue = queue.push(edge.to);
                }
            }
        }
    }

    return visited == courses;
}

flow solve(input: string) -> string {
    let edges = [
        Edge { from = 0, to = 1 },
        Edge { from = 1, to = 2 },
        Edge { from = 2, to = 3 },
    ];
    if can_finish(parse_i32(trim(input)), edges) {
        return "true";
    }

    return "false";
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
