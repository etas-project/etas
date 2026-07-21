module tests.compiler.type_system.positive.bodyless_tcp_stream_from_std_host;

import std.net.tcp.{Host, NetworkError, Port, TcpOptions, TcpStream, connect as tcp_connect};

flow open_tcp(host: Host, port: Port, options: TcpOptions) -> TcpStream ![Error<NetworkError>]
{
    return tcp_connect(host, port, options);
}

flow accept_stream(stream: TcpStream) -> TcpStream
{
    return stream;
}

flow round_trip(host: Host, port: Port, options: TcpOptions) -> TcpStream ![Error<NetworkError>]
{
    let stream = open_tcp(host, port, options);
    return accept_stream(stream);
}

flow main(args: Array<string>) -> i32
{
    return 0;
}
