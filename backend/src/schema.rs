use exonum::{
    crypto::{Hash, PublicKey, CryptoHash}, storage::{Fork, ProofListIndex, ProofMapIndex, Snapshot},
};

use INITIAL_BID;

encoding_struct! {
    struct AuctionItem {
        item_id: u64,
        name: &str,
        owner_pub_key: &PublicKey,
        customer_pub_key: &PublicKey,
        bid: u64,
        history_len: u64,
        history_hash: &Hash,
    }
}

impl AuctionItem {
    pub fn raise_bid(self, new_bid: u64, customer_pub_key: &PublicKey, history_hash: &Hash) -> Self {
        debug_assert!(new_bid > self.bid());
        Self::new(
            self.item_id(),
            self.name(),
            self.owner_pub_key(),
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

    pub fn items(&self) -> ProofMapIndex<&T, Hash, AuctionItem> {
        ProofMapIndex::new("auction.items", &self.view)
    }

    pub fn item_history(&self, item_id: u64) -> ProofListIndex<&T, Hash> {
        ProofListIndex::new_in_family("auction.item_history", &item_id, &self.view)
    }

    pub fn item(&self, item_id: u64) -> Option<AuctionItem> {
        self.items().get(&item_id.hash())
    }

    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.items().merkle_root()]
    }
}

impl<'a> Schema<&'a mut Fork> {
    pub fn items_mut(&mut self) -> ProofMapIndex<&mut Fork, Hash, AuctionItem> {
        ProofMapIndex::new("auction.items", &mut self.view)
    }

    pub fn item_history_mut(&mut self, item_id: u64) -> ProofListIndex<&mut Fork, Hash> {
        ProofListIndex::new_in_family("auction.item_history", &item_id, &mut self.view)
    }

    pub fn raise_bid(&mut self, item: AuctionItem, bid: u64, customer_key: &PublicKey, transaction: &Hash) {
        let item = {
            let mut history = self.item_history_mut(item.item_id());
            history.push(*transaction);
            let history_hash = history.merkle_root();
            item.raise_bid(bid, customer_key, &history_hash)
        };
        self.items_mut().put(&item.item_id().hash(), item.clone());
    }

    pub fn create_item(&mut self, item_id: u64, name: &str, owner_key: &PublicKey, transaction: &Hash) {
        let item = {
            let mut history = self.item_history_mut(item_id);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            AuctionItem::new(item_id, name, owner_key, owner_key, INITIAL_BID, history.len(), &history_hash)
        };
        self.items_mut().put(&item_id.hash(), item);
    }
}
