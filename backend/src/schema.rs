use exonum::{
    crypto::{Hash, PublicKey}, storage::{Fork, ProofListIndex, ProofMapIndex, Snapshot},
};

use INITIAL_BID;

encoding_struct! {
    struct AuctionItem {
        id: u32,
        name: &str,
        customer_pub_key: &PublicKey,
        bid: u64,
        history_len: u64,
        history_hash: &Hash,
    }
}

impl AuctionItem {
    pub fn raise_bid(self, new_bid: u64, customer_pub_key: &PublicKey, history_hash: &Hash) {
        debug_assert!(new_bid > self.bid());
        Self::new(
            self.id(),
            self.name(),
            customer_pub_key,
            new_bid,
            self.history_len() + 1,
            history_hash,
        )
    }
}

pub struct Schema<T> {
    view: T,
}

impl<T> Schema<T>
where
    T: AsRef<dyn Snapshot>,
{
    pub fn new(view: T) -> Self {
        Schema { view }
    }

    pub fn items(&self) -> ProofMapIndex<&T, PublicKey, AuctionItem> {
        ProofMapIndex::new("auction.items", &self.view)
    }

    pub fn wallet_history(&self, public_key: &PublicKey) -> ProofListIndex<&T, Hash> {
        ProofListIndex::new_in_family("cryptocurrency.wallet_history", public_key, &self.view)
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
