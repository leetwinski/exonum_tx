//! pending transfer.

use exonum::crypto::{Hash, PublicKey};

use super::proto;

/// Wallet information stored in the database.
#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::PendingTransfer", serde_pb_convert)]
pub struct PendingTransfer {
    /// hash of the transfer to be confirmed
    pub tx_hash: Hash,
    /// sender public key
    pub from: PublicKey,
    /// receiver public key
    pub to: PublicKey,
    /// approver public key
    pub approver: PublicKey,    
    /// transferred amount
    pub amount: u64,
    /// transfer is fulfilled
    pub fulfilled: bool,
}

impl PendingTransfer {
    /// Create new PendingTransfer.
    pub fn new(
        tx_hash: Hash,
        &from: &PublicKey,
        &to: &PublicKey,
        &approver: &PublicKey,
        amount: u64,
        fulfilled: bool,
    ) -> Self {
        Self {
            tx_hash,
            from,
            to,
            approver,
            amount,
            fulfilled,
        }
    }
    /// Returns a copy of this pending transfer with fulfilled flag set.
    pub fn set_fulfilled(self) -> Self {
        Self {
            fulfilled: true,
            ..self
        }
    }
}
