// support: TraceSpec, Fs.read, Fs.write
// layer: effect
// polarity: negative
// status: blocked-by-impl
// expect: temporal trace spec requires Fs.read before Fs.write
// expect: checker must fail closed when code may perform Fs.write first
module tests.compiler.std_conformance.effect.trace_temporal_order_violation;

effect Fs {
    action read<R>() -> bytes;
    action write<R>(content: bytes) -> unit;
}

spec ReadBeforeWrite: trace = +Fs.read<_> & +Fs.write<_> & (Fs.read<_> >> Fs.write<_>);

flow write_then_read(content: bytes) -> bytes ![Fs.write<string>, Fs.read<string>] ~ ReadBeforeWrite
{
    perform Fs.write<string>(content);
    return perform Fs.read<string>();
}

flow main(args: Array<string>) -> i32
{
    return 0;
}
