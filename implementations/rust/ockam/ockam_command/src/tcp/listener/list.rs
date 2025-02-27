use clap::Args;
use colorful::Colorful;
use tokio::sync::Mutex;
use tokio::try_join;

use ockam::Context;
use ockam_api::colors::OckamColor;
use ockam_api::nodes::models::transport::TransportStatus;
use ockam_api::nodes::BackgroundNodeClient;

use crate::node::NodeOpts;
use crate::{docs, CommandGlobalOpts};

use crate::util::api;

const PREVIEW_TAG: &str = include_str!("../../static/preview_tag.txt");
const AFTER_LONG_HELP: &str = include_str!("./static/list/after_long_help.txt");

/// List TCP listeners
#[derive(Args, Clone, Debug)]
#[command(
before_help = docs::before_help(PREVIEW_TAG),
after_long_help = docs::after_help(AFTER_LONG_HELP))]
pub struct ListCommand {
    #[command(flatten)]
    node_opts: NodeOpts,
}

impl ListCommand {
    pub fn name(&self) -> String {
        "tcp-listener list".into()
    }

    pub async fn run(&self, ctx: &Context, opts: CommandGlobalOpts) -> miette::Result<()> {
        let node = BackgroundNodeClient::create(ctx, &opts.state, &self.node_opts.at_node).await?;
        let is_finished: Mutex<bool> = Mutex::new(false);

        let get_transports = async {
            let transports: Vec<TransportStatus> = node.ask(ctx, api::list_tcp_listeners()).await?;
            *is_finished.lock().await = true;
            Ok(transports)
        };

        let output_messages = vec![format!(
            "Listing TCP Listeners on {}...\n",
            node.node_name().color(OckamColor::PrimaryResource.color())
        )];

        let progress_output = opts.terminal.loop_messages(&output_messages, &is_finished);

        let (transports, _) = try_join!(get_transports, progress_output)?;

        let list = opts.terminal.build_list(
            &transports,
            &format!(
                "No TCP Listeners found on {}",
                node.node_name().color(OckamColor::PrimaryResource.color())
            ),
        )?;
        opts.terminal.stdout().plain(list).write_line()?;
        Ok(())
    }
}
