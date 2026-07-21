module tests.compiler.algorithms.merge_intervals;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

type Interval = {
    start: i32,
    end: i32,
};

flow max_i32(left: i32, right: i32) -> i32 {
    if left > right {
        return left;
    }

    return right;
}

flow merge(intervals: Array<Interval>) -> Array<Interval> {
    var out: Array<Interval> = [intervals[0]];
    var i: usize = 1;

    while i < len(intervals) limit Iterations(1024) {
        let last_index = len(out) - 1;
        if intervals[i].start <= out[last_index].end {
            out[last_index].end = max_i32(out[last_index].end, intervals[i].end);
        } else {
            out = out.push(intervals[i]);
        }
        i = i + 1;
    }

    return out;
}

flow flatten(intervals: Array<Interval>) -> Array<i32> {
    var out: Array<i32> = [];
    for interval in intervals limit Iterations(1024) {
        out = out.push(interval.start);
        out = out.push(interval.end);
    }
    return out;
}

flow solve(input: string) -> string {
    let values = parse_i32_list(input);
    let intervals = [
        Interval { start = values[0], end = values[1] },
        Interval { start = values[2], end = values[3] },
        Interval { start = values[4], end = values[5] },
        Interval { start = values[6], end = values[7] },
    ];
    return join_i32_list(flatten(merge(intervals)), " ");
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
