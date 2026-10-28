module tests.reflection;
import std.compiler.{compile, CompileError};
flow normalize(value: i32) -> i32 { return value; }
flow main() -> unit ![Error<CompileError>] { let candidate = compile(^normalize); return; }
