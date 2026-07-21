module app.consumer;

import app.model.{LocalPacket};
import trace_contract.contract.{ExternalPacket, ExternalStringBox, ExternalEncoder, external_box, external_packet};

public flow consume_external(packet: ExternalPacket) -> string
{
    return packet::ExternalEncoder.encode();
}

public flow consume_external_alias(boxed: ExternalStringBox) -> ExternalStringBox
{
    return boxed;
}

public flow consume_local(local: LocalPacket) -> string
{
    let boxed: ExternalStringBox = external_box(local.body);
    let echoed: ExternalStringBox = consume_external_alias(boxed);
    let packet: ExternalPacket = external_packet(local.body);
    return consume_external(packet);
}
