module tests.compiler.algorithms.negative.bfs_missing_visited_check;

import std.collections.{List, len};

flow bfs(queue: List<i32>) -> i32 {
    var head = 0;
    while head < len(queue) limit Iterations(4096) {
        queue.push(queue[head]);
    }
    return head;
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
