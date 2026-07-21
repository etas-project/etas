module tests.compiler.algorithms.number_of_islands;

import std.collections.{Array, len};
import std.io.{read_line, println};
import tests.compiler.support.algorithms.{abs, join_i32_list, len_i32_grid, len_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

flow flood(grid: Array<Array<i32>>, row: i32, col: i32) -> Array<Array<i32>> {
    if row < 0 || col < 0 || row >= len_i32_grid(grid) || col >= len_i32_list(grid[row]) {
        return grid;
    }

    if grid[row][col] == 0 {
        return grid;
    }

    var next = grid;
    next[row][col] = 0;
    next = flood(next, row + 1, col);
    next = flood(next, row - 1, col);
    next = flood(next, row, col + 1);
    next = flood(next, row, col - 1);
    return next;
}

flow count_islands(grid: Array<Array<i32>>) -> i32 {
    var current = grid;
    var count = 0;
    var row = 0;

    while row < len_i32_grid(current) limit Iterations(1024) {
        var col = 0;
        while col < len_i32_list(current[row]) limit Iterations(1024) {
            if current[row][col] == 1 {
                count = count + 1;
                current = flood(current, row, col);
            }
            col = col + 1;
        }
        row = row + 1;
    }

    return count;
}

flow solve(input: string) -> string {
    let grid: Array<Array<i32>> = [
        [1, 1, 0, 0],
        [1, 0, 0, 1],
        [0, 0, 1, 1],
    ];
    return to_string(count_islands(grid));
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
