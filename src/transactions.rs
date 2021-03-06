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

//! Cryptocurrency transactions.

// Workaround for `failure` see https://github.com/rust-lang-nursery/failure/issues/223 and
// ECR-1771 for the details.
#![allow(bare_trait_objects)]

use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction, TransactionContext},
    crypto::{Hash, PublicKey, SecretKey},
    messages::{Message, RawTransaction, Signed},
};

use super::proto;
use crate::{schema::Schema, CRYPTOCURRENCY_SERVICE_ID};

const ERROR_SENDER_SAME_AS_RECEIVER: u8 = 0;

const ERROR_THIRD_PARTY_SAME_AS_SENDER_OR_RECEIVER: u8 = 1;

const ERROR_UNEXPECTED_THIRD_PARTY: u8 = 2;

/// Error codes emitted by wallet transactions during execution.
#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    /// Wallet already exists.
    ///
    /// Can be emitted by `CreateWallet`.
    #[fail(display = "Wallet already exists")]
    WalletAlreadyExists = 0,

    /// Sender doesn't exist.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Sender doesn't exist")]
    SenderNotFound = 1,

    /// Receiver doesn't exist.
    ///
    /// Can be emitted by `Transfer` or `Issue`.
    #[fail(display = "Receiver doesn't exist")]
    ReceiverNotFound = 2,

    /// Insufficient currency amount.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Insufficient currency amount")]
    InsufficientCurrencyAmount = 3,

    /// Pending transfer not found
    ///
    /// Can be emitted by `ConfirmTransfer`
    #[fail(display = "Pending transfer doesn't exist")]    
    PendingTransferNotFound = 4,

    /// Pending transfer is already fulfilled
    ///
    /// Can be emitted by `ConfirmTransfer`
    #[fail(display = "Pending transfer has already been fulfilled")]    
    PendingTransferAlreadyFulfilled = 5,

    /// Approver doesn't exist.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Approver doesn't exist")]
    ApproverNotFound = 6,    
}

impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}

/// Transfer `amount` of the currency from one wallet to another.
#[derive(Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::Transfer", serde_pb_convert)]
pub struct Transfer {
    /// `PublicKey` of receiver's wallet.
    pub to: PublicKey,
    /// `PublicKey` of approver's wallet.
    pub approver: PublicKey,
    /// Amount of currency to transfer.
    pub amount: u64,
    /// Auxiliary number to guarantee [non-idempotence][idempotence] of transactions.
    ///
    /// [idempotence]: https://en.wikipedia.org/wiki/Idempotence
    pub seed: u64,
}

/// Issue `amount` of the currency to the `wallet`.
#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::Issue")]
pub struct Issue {
    /// Issued amount of currency.
    pub amount: u64,
    /// Auxiliary number to guarantee [non-idempotence][idempotence] of transactions.
    ///
    /// [idempotence]: https://en.wikipedia.org/wiki/Idempotence
    pub seed: u64,
}

/// Create wallet with the given `name`.
#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::CreateWallet")]
pub struct CreateWallet {
    /// Name of the new wallet.
    pub name: String,
}


/// Confirm pending transfer transaction with the given `tx_hash`
#[derive(Serialize, Deserialize, Clone, Debug, ProtobufConvert)]
#[exonum(pb = "proto::ConfirmTransfer")]
pub struct ConfirmTransfer {
    /// Hash of the transaction to be confirmed
    pub tx_hash: Hash,
    /// [idempotence]: https://en.wikipedia.org/wiki/Idempotence    
    pub seed: u64,
}

/// Transaction group.
#[derive(Serialize, Deserialize, Clone, Debug, TransactionSet)]
pub enum WalletTransactions {
    /// Transfer tx.
    Transfer(Transfer),
    /// Issue tx.
    Issue(Issue),
    /// CreateWallet tx.
    CreateWallet(CreateWallet),
    /// ConfirmTransfer tx.
    ConfirmTransfer(ConfirmTransfer),
}

impl ConfirmTransfer {
    #[doc(hidden)]
    pub fn sign(
        pk: &PublicKey, &tx_hash: &Hash, seed: u64, sk: &SecretKey
    ) -> Signed<RawTransaction> {
        Message::sign_transaction(
            Self {
                tx_hash,
                seed,
            },
            CRYPTOCURRENCY_SERVICE_ID,
            *pk,
            sk
        )
    }
}

