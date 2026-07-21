module examples.hello;

import std.effects.Console;
import std.io.println;

flow main(args: Array<string>) -> i32 ![Console, Error<IOError>]
{
    println("hello from Etas");
    return 0;
}
