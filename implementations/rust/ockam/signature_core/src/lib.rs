//! A crate for common methods used by short group signatures
#![deny(unsafe_code)]
#![warn(
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate core;

#[cfg(feature = "alloc")]
extern crate alloc;

/// Common methods for signature schemes
#[macro_use]
pub mod util;

/// The errors generated by short group signatures
#[macro_use]
pub mod error;

/// The methods and structs for proving in zero-knowledge
pub mod proof_committed_builder;

/// The constant values across the signatures
pub mod constants;

/// The challenge values
pub mod challenge;

/// Messages that can be signed
pub mod message;

/// Nonce values for zero-knowledge proofs
pub mod nonce;

/// Commitment for proofs and blinded values
pub mod commitment;

/// Indicates which messages are hidden or revealed in proofs
pub mod proof_message;

/// The kind of hidden message in a proof
pub mod hidden_message;

/// The blinding factor when hiding messages during signing
pub mod signature_blinding;

/// A facade around the various collections and primitives needed
/// when using no alloc, alloc only, or std modes
pub mod lib {
    pub use core::cell::{Cell, RefCell};
    pub use core::clone::{self, Clone};
    pub use core::convert::{self, From, Into};
    pub use core::default::{self, Default};
    pub use core::fmt::{self, Debug, Display};
    pub use core::marker::{self, PhantomData};
    pub use core::num::Wrapping;
    pub use core::ops::{Deref, DerefMut, Range};
    pub use core::option::{self, Option};
    pub use core::result::{self, Result};
    pub use core::{cmp, iter, mem, num, slice, str};
    pub use core::{f32, f64};
    pub use core::{i16, i32, i64, i8, isize};
    pub use core::{u16, u32, u64, u8, usize};

    pub use heapless::String;
    pub use heapless::Vec;

    pub use super::challenge::Challenge;
    pub use super::commitment::Commitment;
    pub use super::hidden_message::HiddenMessage;
    pub use super::message::Message;
    pub use super::nonce::Nonce;
    pub use super::proof_committed_builder::ProofCommittedBuilder;
    pub use super::proof_message::ProofMessage;
    pub use super::signature_blinding::SignatureBlinding;
    pub use super::util::{sum_of_products, VecSerializer};
    pub use hashbrown::{HashMap, HashSet};
}
