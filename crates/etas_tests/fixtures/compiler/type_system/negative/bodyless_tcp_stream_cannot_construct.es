module tests.compiler.type_system.negative.bodyless_tcp_stream_cannot_construct;

type TcpStream;

flow main(args: Array<string>) -> i32
{
    let stream = TcpStream();
    return 0;
}
