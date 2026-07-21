// support: SpecMethodDispatch
// layer: type
// polarity: negative
// status: covered-negative
// expect: duplicate impl of spec `Encoder` for this type
module tests.compiler.std_conformance.type.spec_method_dispatch_ambiguous_impl;

type Packet = {
    body: string,
};

spec Encoder {
    flow encode(value: Packet) -> string;
}

impl Packet ~ Encoder {
    flow encode(value: Packet) -> string
    {
        return value.body;
    }
}

impl Packet ~ Encoder {
    flow encode(value: Packet) -> string
    {
        return value.body;
    }
}

flow main(args: Array<string>) -> i32
{
    let packet = Packet { body = "ambiguous" };
    let encoded: string = packet::Encoder.encode();
    return 0;
}
