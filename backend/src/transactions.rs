use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction}, crypto::{CryptoHash, PublicKey},
    messages::Message, storage::Fork,
};

use schema::Schema;
use SERVICE_ID;

#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    #[fail(display = "Auction item already exists")]
    ItemAlreadyExists = 0,

    #[fail(display = "Auction item doesn't exist")]
    ItemNotFound = 1,

    #[fail(display = "Bid is too small")]
    SmallBid = 3,
}

impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}

transactions! {
    pub AuctionItemTransactions {
        const SERVICE_ID = SERVICE_ID;

        struct CreateItem {
        	item_id: u64,
        	name: &str,
        	owner_key: &PublicKey,
        }

        struct RaiseBid {
        	item_id: u64,
        	bid: u64,
        	customer_key: &PublicKey,
        }
    }
}

impl Transaction for CreateItem {
    fn verify(&self) -> bool {
        self.verify_signature(self.owner_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = Schema::new(fork);
        let owner_key = self.owner_key();
        let hash = self.hash();
        let item_id = self.item_id();

        if schema.item(item_id).is_none() {
        	schema.create_item(item_id, self.name(), owner_key, &hash);
        	Ok(())
        } else {
        	Err(Error::ItemAlreadyExists)?
        }
    }
}

impl Transaction for RaiseBid {
    fn verify(&self) -> bool {
        self.verify_signature(self.customer_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = Schema::new(fork);
        let customer_key = self.customer_key();
        let hash = self.hash();
        let item_id = self.item_id();

        if let Some(item) = schema.item(item_id) {
            schema.raise_bid(item, self.bid(), customer_key, &hash);
            Ok(())
        } else {
            Err(Error::SmallBid)?
        }
    }
}
