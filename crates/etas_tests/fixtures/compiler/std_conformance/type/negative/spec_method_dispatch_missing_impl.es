// support: SpecMethodDispatch
// layer: type
// polarity: negative
// status: covered-negative
// expect: type does not satisfy selected spec
module tests.compiler.std_conformance.type.spec_method_dispatch_missing_impl;

type Packet = {
    body: string,
};

spec Encoder {
    flow encode(value: Packet) -> string;
}

flow main(args: Array<string>) -> i32
{
    let packet = Packet { body = "missing impl" };
    let encoded: string = packet::Encoder.encode();
    return 0;
}
