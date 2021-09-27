use crate::{Vault, VaultRequestMessage, VaultResponseMessage, VaultTrait};
use ockam_core::compat::boxed::Box;
use ockam_core::compat::rand::random;
use ockam_core::{async_trait::async_trait, Address, AsyncTryClone, Result, ResultMessage, Route};
use ockam_node::{block_future, Context};
use tracing::debug;
use zeroize::Zeroize;

mod asymmetric_vault;
mod hasher;
mod key_id_vault;
mod secret_vault;
mod signer;
mod symmetric_vault;
mod verifier;

pub use asymmetric_vault::*;
pub use hasher::*;
pub use key_id_vault::*;
pub use secret_vault::*;
pub use signer::*;
pub use symmetric_vault::*;
pub use verifier::*;

/// Vault sync wrapper
pub struct VaultSync {
    ctx: Context,
    vault_worker_address: Address,
}

impl VaultSync {
    pub(crate) async fn send_message(&self, m: VaultRequestMessage) -> Result<()> {
        self.ctx
            .send(Route::new().append(self.vault_worker_address.clone()), m)
            .await
    }

    pub(crate) async fn receive_message(&mut self) -> Result<VaultResponseMessage> {
        self.ctx
            .receive::<ResultMessage<VaultResponseMessage>>()
            .await?
            .take()
            .body()
            .into()
    }
}

#[async_trait]
impl ockam_core::traits::AsyncClone for VaultSync {
    async fn async_clone(&self) -> Self {
        self.async_start_another().await.unwrap()
    }
}

impl Clone for VaultSync {
    fn clone(&self) -> Self {
        self.start_another().unwrap()
    }
}

#[async_trait]
impl AsyncTryClone for VaultSync {
    async fn async_try_clone(&self) -> Result<Self> {
        self.async_start_another().await
    }
}

impl VaultSync {
    /// Start another Vault at the same address.
    pub fn start_another(&self) -> Result<Self> {
        let vault_worker_address = self.vault_worker_address.clone();

        let clone = VaultSync::create_with_worker(&self.ctx, &vault_worker_address)?;

        Ok(clone)
    }

    /// Start another Vault at the same address.
    pub async fn async_start_another(&self) -> Result<Self> {
        let vault_worker_address = self.vault_worker_address.clone();

        let clone = VaultSync::async_create_with_worker(&self.ctx, &vault_worker_address).await?;

        Ok(clone)
    }
}

impl Zeroize for VaultSync {
    fn zeroize(&mut self) {}
}

impl VaultSync {
    /// Create and start a new Vault using Worker.
    pub fn create_with_worker(ctx: &Context, vault: &Address) -> Result<Self> {
        let address: Address = random();

        debug!("Starting VaultSync at {}", &address);

        let ctx = block_future(
            &ctx.runtime(),
            async move { ctx.new_context(address).await },
        )?;

        Ok(Self {
            ctx,
            vault_worker_address: vault.clone(),
        })
    }

    /// Create and start a new Vault using Worker.
    pub async fn async_create_with_worker(ctx: &Context, vault: &Address) -> Result<Self> {
        let address: Address = random();

        debug!("Starting VaultSync at {}", &address);

        let ctx = ctx.new_context(address).await?;

        Ok(Self {
            ctx,
            vault_worker_address: vault.clone(),
        })
    }

    /// Start a Vault.
    pub fn create<T: VaultTrait>(ctx: &Context, vault: T) -> Result<Self> {
        let vault_address = Vault::create_with_inner(ctx, vault)?;

        Self::create_with_worker(ctx, &vault_address)
    }

    /// Return the Vault worker address
    pub fn address(&self) -> Address {
        self.vault_worker_address.clone()
    }
}
