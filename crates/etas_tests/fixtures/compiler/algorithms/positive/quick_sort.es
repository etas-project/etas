module tests.compiler.algorithms.quick_sort;

import std.collections.{Array, len};
import std.io.{read_line, println};
import std.option.unwrap;
import tests.compiler.support.algorithms.{abs, join_i32_list, matrix_i32, parse_i32, parse_i32_list, repeat_bool, repeat_i32, slice, split_lines, string_len, to_string, trim};

type PartitionResult = {
    values: Array<i32>,
    pivot: usize,
};

flow partition(values: Array<i32>, low: usize, high: usize) -> PartitionResult {
    var out = values;
    let pivot = out[high];
    var i = low;
    var j = low;

    while j < high limit Iterations(1024) {
        if out[j] <= pivot {
            let tmp = out[i];
            out[i] = out[j];
            out[j] = tmp;
            i = i + 1;
        }

        j = j + 1;
    }

    let tmp = out[i];
    out[i] = out[high];
    out[high] = tmp;
    return PartitionResult { values = out, pivot = i };
}

flow quick_sort_range(values: Array<i32>, low: usize, high: usize) -> Array<i32> {
    var out = values;

    if low < high {
        let partitioned = partition(out, low, high);
        out = partitioned.values;
        let pivot_index = partitioned.pivot;
        if pivot_index > low {
            out = quick_sort_range(out, low, pivot_index - 1);
        }
        out = quick_sort_range(out, pivot_index + 1, high);
    }

    return out;
}

flow quick_sort(values: Array<i32>) -> Array<i32> {
    let n = len(values);
    var out = values;
    if n > 1 {
        out = quick_sort_range(out, 0, n - 1);
    }
    return out;
}

flow solve(input: string) -> string {
    let values = parse_i32_list(input);
    return join_i32_list(quick_sort(values), " ");
}

flow main(args: Array<string>) -> i32 ![Error<IOError>, Error<IndexError>]
{
    let input = read_line();
    println(solve(input));
    return 0;
}
