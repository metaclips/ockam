use ockam::{
    route, Context, Entity, Result, SecureChannels, TcpTransport, TrustEveryonePolicy, Vault, TCP,
};

#[ockam::node]
async fn main(mut ctx: Context) -> Result<()> {
    let vault = Vault::create(&ctx)?;
    let mut robotic_arm = Entity::create(&ctx, &vault)?;

    let tcp = TcpTransport::create(&ctx).await?;

    let channel = robotic_arm.create_secure_channel(
        route![(TCP, "127.0.0.1:4000"), "secure_channel_listener"],
        TrustEveryonePolicy,
    )?;

    tcp.create_outlet("outlet", "127.0.0.1:22").await?;

    ctx.send(route![channel, "inlet_fabric"], "outlet".to_string())
        .await?;

    let port = ctx.receive::<i32>().await?.take().body();
    println!("Inlet is accessible on port {}", port);

    Ok(())
}
