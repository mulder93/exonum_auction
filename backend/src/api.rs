use schema::AuctionItem;
use exonum::{
    api::{self, ServiceApiBuilder, ServiceApiState},
    blockchain::{self, BlockProof, Transaction, TransactionSet}, crypto::{Hash, PublicKey},
    helpers::Height, node::TransactionSend, storage::{ListProof, MapProof},
};

use transactions::AuctionItemTransactions;
use {Schema};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AuctionItemQuery {
    pub item_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuctionItemHistory {
    pub transactions: Vec<AuctionItemTransactions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuctionItemInfo {
    pub item_history: Option<AuctionItemHistory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub tx_hash: Hash,
}

pub struct PublicApi;

impl PublicApi {
    pub fn item_info(state: &ServiceApiState, query: AuctionItemQuery) -> api::Result<AuctionItemInfo> {
        let snapshot = state.snapshot();
        let general_schema = blockchain::Schema::new(&snapshot);
        let auction_schema = Schema::new(&snapshot);

        let item = auction_schema.item(query.item_id);

        let item_history = item.map(|_| {
            let history = auction_schema.item_history(query.item_id);

            let transactions: Vec<AuctionItemTransactions> = history
                .iter()
                .map(|record| general_schema.transactions().get(&record).unwrap())
                .map(|raw| AuctionItemTransactions::tx_from_raw(raw).unwrap())
                .collect::<Vec<_>>();

            AuctionItemHistory {
                transactions,
            }
        });

        Ok(AuctionItemInfo {
            item_history,
        })
    }

    pub fn get_items(state: &ServiceApiState, _query: ()) -> api::Result<Vec<AuctionItem>> {
        let snapshot = state.snapshot();
        let schema = Schema::new(snapshot);
        let idx = schema.items();
        let items = idx.values().collect();
        Ok(items)
    }

    pub fn post_transaction(
        state: &ServiceApiState,
        query: AuctionItemTransactions,
    ) -> api::Result<TransactionResponse> {
        let transaction: Box<dyn Transaction> = query.into();
        let tx_hash = transaction.hash();
        state.sender().send(transaction)?;
        Ok(TransactionResponse { tx_hash })
    }

    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder
            .public_scope()
            .endpoint("v1/auction/item_info", Self::item_info)
            .endpoint("v1/auction/items", Self::get_items)
            .endpoint_mut("v1/auction/transaction", Self::post_transaction);
    }
}
