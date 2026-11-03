module tests.reflection;
import std.compiler.{compile, CompileError};
agent Worker(value: bool) -> bool ![] { return value; }
impl agent Worker { public flow helper(value: bool) -> bool ![] { return value; } }
flow main() -> unit ![Error<CompileError>] { let detached = compile(^Worker.helper); return; }
