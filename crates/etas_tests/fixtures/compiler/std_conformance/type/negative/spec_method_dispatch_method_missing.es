// support: SpecMethodDispatch
// layer: type
// polarity: negative
// status: covered-negative
// expect: spec `Encoder` does not declare method `decode`
module tests.compiler.std_conformance.type.spec_method_dispatch_method_missing;

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
    let packet = Packet { body = "wrong method" };
    let decoded: string = packet::Encoder.decode();
    return 0;
}
