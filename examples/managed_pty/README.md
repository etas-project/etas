# Managed terminal session

Run the consumer with an installed Linux SDK from this directory.

```sh
etas pkg lock .
etas run . --trace-out trace.json
```

The child checks all three terminal descriptors and reports its initial size of
80 columns by 24 rows. The application resizes it to 120 by 40, submits a line,
and reads the shared output cursor two bytes at a time. It checks the echo,
the resized dimensions and the final unterminated UTF-8 text before checking
exit code 7. The application itself returns 0 on success.

The example's runtime command profile enables `pty` and allows `/bin/sh` for this
fixed script. The public trace records terminal events and byte counts. To replay
the contents, choose a new private recording file when starting the session.

```sh
etas run . --trace-out recorded-trace.json --pty-recording terminal.jsonl
etas --format json replay recorded-trace.json --pty-recording terminal.jsonl
```

A terminal UI can forward encoded keystrokes using `write` and window changes
using `resize` while a read is pending. Feed each `Output` chunk to the same
incremental decoder or terminal emulator. Flush it on `Exited` or `Stopped`
before displaying the terminal status. Input echo can also appear in output,
so the private recording may contain typed credentials.
