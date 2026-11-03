module tests.reflection;
import std.compiler.{compile, CompileError, CallableDef};
flow normalize(value: i32) -> i32 { return value; }
flow main() -> unit ![Error<CompileError>] {
    let definition: CallableDef = ^normalize;
    let candidate = compile(definition);
    return;
}
