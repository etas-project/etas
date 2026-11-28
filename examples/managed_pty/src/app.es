module app;

import std.command.{Command, DefaultCommandSandbox, ExitStatus, PtyEvent, PtySize, start_pty};
import std.codec.text.utf8_encode;
import std.bytes.BytesBuilder;

flow main(args: Array<string>) -> i32 {
    let script = "test -t 0 && test -t 1 && test -t 2 || exit 1; stty size; IFS= read -r line; stty size; printf '尾:%s' \"$line\" >&2; exit 7";
    let tty = start_pty(Command.new("/bin/sh", ["-c", script]), DefaultCommandSandbox, PtySize { cols= 80, rows= 24 });
    let initial = utf8_encode("24 80\r\n");
    var size = BytesBuilder.new();
    while size.len_bytes() < initial.length {
        match tty.read(1) {
            PtyEvent.Output(part) => size.append(part),
            _ => { return 1; },
        };
    }
    if size.build() != initial { return 2; }

    let alias = tty;
    alias.resize(PtySize { cols= 120, rows= 40 });
    tty.write(utf8_encode("hello\n"));

    // A terminal may split an escape sequence or a UTF-8 character across reads.
    var output = BytesBuilder.new();
    while true {
        match alias.read(2) {
            PtyEvent.Output(part) => output.append(part),
            PtyEvent.Exited(ExitStatus.Code(7)) => { break; },
            _ => { return 3; },
        };
    }
    if output.build() != utf8_encode("hello\r\n40 120\r\n尾:hello") { return 4; }
    alias.stop();
    return match tty.read(1) {
        PtyEvent.Exited(ExitStatus.Code(7)) => 0,
        _ => 5,
    };
}
