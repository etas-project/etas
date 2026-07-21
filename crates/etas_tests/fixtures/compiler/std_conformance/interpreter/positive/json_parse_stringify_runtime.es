// support: JsonValue, JsonError
// layer: interpreter
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.interpreter.json_parse_stringify_runtime;

import std.json.{InvalidJson, JsonError, JsonValue, parse, stringify};

flow stringify_sample() -> Result<string, JsonError> {
    return match parse("[true,\"ok\"]") {
        Ok(value) => stringify(value),
        Err(error) => Err<string, JsonError>(error)
    };
}

flow invalid_json_shape() -> i32 {
    return match parse("{") {
        Ok(_) => 1,
        Err(InvalidJson(message)) => if message == "" { 1 } else { 0 },
        Err(_) => 1
    };
}

flow main(args: Array<string>) -> i32 {
    return match stringify_sample() {
        Ok(encoded) => if encoded == "[true,\"ok\"]" { invalid_json_shape() } else { 1 },
        Err(_) => 1
    };
}
