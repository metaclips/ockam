use ockam::identity::SecureChannelListenerOptions;
use ockam::tcp::{TcpListenerOptions, TcpOutletOptions, TcpTransportExtension};
use ockam::{node, Context, Result};

#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    // Initialize the TCP Transport.
    let node = node(ctx).await?;
    let tcp = node.create_tcp_transport()?;

    // Create:
    //   1. An Identity to represent this Node
    //   2. A Secure Channel Listener at Worker address - secure_channel_listener
    //      that will wait for requests to start an Authenticated Key Exchange.
    let e = node.create_identity().await?;

    let tcp_listener_options = TcpListenerOptions::new();

    let secure_channel_listener_options =
        SecureChannelListenerOptions::new().as_consumer(&tcp_listener_options.spawner_flow_control_id());
    let secure_channel_flow_control_id = secure_channel_listener_options.spawner_flow_control_id();
    node.create_secure_channel_listener(&e, "secure_channel_listener", secure_channel_listener_options)?;

    // Expect first command line argument to be the TCP address of a target TCP server.
    // For example: 127.0.0.1:4002
    //
    // Create a TCP Transport Outlet - at Ockam Worker address "outlet" -
    // that will connect, as a TCP client, to the target TCP server.
    //
    // This Outlet will:
    // 1. Unwrap the payload of any Ockam Routing Message that it receives from an Inlet
    //    and send it as raw TCP data to the target TCP server. First such message from
    //    an Inlet is used to remember the route back the Inlet.
    //
    // 2. Wrap any raw TCP data it receives, from the target TCP server,
    //    as payload of a new Ockam Routing Message. This Ockam Routing Message will have
    //    its onward_route be set to the route to an Inlet that is knows about because of
    //    a previous message from the Inlet.

    let outlet_target = std::env::args().nth(1).expect("no outlet target given");
    tcp.create_outlet(
        "outlet",
        outlet_target.try_into()?,
        TcpOutletOptions::new().as_consumer(&secure_channel_flow_control_id),
    )?;

    // Create a TCP listener to receive Ockam Routing Messages from other ockam nodes.
    //
    // Use port 4000, unless otherwise specified by second command line argument.

    let port = std::env::args().nth(2).unwrap_or_else(|| "4000".to_string());
    tcp.listen(format!("127.0.0.1:{port}"), tcp_listener_options).await?;

    // We won't call ctx.shutdown_node() here,
    // so this program will keep running until you interrupt it with Ctrl-C.
    Ok(())
}
