#[ockam::test(timeout = 1000)]
async fn my_test(ctx: &mut ockam_node::Context) -> ockam_core::Result<()> {
    ctx.shutdown_node().await
}

fn main() {}
