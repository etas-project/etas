module examples.stdin_stdout;

import std.effects.Console;
import std.io.{read_line, println};
import std.option.unwrap;

flow main(args: Array<string>) -> i32 ![Console, Error<IOError>]
{
    let line = read_line();
    println(line);
    return 0;
}
