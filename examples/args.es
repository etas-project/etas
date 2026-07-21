module examples.args;

import std.effects.Console;
import std.io.println;

flow main(args: Array<string>) -> i32 ![Console, Error<IOError>, Error<IndexError>]
{
    println(args[0]);
    println(args[1]);
    return 0;
}
