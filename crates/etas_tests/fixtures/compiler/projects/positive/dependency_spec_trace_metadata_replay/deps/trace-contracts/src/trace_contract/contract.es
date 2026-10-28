module trace_contract.contract;

public type ExternalPacket = { body: string };
public type ExternalBox<T> = { value: T };
public alias ExternalStringBox = ExternalBox<string>;

public spec ExternalReadOnly: trace = -Console.stdout_write;

public spec ExternalEncoder {
    flow encode(packet: ExternalPacket) -> string;
}

impl ExternalPacket ~ ExternalEncoder {
    flow encode(packet: ExternalPacket) -> string ~ ExternalReadOnly {
        return packet.body;
    }
}

public flow external_packet(body: string) -> ExternalPacket {
    return ExternalPacket { body };
}

public flow external_box(value: string) -> ExternalStringBox {
    return ExternalBox<string> { value };
}
