// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The payload module defines the core data types for representing message payloads.

pub mod milestone;
pub mod receipt;
pub mod tagged_data;
pub mod transaction;
pub mod treasury_transaction;

pub use milestone::MilestonePayload;
pub(crate) use milestone::{PublicKeyCount, SignatureCount};
pub use receipt::ReceiptPayload;
pub(crate) use receipt::{MigratedFundsAmount, ReceiptFundsCount};
pub use tagged_data::TaggedDataPayload;
pub(crate) use tagged_data::{TagLength, TaggedDataLength};
pub use transaction::TransactionPayload;
pub(crate) use transaction::{InputCount, OutputCount};
pub use treasury_transaction::TreasuryTransactionPayload;

use crate::Error;

use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable, PackableExt,
};

use alloc::boxed::Box;
use core::ops::Deref;

/// A generic payload that can represent different types defining message payloads.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u32, with_error = Error::InvalidPayloadKind)]
pub enum Payload {
    /// A transaction payload.
    #[packable(tag = TransactionPayload::KIND)]
    Transaction(Box<TransactionPayload>),
    /// A milestone payload.
    #[packable(tag = MilestonePayload::KIND)]
    Milestone(Box<MilestonePayload>),
    /// A receipt payload.
    #[packable(tag = ReceiptPayload::KIND)]
    Receipt(Box<ReceiptPayload>),
    /// A treasury transaction payload.
    #[packable(tag = TreasuryTransactionPayload::KIND)]
    TreasuryTransaction(Box<TreasuryTransactionPayload>),
    /// A tagged data payload.
    #[packable(tag = TaggedDataPayload::KIND)]
    TaggedData(Box<TaggedDataPayload>),
}

impl From<TransactionPayload> for Payload {
    fn from(payload: TransactionPayload) -> Self {
        Self::Transaction(Box::new(payload))
    }
}

impl From<MilestonePayload> for Payload {
    fn from(payload: MilestonePayload) -> Self {
        Self::Milestone(Box::new(payload))
    }
}

impl From<ReceiptPayload> for Payload {
    fn from(payload: ReceiptPayload) -> Self {
        Self::Receipt(Box::new(payload))
    }
}

impl From<TreasuryTransactionPayload> for Payload {
    fn from(payload: TreasuryTransactionPayload) -> Self {
        Self::TreasuryTransaction(Box::new(payload))
    }
}

impl From<TaggedDataPayload> for Payload {
    fn from(payload: TaggedDataPayload) -> Self {
        Self::TaggedData(Box::new(payload))
    }
}

impl Payload {
    /// Returns the payload kind of a `Payload`.
    pub fn kind(&self) -> u32 {
        match self {
            Self::Transaction(_) => TransactionPayload::KIND,
            Self::Milestone(_) => MilestonePayload::KIND,
            Self::Receipt(_) => ReceiptPayload::KIND,
            Self::TreasuryTransaction(_) => TreasuryTransactionPayload::KIND,
            Self::TaggedData(_) => TaggedDataPayload::KIND,
        }
    }
}

/// Representation of an optional [`Payload`].
/// Essentially an `Option<Payload>` with a different [`Packable`] implementation, to conform to specs.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct OptionalPayload(Option<Payload>);

impl OptionalPayload {
    fn pack_ref<P: Packer>(payload: &Payload, packer: &mut P) -> Result<(), P::Error> {
        (payload.packed_len() as u32).pack(packer)?;
        payload.pack(packer)
    }
}

impl Deref for OptionalPayload {
    type Target = Option<Payload>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Packable for OptionalPayload {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        match &self.0 {
            None => 0u32.pack(packer),
            Some(payload) => Self::pack_ref(payload, packer),
        }
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let len = u32::unpack::<_, VERIFY>(unpacker).infallible()? as usize;

        if len > 0 {
            unpacker.ensure_bytes(len)?;

            let payload = Payload::unpack::<_, VERIFY>(unpacker)?;
            let actual_len = payload.packed_len();

            if len != actual_len {
                Err(UnpackError::Packable(Error::InvalidPayloadLength {
                    expected: len,
                    actual: actual_len,
                }))
            } else {
                Ok(Self(Some(payload)))
            }
        } else {
            Ok(Self(None))
        }
    }
}

// FIXME: does this break any invariant about the Payload length?
impl From<Option<Payload>> for OptionalPayload {
    fn from(option: Option<Payload>) -> Self {
        Self(option)
    }
}

#[allow(clippy::from_over_into)]
impl Into<Option<Payload>> for OptionalPayload {
    fn into(self) -> Option<Payload> {
        self.0
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    pub use super::{
        milestone::dto::MilestonePayloadDto, receipt::dto::ReceiptPayloadDto, tagged_data::dto::TaggedDataPayloadDto,
        transaction::dto::TransactionPayloadDto, treasury_transaction::dto::TreasuryTransactionPayloadDto,
    };
    use crate::error::dto::DtoError;

    /// Describes all the different payload types.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum PayloadDto {
        Transaction(Box<TransactionPayloadDto>),
        Milestone(Box<MilestonePayloadDto>),
        Receipt(Box<ReceiptPayloadDto>),
        TreasuryTransaction(Box<TreasuryTransactionPayloadDto>),
        TaggedData(Box<TaggedDataPayloadDto>),
    }

    impl From<&Payload> for PayloadDto {
        fn from(value: &Payload) -> Self {
            match value {
                Payload::Transaction(p) => PayloadDto::Transaction(Box::new(TransactionPayloadDto::from(p.as_ref()))),
                Payload::Milestone(p) => PayloadDto::Milestone(Box::new(MilestonePayloadDto::from(p.as_ref()))),
                Payload::Receipt(p) => PayloadDto::Receipt(Box::new(ReceiptPayloadDto::from(p.as_ref()))),
                Payload::TreasuryTransaction(p) => {
                    PayloadDto::TreasuryTransaction(Box::new(TreasuryTransactionPayloadDto::from(p.as_ref())))
                }
                Payload::TaggedData(p) => PayloadDto::TaggedData(Box::new(TaggedDataPayloadDto::from(p.as_ref()))),
            }
        }
    }

    impl TryFrom<&PayloadDto> for Payload {
        type Error = DtoError;
        fn try_from(value: &PayloadDto) -> Result<Self, Self::Error> {
            Ok(match value {
                PayloadDto::Transaction(p) => Payload::Transaction(Box::new(TransactionPayload::try_from(p.as_ref())?)),
                PayloadDto::Milestone(p) => Payload::Milestone(Box::new(MilestonePayload::try_from(p.as_ref())?)),
                PayloadDto::Receipt(p) => Payload::Receipt(Box::new(ReceiptPayload::try_from(p.as_ref())?)),
                PayloadDto::TreasuryTransaction(p) => {
                    Payload::TreasuryTransaction(Box::new(TreasuryTransactionPayload::try_from(p.as_ref())?))
                }
                PayloadDto::TaggedData(p) => Payload::TaggedData(Box::new(TaggedDataPayload::try_from(p.as_ref())?)),
            })
        }
    }
}
