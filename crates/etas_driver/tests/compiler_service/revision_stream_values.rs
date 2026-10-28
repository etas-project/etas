use super::revisions::execute;
use etas_interpreter::api::InterpValue;

#[test]
fn a_compiled_revision_stream_reducer_mediates_every_read_in_its_parent_handler() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, CompileError};
import std.stream.{StreamRead, StreamError, ByteStream, ByteLimit, read_until_limit, Timeout};
import std.codec.text.utf8_encode;
import std.option.None;
import std.effects.Stream;
type Buffer = string;
impl Buffer ~ ByteStream;
agent Worker(stream: Buffer) -> bytes ![Error<StreamError>] {
    return read_until_limit(stream, ByteLimit { bytes = 4 }, None<Timeout>());
}
flow main() -> bool ![Error<CompileError>, Error<StreamError>] {
    let revised = compile(^Worker);
    var reads = 0;
    let result = handle { revised(Buffer("source stream")) } with {
        Stream.read<Buffer>(_) => {
            reads = reads + 1;
            let response = match reads { 1 => StreamRead.Data(utf8_encode("ab")),
                2 => StreamRead.Data(utf8_encode("cd")), _ => StreamRead.Eof, };
            resume response;
        }
    };
    return result == utf8_encode("abcd") && reads == 3;
}
"#,
        Some(5_000),
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn a_stream_read_resume_value_is_relocated_back_into_its_compiled_revision() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, CompileError};
import std.stream.{StreamRead, ByteStream, read, Timeout};
import std.codec.text.utf8_encode;
import std.option.None;
import std.effects.Stream;
type Buffer = string;
impl Buffer ~ ByteStream;
agent Worker(stream: Buffer) -> bytes {
    return match read(stream, 16, None<Timeout>()) {
        StreamRead.Data(content) => content,
        StreamRead.Eof => utf8_encode("eof"),
    };
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    let content = utf8_encode("from parent handler");
    let result = handle { revised(Buffer("source stream")) } with {
        Stream.read<Buffer>(_) => { resume StreamRead.Data(content); }
    };
    return result == content;
}
"#,
        Some(5_000),
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}

#[test]
fn a_stream_error_payload_is_relocated_to_its_parent_handler() {
    let result = execute(
        r#"
module app;
import std.compiler.{compile, CompileError};
import std.stream.StreamError;
agent Worker() -> unit ![Error<StreamError>] {
    perform Error<StreamError>.raise(StreamError.Host("from candidate"));
}
flow main() -> bool ![Error<CompileError>] {
    let revised = compile(^Worker);
    return handle { revised(); false } with {
        Error<StreamError>.raise(error) => {
            finish match error { StreamError.Host(message) => message == "from candidate", _ => false, };
        }
    };
}
"#,
        Some(5_000),
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.value(), Some(&InterpValue::Bool(true)));
}
