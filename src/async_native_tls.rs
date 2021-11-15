//! `async-native-tls` integration.
use real_async_native_tls::TlsConnector as AsyncTlsConnector;
use real_async_native_tls::TlsStream;

use tungstenite::client::uri_mode;
use tungstenite::handshake::client::Request;
use tungstenite::stream::Mode;
use tungstenite::Error;

use futures_io::{AsyncRead, AsyncWrite};

use crate::stream::Stream as StreamSwitcher;
use crate::{
    client_async_with_config, domain, IntoClientRequest, Response, WebSocketConfig,
    WebSocketStream,
};

/// A stream that might be protected with TLS.
pub type MaybeTlsStream<S> = StreamSwitcher<S, TlsStream<S>>;

pub(crate) type AutoStream<S> = MaybeTlsStream<S>;

async fn wrap_stream<S>(
    socket: S,
    domain: String,
    connector: Option<AsyncTlsConnector>,
    mode: Mode,
) -> Result<AutoStream<S>, Error>
    where
        S: 'static + AsyncRead + AsyncWrite + Unpin,
{
    match mode {
        Mode::Plain => Ok(StreamSwitcher::Plain(socket)),
        Mode::Tls => {
            let stream = {
                let connector = if let Some(connector) = connector {
                    connector
                } else {
                    AsyncTlsConnector::new()
                };
                connector
                    .connect(&domain, socket)
                    .await
                    .map_err(|err| Error::Tls(err.into()))?
            };
            Ok(StreamSwitcher::Tls(stream))
        }
    }
}

/// Type alias for the stream type of the `client_async()` functions.
pub type ClientStream<S> = AutoStream<S>;

/// Creates a WebSocket handshake from a request and a stream,
/// upgrading the stream to TLS if required and using the given
/// connector and WebSocket configuration.
pub async fn client_async_tls_with_connector_and_config<R, S>(
    request: R,
    stream: S,
    connector: Option<AsyncTlsConnector>,
    config: Option<WebSocketConfig>,
) -> Result<(WebSocketStream<AutoStream<S>>, Response), Error>
    where
        R: IntoClientRequest + Unpin,
        S: 'static + AsyncRead + AsyncWrite + Unpin,
        AutoStream<S>: Unpin,
{
    let request: Request = request.into_client_request()?;

    let domain = domain(&request)?;

    // Make sure we check domain and mode first. URL must be valid.
    let mode = uri_mode(request.uri())?;

    let stream = wrap_stream(stream, domain, connector, mode).await?;
    client_async_with_config(request, stream, config).await
}

/// Creates a WebSocket handshake from a request and a stream,
/// upgrading the stream to TLS if required.
pub async fn client_async_tls<R, S>(
    request: R,
    stream: S,
) -> Result<(WebSocketStream<ClientStream<S>>, Response), Error>
    where
        R: IntoClientRequest + Unpin,
        S: 'static + AsyncRead + AsyncWrite + Unpin,
        AutoStream<S>: Unpin,
{
    client_async_tls_with_connector_and_config(request, stream, None, None).await
}

/// Creates a WebSocket handshake from a request and a stream,
/// upgrading the stream to TLS if required and using the given
/// WebSocket configuration.
pub async fn client_async_tls_with_config<R, S>(
    request: R,
    stream: S,
    config: Option<WebSocketConfig>,
) -> Result<(WebSocketStream<ClientStream<S>>, Response), Error>
    where
        R: IntoClientRequest + Unpin,
        S: 'static + AsyncRead + AsyncWrite + Unpin,
        AutoStream<S>: Unpin,
{
    client_async_tls_with_connector_and_config(request, stream, None, config).await
}

/// Creates a WebSocket handshake from a request and a stream,
/// upgrading the stream to TLS if required and using the given
/// connector.
pub async fn client_async_tls_with_connector<R, S>(
    request: R,
    stream: S,
    connector: Option<AsyncTlsConnector>,
) -> Result<(WebSocketStream<ClientStream<S>>, Response), Error>
    where
        R: IntoClientRequest + Unpin,
        S: 'static + AsyncRead + AsyncWrite + Unpin,
        AutoStream<S>: Unpin,
{
    client_async_tls_with_connector_and_config(request, stream, connector, None).await
}