impl CreateWallet {
    #[doc(hidden)]
    pub fn sign(name: &str, pk: &PublicKey, sk: &SecretKey) -> Signed<RawTransaction> {
        Message::sign_transaction(
            Self {
                name: name.to_owned(),
            },
            CRYPTOCURRENCY_SERVICE_ID,
            *pk,
            sk,
        )
    }
}

impl Transfer {
    #[doc(hidden)]
    pub fn sign(
        pk: &PublicKey,
        &to: &PublicKey,
        &approver: &PublicKey,
        amount: u64,
        seed: u64,
        sk: &SecretKey,
    ) -> Signed<RawTransaction> {
        Message::sign_transaction(
            Self { to, amount, approver, seed },
            CRYPTOCURRENCY_SERVICE_ID,
            *pk,
            sk,
        )
    }
}

impl Transaction for Transfer {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let from = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        let to = &self.to;
        let approver = &self.approver;
        let amount = self.amount;

        if from == approver || to == approver {
            return Err(ExecutionError::new(ERROR_THIRD_PARTY_SAME_AS_SENDER_OR_RECEIVER))
        }

        if from == to {
            return Err(ExecutionError::new(ERROR_SENDER_SAME_AS_RECEIVER));
        }

        schema.wallet(approver).ok_or(Error::ApproverNotFound)?;
        let sender = schema.wallet(from).ok_or(Error::SenderNotFound)?;

        schema.wallet(to).ok_or(Error::ReceiverNotFound)?;

        // considering frozen_amount to still be awailable for withdrawal,
        // since it can be left unconfirmed
        if sender.balance + (sender.frozen_amount as i64) < (amount as i64) {
            Err(Error::InsufficientCurrencyAmount)?
        }

        schema.decrease_wallet_balance(sender, amount, &hash);
        schema.create_pending_transfer(hash, from, to, approver, amount);

        Ok(())
    }
}

/// checking if withdrawal can be confirmed.
/// handling withdrawal amount which is greater than frozen amount
/// and also the situation when the frozen balance is greater than initial balance
pub fn can_confirm_withdrawal(balance: i64, frozen: u64, amount: u64) -> bool {
    frozen >= amount && (frozen as i64) + balance >= (amount as i64)
}

impl Transaction for ConfirmTransfer {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let approver = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        if let Some(pending_transfer) = schema.pending_transfer(&self.tx_hash) {
            if pending_transfer.approver != *approver {
                return Err(ExecutionError::new(ERROR_UNEXPECTED_THIRD_PARTY))
            }
            
            if pending_transfer.fulfilled {
                Err(Error::PendingTransferAlreadyFulfilled)?
            }
            
            let from = &pending_transfer.from;
            let to = &pending_transfer.to;

            let sender = schema.wallet(from).ok_or(Error::SenderNotFound)?;
            let receiver = schema.wallet(to).ok_or(Error::ReceiverNotFound)?;

            let amount = pending_transfer.amount;

            if !can_confirm_withdrawal(sender.balance, sender.frozen_amount, amount) {
                Err(Error::InsufficientCurrencyAmount)?                
            }

            schema.decrease_wallet_frozen_balance(sender, amount, &hash);
            schema.increase_wallet_balance(receiver, amount, &hash);
            schema.fulfill_pending_transfer(pending_transfer);
            
            Ok(())
        } else {
            Err(Error::PendingTransferNotFound)?
        }
    }    
}

impl Transaction for Issue {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pub_key = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        if let Some(wallet) = schema.wallet(pub_key) {
            let amount = self.amount;

            schema.increase_wallet_balance(wallet, amount, &hash);
            Ok(())
        } else {
            Err(Error::ReceiverNotFound)?
        }
    }
}

impl Transaction for CreateWallet {
    fn execute(&self, mut context: TransactionContext) -> ExecutionResult {
        let pub_key = &context.author();
        let hash = context.tx_hash();

        let mut schema = Schema::new(context.fork());

        if schema.wallet(pub_key).is_none() {
            let name = &self.name;
            schema.create_wallet(pub_key, name, &hash);
            Ok(())
        } else {
            Err(Error::WalletAlreadyExists)?
        }
    }
}
