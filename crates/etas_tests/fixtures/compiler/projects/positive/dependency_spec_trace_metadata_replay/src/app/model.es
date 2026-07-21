module app.model;

public type LocalPacket = {
    body: string,
};

public flow local_packet() -> LocalPacket
{
    return LocalPacket { body = "metadata replay" };
}
