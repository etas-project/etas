module app.main;

import std.fs.{Region, IOError, path, read_bytes};
import std.codec.text.utf8_encode;

type Sandbox;
impl Sandbox ~ Region;

flow main(args: Array<string>) -> i32 ![] {
    let target = match path<Sandbox>("data.txt") {
        Ok(value) => value,
        Err(_) => { return 0; },
    };
    return handle {
        if read_bytes(target) == utf8_encode("source sdk") { 42 } else { 0 }
    } with {
        Error<IOError>.raise(error) => { finish 0; }
    };
}
