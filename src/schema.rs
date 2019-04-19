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

//! Cryptocurrency database schema.

use exonum::{
    crypto::{Hash, PublicKey},
    storage::{Fork, ProofListIndex, ProofMapIndex, Snapshot},
};

use crate::{wallet::Wallet, INITIAL_BALANCE, pending_transfer::PendingTransfer};

/// Database schema for the cryptocurrency.
#[derive(Debug)]
pub struct Schema<T> {
    view: T,
}

impl<T> AsMut<T> for Schema<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.view
    }
}

impl<T> Schema<T>
where
    T: AsRef<dyn Snapshot>,
{
    /// Creates a new schema from the database view.
    pub fn new(view: T) -> Self {
        Schema { view }
    }

    /// Returns `ProofMapIndex` with wallets.
    pub fn wallets(&self) -> ProofMapIndex<&T, PublicKey, Wallet> {
        ProofMapIndex::new("cryptocurrency.wallets", &self.view)
    }

    /// Returns history of the wallet with the given public key.
    pub fn wallet_history(&self, public_key: &PublicKey) -> ProofListIndex<&T, Hash> {
        ProofListIndex::new_in_family("cryptocurrency.wallet_history", public_key, &self.view)
    }

    /// Returns `ProofMapIndex` with pending transfers.
    pub fn pending_transfers(&self) -> ProofMapIndex<&T, Hash, PendingTransfer> {
        ProofMapIndex::new("cryptocurrency.pending_transfers", &self.view)
    }

    /// Returns pending transfer for the transfer transaction hash.
    pub fn pending_transfer(&self, hash: &Hash) -> Option<PendingTransfer> {
        self.pending_transfers().get(hash)
    }

    /// Returns wallet for the given public key.
    pub fn wallet(&self, pub_key: &PublicKey) -> Option<Wallet> {
        self.wallets().get(pub_key)
    }

    /// Returns the state hash of cryptocurrency service.
    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.wallets().merkle_root()]
    }
}

/// Implementation of mutable methods.
impl<'a> Schema<&'a mut Fork> {
    /// Returns mutable `ProofMapIndex` with wallets.
    pub fn wallets_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, Wallet> {
        ProofMapIndex::new("cryptocurrency.wallets", &mut self.view)
    }

    /// Returns mutable `ProofMapIndex` with pending transfers.
    pub fn pending_transfers_mut(&mut self) -> ProofMapIndex<&mut Fork, Hash, PendingTransfer> {
        ProofMapIndex::new("cryptocurrency.pending_transfers", self.view)
    }

    /// Returns history for the wallet by the given public key.
    pub fn wallet_history_mut(
        &mut self,
        public_key: &PublicKey,
    ) -> ProofListIndex<&mut Fork, Hash> {
        ProofListIndex::new_in_family("cryptocurrency.wallet_history", public_key, &mut self.view)
    }

    /// Increase balance of the wallet and append new record to its history.
    ///
    /// Panics if there is no wallet with given public key.
    pub fn increase_wallet_balance(&mut self, wallet: Wallet, amount: u64, transaction: &Hash) {
        let wallet = {
            let mut history = self.wallet_history_mut(&wallet.pub_key);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            let balance = wallet.balance;
            wallet.set_balance(balance + amount, &history_hash)
        };
        self.wallets_mut().put(&wallet.pub_key, wallet.clone());
    }

    /// Decrease balance of the wallet and append new record to its history.
    ///
    /// Panics if there is no wallet with given public key.
    pub fn decrease_wallet_balance(&mut self, wallet: Wallet, amount: u64, transaction: &Hash) {
        let wallet = {
            let mut history = self.wallet_history_mut(&wallet.pub_key);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            let balance = wallet.balance;
            let frozen = wallet.frozen_amount;
            wallet
                .set_balance(balance - amount, &history_hash)
                .set_frozen_amount(frozen + amount, &history_hash)
        };
        self.wallets_mut().put(&wallet.pub_key, wallet.clone());
    }

    /// Decrease frozen balance of the wallet and append new record to its history.
    ///
    /// Panics if there is no wallet with given public key.
    pub fn decrease_wallet_frozen_balance(&mut self, wallet: Wallet, amount: u64, transaction: &Hash) {
        let wallet = {
            let mut history = self.wallet_history_mut(&wallet.pub_key);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            let frozen = wallet.frozen_amount;
            wallet.set_frozen_amount(frozen - amount, &history_hash)
        };
        self.wallets_mut().put(&wallet.pub_key, wallet.clone());
    }
    
    /// Fulfill pending transfer
    pub fn fulfill_pending_transfer(&mut self, transfer: PendingTransfer) {
        let fulfilled_transfer = transfer.set_fulfilled();
        
        self.pending_transfers_mut().put(&fulfilled_transfer.tx_hash, fulfilled_transfer.clone());
    }

    /// Create new pending transfer
    pub fn create_pending_transfer(&mut self, tx_hash: Hash, from: &PublicKey, to: &PublicKey, amount: u64) {
        self.pending_transfers_mut().put(&tx_hash, PendingTransfer::new(tx_hash, from, to, amount, false));
    }

    /// Create new wallet and append first record to its history.
    pub fn create_wallet(&mut self, key: &PublicKey, name: &str, transaction: &Hash) {
        let wallet = {
            let mut history = self.wallet_history_mut(key);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            Wallet::new(key, name, INITIAL_BALANCE, history.len(), &history_hash, 0)
        };
        self.wallets_mut().put(key, wallet);
    }
}
