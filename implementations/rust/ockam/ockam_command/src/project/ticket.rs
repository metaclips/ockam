use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;
use clap::Args;
use colorful::Colorful;
use miette::{miette, IntoDiagnostic};
use tracing::debug;

use crate::shared_args::{IdentityOpts, RetryOpts, TrustOpts};
use crate::util::parsers::{duration_parser, duration_to_human_format};
use crate::{docs, Command, CommandGlobalOpts, Error, Result};
use ockam::Context;
use ockam_api::authenticator::direct::{
    OCKAM_ROLE_ATTRIBUTE_ENROLLER_VALUE, OCKAM_ROLE_ATTRIBUTE_KEY, OCKAM_TLS_ATTRIBUTE_KEY,
};
use ockam_api::authenticator::enrollment_tokens::{
    TokenIssuer, DEFAULT_TOKEN_DURATION, DEFAULT_TOKEN_USAGE_COUNT, MAX_RECOMMENDED_TOKEN_DURATION,
    MAX_RECOMMENDED_TOKEN_USAGE_COUNT,
};
use ockam_api::cli_state::{ExportedEnrollmentTicket, ProjectRoute};
use ockam_api::colors::color_primary;
use ockam_api::nodes::InMemoryNode;
use ockam_api::terminal::fmt;
use ockam_api::{fmt_info, fmt_log, fmt_ok, fmt_warn};
use ockam_multiaddr::MultiAddr;

const LONG_ABOUT: &str = include_str!("./static/ticket/long_about.txt");
const AFTER_LONG_HELP: &str = include_str!("./static/ticket/after_long_help.txt");

/// This attribute in credential allows member to create a relay on the Project node, the name of the relay should be
/// equal to the value of that attribute. If the value is `*` then any name is allowed
pub const OCKAM_RELAY_ATTRIBUTE: &str = "ockam-relay";

/// Add members to a Project, as an authorized enroller, directly, or via an enrollment ticket
#[derive(Clone, Debug, Args)]
#[command(
long_about = docs::about(LONG_ABOUT),
after_long_help = docs::after_help(AFTER_LONG_HELP),
)]
pub struct TicketCommand {
    /// Orchestrator address to resolve projects present in the `at` argument
    #[command(flatten)]
    identity_opts: IdentityOpts,

    #[command(flatten)]
    trust_opts: TrustOpts,

    /// Attributes in `key=value` format to be attached to the member. You can specify this option multiple times for multiple attributes
    #[arg(short, long = "attribute", value_name = "ATTRIBUTE")]
    attributes: Vec<String>,

    /// Duration for which the enrollment ticket is valid, if you don't specify this, the default is 10 minutes. Examples: 10000ms, 600s, 600, 10m, 1h, 1d. If you don't specify a length sigil, it is assumed to be seconds
    #[arg(long = "expires-in", value_name = "DURATION", value_parser = duration_parser)]
    expires_in: Option<Duration>,

    /// Number of times the ticket can be used to enroll, the default is 1
    #[arg(long = "usage-count", value_name = "USAGE_COUNT")]
    usage_count: Option<u64>,

    /// Name of the relay that the identity using the ticket will be allowed to create. This name is transformed into attributes to prevent collisions when creating relay names. For example: `--relay foo` is shorthand for `--attribute ockam-relay=foo`
    #[arg(long = "relay", value_name = "ENROLLEE_ALLOWED_RELAY_NAME")]
    allowed_relay_name: Option<String>,

    /// Add the enroller role to your ticket. If you specify it, this flag is transformed into the attributes `--attribute ockam-role=enroller`. This role allows the Identity using the ticket to enroll other Identities into the Project, typically something that only admins can do
    #[arg(long = "enroller")]
    enroller: bool,

    /// Allows the access to the TLS certificate of the Project, this flag is transformed into the attributes `--attribute ockam-tls-certificate=true`
    #[arg(long = "tls", hide = true)]
    tls: bool,

    #[command(flatten)]
    retry_opts: RetryOpts,

    /// Return the ticket in hex encoded format
    #[arg(long = "hex", hide = true)]
    hex_encoded: bool,

    /// Return the ticket using the legacy encoding format
    #[arg(long, hide = true)]
    legacy: bool,
}

#[async_trait]
impl Command for TicketCommand {
    const NAME: &'static str = "project ticket";

