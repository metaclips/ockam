pub(crate) mod common;
mod connection;
mod lifecycle;
mod listener;
mod portals;

pub(crate) use common::*;

pub use crate::portal::options::*;
pub use connection::*;
pub use listener::*;
pub use portals::*;

use crate::TcpRegistry;
use ockam_core::compat::sync::Arc;
use ockam_core::Result;
use ockam_node::{Context, HasContext};

/// High level management interface for TCP transports
///
/// Be aware that only one `TcpTransport` can exist per node, as it
/// registers itself as a router for the `TCP` address type.  Multiple
/// calls to [`TcpTransport::create`](crate::TcpTransport::create)
/// will fail.
///
/// To listen for incoming connections use
/// [`tcp.listen()`](crate::TcpTransport::listen).
///
/// To register additional connections on an already initialised
/// `TcpTransport`, use [`tcp.connect()`](crate::TcpTransport::connect).
/// This step is optional because the underlying TcpRouter is capable of lazily
/// establishing a connection upon arrival of an initial message.
///
/// ```rust
/// use ockam_transport_tcp::{TcpConnectionOptions, TcpListenerOptions, TcpTransport};
/// # use ockam_node::Context;
/// # use ockam_core::Result;
/// # async fn test(ctx: Context) -> Result<()> {
/// let tcp = TcpTransport::create(&ctx)?;
/// tcp.listen("127.0.0.1:8000", TcpListenerOptions::new()).await?; // Listen on port 8000
/// tcp.connect("127.0.0.1:5000", TcpConnectionOptions::new()).await?; // And connect to port 5000
/// # Ok(()) }
/// ```
///
/// The same `TcpTransport` can also bind to multiple ports.
///
/// ```rust
/// use ockam_transport_tcp::{TcpListenerOptions, TcpTransport};
/// # use ockam_node::Context;
/// # use ockam_core::Result;
/// # async fn test(ctx: Context) -> Result<()> {
/// let tcp = TcpTransport::create(&ctx)?;
/// tcp.listen("127.0.0.1:8000", TcpListenerOptions::new()).await?; // Listen on port 8000
/// tcp.listen("127.0.0.1:9000", TcpListenerOptions::new()).await?; // Listen on port 9000
/// # Ok(()) }
/// ```
#[derive(Clone, Debug)]
pub struct TcpTransport {
    ctx: Arc<Context>,
    registry: TcpRegistry,

    #[cfg(privileged_portals_support)]
    pub(crate) ebpf_support: Arc<crate::privileged_portal::TcpTransportEbpfSupport>,
}

impl TcpTransport {
    /// Constructor.
    pub fn new(ctx: Context) -> Self {
        Self {
            ctx: Arc::new(ctx),
            registry: TcpRegistry::default(),
            #[cfg(privileged_portals_support)]
            ebpf_support: Default::default(),
        }
    }
}

/// This trait adds a `create_tcp_transport` method to any struct returning a Context.
/// This is the case for an ockam::Node, so you can write `node.create_tcp_transport()`
pub trait TcpTransportExtension: HasContext {
    /// Create a TCP transport
    fn create_tcp_transport(&self) -> Result<TcpTransport> {
        TcpTransport::create(self.get_context())
    }
}

impl<A: HasContext> TcpTransportExtension for A {}
