// Copyright 2019 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// TODO
//! pending transfer.

use exonum::crypto::{Hash, PublicKey};

use super::proto;

/// Wallet information stored in the database.
#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::PendingTransfer", serde_pb_convert)]
pub struct PendingTransfer {
    /// TODO
    pub tx_hash: Hash,
    /// TODO
    pub from: PublicKey,
    /// TODO    
    pub to: PublicKey,
    /// TODO    
    pub amount: u64,
    /// TODO    
    pub fulfilled: bool,
}

impl PendingTransfer {
    /// Create new PendingTransfer.
    pub fn new(
        tx_hash: Hash,
        &from: &PublicKey,
        &to: &PublicKey,
        amount: u64,
        fulfilled: bool,
    ) -> Self {
        Self {
            tx_hash,
            from,
            to,
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
