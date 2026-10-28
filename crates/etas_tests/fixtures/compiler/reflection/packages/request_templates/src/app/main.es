module app.main;
import std.compiler.{CompileError, compile};
import requests.api.{build, apply, wrap};

agent Worker() -> i32 { return 7; }
agent Other() -> i32 { return 9; }

flow main() -> Worker ![Error<CompileError>] {
    return handle { build(^Worker) } with {
        Code.compile(_) => { resume Worker; }
    };
}
