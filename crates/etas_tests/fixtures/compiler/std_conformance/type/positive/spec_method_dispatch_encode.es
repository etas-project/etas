// support: SpecMethodDispatch
// layer: type
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.type.spec_method_dispatch_encode;

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

flow main(args: Array<string>) -> i32
{
    let packet = Packet { body = "ok" };
    let encoded: string = packet::Encoder.encode();
    return 0;
}
