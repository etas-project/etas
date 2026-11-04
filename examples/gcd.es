module examples.gcd;

import std.effects.Console;
import std.io.println;
import std.text.from_i32;

flow gcd(a0: i32, b0: i32) -> i32 {
    var a = a0;
    var b = b0;

    while b != 0 limit Iterations(32) {
        let remainder = a % b;
        a = b;
        b = remainder;
    }

    return a;
}

flow main(args: Array<string>) -> i32 ![Console, Error<IOError>]
{
    println(from_i32(gcd(84, 30)));
    return 0;
}
