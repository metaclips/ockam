// This node creates a tcp connection to a node at 127.0.0.1:4000
// Starts a relay worker to forward messages to 127.0.0.1:4000
// Starts a tcp listener at 127.0.0.1:3000
// It then runs forever waiting to route messages.

use hello_ockam::Relay;
use ockam::tcp::{TcpConnectionOptions, TcpListenerOptions, TcpTransportExtension};
use ockam::{node, Context, Result};

#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    // Create a node with default implementations
    let node = node(ctx).await?;

    // Initialize the TCP Transport
    let tcp = node.create_tcp_transport()?;

    // Create a TCP connection to the responder node.
    let connection_to_responder = tcp.connect("127.0.0.1:4000", TcpConnectionOptions::new()).await?;

    // Create and start a Relay worker
    node.start_worker("forward_to_responder", Relay::new(connection_to_responder))?;

    // Create a TCP listener and wait for incoming connections.
    let listener = tcp.listen("127.0.0.1:3000", TcpListenerOptions::new()).await?;

    // Allow access to the Relay via TCP connections from the TCP listener
    node.flow_controls()
        .add_consumer(&"forward_to_responder".into(), listener.flow_control_id());

    // Don't call node.shutdown() here so this node runs forever.
    Ok(())
}
