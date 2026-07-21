// support: TraceSpec, Fs.read, Fs.write
// layer: effect
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.effect.trace_spec_read_only;

effect Fs {
    action read<R>() -> bytes;
    action write<R>(content: bytes) -> unit;
}

spec ReadOnly: trace = +Fs.read<_> & -Fs.write<_>;

flow load_snapshot() -> bytes ![Fs.read<string>] ~ ReadOnly
{
    return perform Fs.read<string>();
}

flow main(args: Array<string>) -> i32
{
    let snapshot: bytes = load_snapshot();
    return 0;
}
