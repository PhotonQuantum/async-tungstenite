//! `async-std` integration.
use tungstenite::client::IntoClientRequest;
use tungstenite::handshake::client::{Request, Response};
use tungstenite::protocol::WebSocketConfig;
use tungstenite::Error;

use async_std::net::TcpStream;

use super::{domain, port, WebSocketStream};

#[cfg(not(any(feature = "async-tls", feature = "async-native-tls")))]
pub(crate) mod dummy_tls {
    use futures_io::{AsyncRead, AsyncWrite};

    use tungstenite::client::{uri_mode, IntoClientRequest};
    use tungstenite::handshake::client::Request;
    use tungstenite::stream::Mode;
    use tungstenite::Error;

    use crate::{client_async_with_config, domain, Response, WebSocketConfig, WebSocketStream};

    pub type AutoStream<S> = S;
    type Connector = ();

    async fn wrap_stream<S>(
        socket: S,
        _domain: String,
        _connector: Option<()>,
        mode: Mode,
    ) -> Result<AutoStream<S>, Error>
    where
        S: 'static + AsyncRead + AsyncWrite + Unpin,
    {
        match mode {
            Mode::Plain => Ok(socket),
            Mode::Tls => Err(Error::Url(
                tungstenite::error::UrlError::TlsFeatureNotEnabled,
            )),
        }
    }

    /// Creates a WebSocket handshake from a request and a stream,
    /// upgrading the stream to TLS if required and using the given
    /// connector and WebSocket configuration.
    pub async fn client_async_tls_with_connector_and_config<R, S>(
        request: R,
        stream: S,
        connector: Option<Connector>,
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
}

#[cfg(not(any(feature = "async-tls", feature = "async-native-tls")))]
pub use self::dummy_tls::client_async_tls_with_connector_and_config;
#[cfg(not(any(feature = "async-tls", feature = "async-native-tls")))]
use self::dummy_tls::AutoStream;

#[cfg(all(feature = "async-tls", not(feature = "async-native-tls")))]
pub use crate::async_tls::client_async_tls_with_connector_and_config;
#[cfg(all(feature = "async-tls", not(feature = "async-native-tls")))]
use crate::async_tls::AutoStream;
#[cfg(all(feature = "async-tls", not(feature = "async-native-tls")))]
type Connector = real_async_tls::TlsConnector;

#[cfg(feature = "async-native-tls")]
pub use crate::async_native_tls::client_async_tls_with_connector_and_config;
#[cfg(feature = "async-native-tls")]
use crate::async_native_tls::AutoStream;
#[cfg(feature = "async-native-tls")]
type Connector = real_async_native_tls::TlsConnector;

/// Type alias for the stream type of the `client_async()` functions.
pub type ClientStream<S> = AutoStream<S>;

/// Type alias for the stream type of the `connect_async()` functions.
pub type ConnectStream = ClientStream<TcpStream>;

/// Connect to a given URL.
pub async fn connect_async<R>(
    request: R,
) -> Result<(WebSocketStream<ConnectStream>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    connect_async_with_config(request, None).await
}

/// Connect to a given URL with a given WebSocket configuration.
pub async fn connect_async_with_config<R>(
    request: R,
    config: Option<WebSocketConfig>,
) -> Result<(WebSocketStream<ConnectStream>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    let request: Request = request.into_client_request()?;

    let domain = domain(&request)?;
    let port = port(&request)?;

    let try_socket = TcpStream::connect((domain.as_str(), port)).await;
    let socket = try_socket.map_err(Error::Io)?;
    client_async_tls_with_connector_and_config(request, socket, None, config).await
}

#[cfg(any(feature = "async-tls", feature = "async-native-tls"))]
/// Connect to a given URL using the provided TLS connector.
pub async fn connect_async_with_tls_connector<R>(
    request: R,
    connector: Option<Connector>,
) -> Result<(WebSocketStream<ConnectStream>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    connect_async_with_tls_connector_and_config(request, connector, None).await
}

#[cfg(any(feature = "async-tls", feature = "async-native-tls"))]
/// Connect to a given URL using the provided TLS connector.
pub async fn connect_async_with_tls_connector_and_config<R>(
    request: R,
    connector: Option<Connector>,
    config: Option<WebSocketConfig>,
) -> Result<(WebSocketStream<ConnectStream>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    let request: Request = request.into_client_request()?;

    let domain = domain(&request)?;
    let port = port(&request)?;

    let try_socket = TcpStream::connect((domain.as_str(), port)).await;
    let socket = try_socket.map_err(Error::Io)?;
    client_async_tls_with_connector_and_config(request, socket, connector, config).await
}
