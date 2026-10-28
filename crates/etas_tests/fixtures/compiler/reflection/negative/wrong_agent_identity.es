module tests.reflection;
import std.compiler.AgentDef;
agent Worker() -> i32 { return 1; }
agent Other() -> i32 { return 2; }
flow main() -> unit { let definition: AgentDef<Worker> = ^Other; return; }
