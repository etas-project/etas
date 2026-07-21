module trace_contract.contract;

type ExternalPacket = {
    body: string,
};

policy LegacyContract {
    allow Fs.read<_>;
}

flow stale_source(packet: ExternalPacket) -> string
    follows LegacyContract
{
    return packet.body;
}
