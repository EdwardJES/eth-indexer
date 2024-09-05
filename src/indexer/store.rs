use alloy::rpc::types::Block;
use eyre::{eyre, Result};
use rusqlite::Connection;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug)]
pub struct Store {
    db: Arc<Mutex<Connection>>,
}

impl Store {
    pub fn new(db_path: impl AsRef<Path>) -> Self {
        File::create(db_path).expect("Failed to ceate db file");
        Self {
            db: Arc::new(Mutex::new(
                Connection::open("indexer.db").expect("Failed to open db"),
            )),
        }
    }

    pub fn connection(&mut self) -> MutexGuard<'_, Connection> {
        self.db.lock().expect("Failed to aquire db lock")
    }

    //     tx.execute(
    //         r#"
    //             INSERT INTO deposits (block_number, tx_hash, root_token, child_token, "from", "to", amount)
    //             VALUES (?, ?, ?, ?, ?, ?, ?)
    //             "#,
    //     (
    //         event.block_number,
    //         event.tx_hash.to_string(),
    //         event.root_token.to_string(),
    //         event.child_token.to_string(),
    //         event.from.to_string(),
    //         event.to.to_string(),
    //         event.amount.to_string(),
    //     ),
    // )?;

    pub fn create_tables(&mut self) {
        // TODO populate block fields correctly
        // Likely will use batch to create N tables
        // e.g. other tables go could be transactions
        self.connection()
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS blocks (
                    id               INTEGER PRIMARY KEY,
                    number           INTEGER NOT NULL,
                    hash             TEXT NOT NULL UNIQUE
                );
                "#,
            )
            .expect("Failed to create tables");
    }

    pub fn save_block(&mut self, block: Block) -> Result<()> {
        // TODO we likely wouldn't panic here but log the error and try give as much context
        let block_num = block
            .header
            .number
            .ok_or(eyre!("failed to get block number"))?;
        let hash = block
            .header
            .hash
            .ok_or(eyre!("failed to get block hash"))?;
        let mut conn = self.connection();
        let tx = conn.transaction()?;
        tx.execute(
            r#"
            INSERT INTO blocks (number, hash) 
            VALUES (?, ?) 
            "#,
            (block_num, hash.to_string()),
        )?;
        Ok(())
    }

    // pub fn get_block(&mut self, block_num: u64) -> Result<Block> {
    //     let mut stmt = self
    //         .connection()
    //         .prepare("SELECT * FROM blocks WHERE block_number = ?")?;
    //     let block = stmt.query_row(params![block_num], |row| {
    //         let block = alloy::rpc::types::Block {
    //             header: Default::default(),
    //             transactions: Default::default(),
    //             uncles: Default::default(),
    //         };
    //         Ok(block)
    //     })?;
    //     Ok(block)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::rpc::types::Block;
    use std::path;

    #[test]
    fn test_new_should_create_db_file() {
        let tmp = "./testdata/testdb";
        if std::path::Path::exists(path::Path::new(tmp)) {
            std::fs::remove_file(tmp).expect("Failed to remove file");
        }
        Store::new(tmp);
        assert!(std::path::Path::new(tmp).exists());
        std::fs::remove_file(tmp).expect("Failed to remove file");
    }

    fn create_block() -> Block {
        let s = r#"{
            "baseFeePerGas":"0x886b221ad",
            "blobGasUsed":"0x0",
            "difficulty":"0x0",
            "excessBlobGas":"0x0",
            "extraData":"0x6265617665726275696c642e6f7267",
            "gasLimit":"0x1c9c380",
            "gasUsed":"0xb0033c",
            "hash":"0x85cdcbe36217fd57bf2c33731d8460657a7ce512401f49c9f6392c82a7ccf7ac",
            "logsBloom":"0xc36919406572730518285284f2293101104140c0d42c4a786c892467868a8806f40159d29988002870403902413a1d04321320308da2e845438429e0012a00b419d8ccc8584a1c28f82a415d04eab8a5ae75c00d07761acf233414c08b6d9b571c06156086c70ea5186e9b989b0c2d55c0213c936805cd2ab331589c90194d070c00867549b1e1be14cb24500b0386cd901197c1ef5a00da453234fa48f3003dcaa894e3111c22b80e17f7d4388385a10720cda1140c0400f9e084ca34fc4870fb16b472340a2a6a63115a82522f506c06c2675080508834828c63defd06bc2331b4aa708906a06a560457b114248041e40179ebc05c6846c1e922125982f427",
            "miner":"0x95222290dd7278aa3ddd389cc1e1d165cc4bafe5",
            "mixHash":"0x4c068e902990f21f92a2456fc75c59bec8be03b7f13682b6ebd27da56269beb5",
            "nonce":"0x0000000000000000",
            "number":"0x128c6df",
            "parentBeaconBlockRoot":"0x2843cb9f7d001bd58816a915e685ed96a555c9aeec1217736bd83a96ebd409cc",
            "parentHash":"0x90926e0298d418181bd20c23b332451e35fd7d696b5dcdc5a3a0a6b715f4c717",
            "receiptsRoot":"0xd43aa19ecb03571d1b86d89d9bb980139d32f2f2ba59646cd5c1de9e80c68c90",
            "sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
            "size":"0xdcc3",
            "stateRoot":"0x707875120a7103621fb4131df59904cda39de948dfda9084a1e3da44594d5404",
            "timestamp":"0x65f5f4c3",
            "transactionsRoot":"0x889a1c26dc42ba829dab552b779620feac231cde8a6c79af022bdc605c23a780",
            "withdrawals":[
               {
                  "index":"0x24d80e6",
                  "validatorIndex":"0x8b2b6",
                  "address":"0x7cd1122e8e118b12ece8d25480dfeef230da17ff",
                  "amount":"0x1161f10"

            ],
            "withdrawalsRoot":"0x360c33f20eeed5efbc7d08be46e58f8440af5db503e40908ef3d1eb314856ef7"
         }"#;

        serde_json::from_str::<Block>pect("Failed to generate test block")
    }

    fn with_temp_store<F>(f: F)
    where
        F: FnOnce(Store),
    {
        let tmp = tempfile::NamedTempFile::new().expect("failed to create temp file for test");
        let store = Store::new(&tmp);
        f(store);
    }

    #[test]
    fn test_save_block_should_save_and_return_block() {
        with_temp_store(|mut store| {
            store.create_tables();
            let block = create_block();
            store.save_block(block).expect("Failed to save block");

            // let deposits = store.connection()
            // .prepare(r#"SELECT block_number, tx_hash, root_token, child_token, "from", "to", amount FROM withdrawals"#)?
            // .query_map([], |_| {
            //     Ok(())
            // }).unwrap()
            // .count();
            // assert_eq!(deposits, 0);
        });
    }
}
