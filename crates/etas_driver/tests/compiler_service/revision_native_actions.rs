use super::revisions::execute;
use etas_interpreter::api::InterpValue;

fn assert_value(source: &str, expected: InterpValue) {
    let result = execute(source, Some(5_000));
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&expected));
}

#[test]
fn a_parent_handler_intercepts_native_tcp_without_a_network_host() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
import std.net.tcp.{Host, Port, TcpOptions, connect};
import std.effects.Net;
agent Worker() -> bool {
    let stream = connect(Host { host = "example.test" }, Port { port = 443 }, TcpOptions {});
    return false;
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    return handle { revised() } with {
        Net.tcp_connect<_, _>(host, port) => { finish host.host == "example.test" && port.port == 443; }
    };
}
"#,
        InterpValue::Bool(true),
    );
}

#[test]
fn a_parent_handler_intercepts_the_exact_filesystem_region_across_revisions() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
import std.fs.{Region, WorkspacePath, path, read_bytes};
import std.effects.Fs;
type Root;
type Other;
impl Root ~ Region;
impl Other ~ Region;
agent Worker(path: WorkspacePath<Root>) -> bool {
    let content = read_bytes(path);
    return false;
}
flow main() -> bool ![Error<CompileError>] {
    let file = match path<Root>("cached") { Ok(path) => path, Err(_) => { return false; }, };
    let revised = compile(^Worker);
    return handle { revised(file) } with {
        Fs.read<Other>(_) => { finish false; },
        Fs.read<Root>(_) => { finish true; }
    };
}
"#,
        InterpValue::Bool(true),
    );
}

#[test]
fn a_parent_handler_can_handle_a_user_defined_byte_stream_in_a_compiled_agent() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
import std.stream.{ByteStream, read, Timeout};
import std.option.None;
import std.effects.Stream;
type Buffer = string;
impl Buffer ~ ByteStream;
agent Worker(stream: Buffer) -> i32 {
    let content = read(stream, 16, None<Timeout>());
    return 0;
}
flow main() -> i32 ![Error<CompileError>] {
    let revised = compile(^Worker);
    return handle { revised(Buffer("source value")) } with {
        Stream.read<Buffer>(_) => { finish 41; }
    };
}
"#,
        InterpValue::i32(41),
    );
}

#[test]
fn a_native_callable_keeps_its_action_proof_when_passed_into_a_compiled_agent() {
    assert_value(
        r#"
module app;
import std.compiler.{compile, CompileError};
import std.net.tcp.{Host, Port, TcpOptions, TcpStream, NetworkError, connect};
import std.effects.Net;
agent Worker(open: (Host, Port, TcpOptions) -> TcpStream ![Error<NetworkError>]) -> bool {
    let stream = open(Host { host = "example.test" }, Port { port = 443 }, TcpOptions {});
    return false;
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let open = connect;
    return handle { revised(open) } with {
        Net.tcp_connect<_, _>(host, port) => { finish host.host == "example.test" && port.port == 443; },
        Error<NetworkError>.raise(_) => { finish false; }
    };
}
"#,
        InterpValue::Bool(true),
    );
}
