module app.substrate.service;

import std.codec.text.utf8_encode;
import std.http.codec.{HttpCodecError, HttpHeader, HttpWireRequest, HttpWireResponse, HttpWireResponseHead, decode_response, decode_response_head, encode_request};
import std.net.tcp.{Host as TcpHost, NetworkError, Port, TcpOptions, TcpStream, connect as tcp_connect};
import std.stream.{ByteLimit, StreamError, Timeout, read_until_limit};
import std.tls.{Host as TlsHost, TlsConfig, TlsError, TlsStream, connect as tls_connect};
import app.substrate.model.{SubstrateReport, build_report};

public flow encode_http_request(request: HttpWireRequest) -> Result<bytes, HttpCodecError> ![]
{
    return encode_request(request);
}

public flow encode_root_request() -> Result<bytes, HttpCodecError> ![]
{
    let headers: List<HttpHeader> = [];
    let body = utf8_encode("");
    let request = HttpWireRequest {
        method = "GET",
        target = "/",
        version = "HTTP/1.1",
        headers = headers,
        body = body,
    };
    return encode_request(request);
}

public flow decode_http_head(head: bytes) -> Result<HttpWireResponseHead, HttpCodecError> ![]
{
    return decode_response_head(head);
}

public flow decode_http_response(response: bytes) -> Result<HttpWireResponse, HttpCodecError> ![]
{
    return decode_response(response);
}

public flow open_tcp(host: TcpHost, port: Port, options: TcpOptions) -> TcpStream ![Error<NetworkError>]
{
    return tcp_connect(host, port, options);
}

public flow wrap_tls(stream: TcpStream, host: TlsHost, config: TlsConfig) -> TlsStream ![Error<TlsError>]
{
    return tls_connect(stream, host, config);
}

public flow read_bounded(stream: TlsStream, limit: ByteLimit, timeout: Option<Timeout>) -> bytes ![Error<StreamError>]
{
    return read_until_limit(stream, limit, timeout);
}

public flow summarize_substrate() -> SubstrateReport ![]
{
    return build_report();
}