    async fn run(self, ctx: &Context, opts: CommandGlobalOpts) -> Result<()> {
        let cmd = self.parse_args(&opts).await?;
        let identity = opts
            .state
            .get_identity_name_or_default(&cmd.identity_opts.identity_name)
            .await?;

        let node = InMemoryNode::start_with_project_name(
            ctx,
            &opts.state,
            cmd.trust_opts.project_name.clone(),
        )
        .await?;

        let project = opts
            .state
            .projects()
            .get_project_by_name_or_default(&cmd.trust_opts.project_name)
            .await?;

        let authority_node_client = node
            .create_authority_client_with_project(ctx, &project, Some(identity))
            .await?;

        let attributes = cmd.attributes()?;
        debug!(attributes = ?attributes, "Attributes passed");

        // Request an enrollment token that a future member can use to get a
        // credential.
        let token = {
            let pb = opts.terminal.spinner();
            if let Some(pb) = pb.as_ref() {
                pb.set_message("Creating an enrollment ticket...");
            }
            authority_node_client
                .create_token(ctx, attributes.clone(), cmd.expires_in, cmd.usage_count)
                .await
                .map_err(Error::Retry)?
        };
        let project = project.model();
        let ticket = ExportedEnrollmentTicket::new(
            token,
            ProjectRoute::new(MultiAddr::from_str(&project.access_route)?)?,
            project
                .identity
                .clone()
                .ok_or(miette!("missing project's identity"))?,
            &project.name,
            project
                .project_change_history
                .as_ref()
                .ok_or(miette!("missing project's change history"))?,
            project
                .authority_identity
                .as_ref()
                .ok_or(miette!("missing authority's change history"))?,
            MultiAddr::from_str(
                project
                    .authority_access_route
                    .as_ref()
                    .ok_or(miette!("missing authority's route"))?,
            )?,
        )
        .import()
        .await?;
        let (as_json, encoded_ticket) = if cmd.legacy {
            let exported = ticket.export_legacy()?;
            (
                serde_json::to_string(&exported).into_diagnostic()?,
                exported.hex_encoded()?,
            )
        } else {
            let exported = ticket.export()?;
            let encoded = if cmd.hex_encoded {
                exported.hex_encoded()?
            } else {
                exported.to_string()
            };
            (serde_json::to_string(&exported).into_diagnostic()?, encoded)
        };

        let usage_count = cmd.usage_count.unwrap_or(DEFAULT_TOKEN_USAGE_COUNT);
        let attributes_msg = if attributes.is_empty() {
            "".to_string()
        } else {
            let mut attributes_msg =
                fmt_log!("The redeemer will be assigned the following attributes:\n");
            let mut attributes: Vec<_> = attributes.iter().collect();
            attributes.sort();
            for (key, value) in &attributes {
                attributes_msg += &fmt_log!(
                    "{}{}",
                    fmt::INDENTATION,
                    color_primary(format!("\"{key}={value}\"\n"))
                );
            }
            attributes_msg += "\n";
            attributes_msg
        };
        opts.terminal.write_line(
            fmt_ok!("Created enrollment ticket\n\n")
                + &attributes_msg
                + &fmt_info!(
                    "It will expire in {} and it can be used {}\n",
                    color_primary(duration_to_human_format(
                        &cmd.expires_in.unwrap_or(DEFAULT_TOKEN_DURATION)
                    )),
                    if usage_count == 1 {
                        color_primary("once").to_string()
                    } else {
                        format!("up to {} times", color_primary(usage_count))
                    }
                )
                + &fmt_log!(
                    "You can use it to enroll another machine using: {}",
                    color_primary("ockam project enroll")
                ),
        )?;

        opts.terminal
            .stdout()
            .plain(format!("\n{encoded_ticket}"))
            .machine(encoded_ticket)
            .json(as_json)
            .write_line()?;

        Ok(())
    }
}

impl TicketCommand {
    async fn parse_args(self, opts: &CommandGlobalOpts) -> miette::Result<Self> {
        // Handle expires_in and usage_count limits
        if let Some(usage_count) = self.usage_count {
            if usage_count < 1 {
                return Err(miette!("The usage count must be at least 1"));
            }
        }
        if let (Some(expires_in), Some(usage_count)) = (self.expires_in, self.usage_count) {
            if expires_in >= MAX_RECOMMENDED_TOKEN_DURATION
                && usage_count >= MAX_RECOMMENDED_TOKEN_USAGE_COUNT
            {
                opts.terminal.write_line(
                    fmt_warn!(
                        "You are creating a ticket with a long expiration time and a high usage count\n"
                    ) + &fmt_log!(
                        "This is a security risk. Please consider reducing the values according to the ticket's intended use\n"
                    ),
                )?;
            }
        }
        Ok(self)
    }

    fn attributes(&self) -> Result<BTreeMap<String, String>> {
        let mut attributes = BTreeMap::new();
        for attr in &self.attributes {
            let mut parts = attr.splitn(2, '=');
            let key = parts.next().ok_or(miette!("key expected"))?;
            // If no value is provided we assume that the attribute is a boolean attribute set to "true"
            let value = parts.next().unwrap_or("true");
            attributes.insert(key.to_string(), value.to_string());
        }
        if let Some(relay_name) = self.allowed_relay_name.clone() {
            attributes.insert(OCKAM_RELAY_ATTRIBUTE.to_string(), relay_name);
        }
        if self.enroller {
            attributes.insert(
                OCKAM_ROLE_ATTRIBUTE_KEY.to_string(),
                OCKAM_ROLE_ATTRIBUTE_ENROLLER_VALUE.to_string(),
            );
        }

        if self.tls {
            attributes.insert(OCKAM_TLS_ATTRIBUTE_KEY.to_string(), "true".to_string());
        }

        Ok(attributes)
    }
}
