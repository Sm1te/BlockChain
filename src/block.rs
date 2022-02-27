//#[cfg(test)]
//#[macro_use]
use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256, Hashable};
use crate::crypto::merkle::{MerkleTree};
use crate::transaction::{self,Transaction, SignTransaction, generate_random_signed_transaction};


extern crate chrono;
use chrono::prelude::*;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use ring::{digest};

use std::time::{Duration, SystemTime, UNIX_EPOCH};



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub parent: H256,
    pub nonce: u32,
    pub difficulty: H256,
    pub timestamp: u128,
    pub merkleRoot: H256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    pub content: Vec<SignTransaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
	pub Header: Header,
    pub Content: Content,
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        let header_serialized = bincode::serialize(&self.Header).unwrap();
        return ring::digest::digest(&ring::digest::SHA256, &header_serialized).into();
    }
}


impl Block{
    pub fn getparent(&self) -> H256 {
        self.Header.parent
    }

    pub fn getdifficulty(&self) -> H256 {
        self.Header.difficulty
    }

    pub fn gettimestamp(&self) -> u128 {
        self.Header.timestamp
    }
}


// The difficulty here should also be modified if it is modified in transaction generator
pub fn generate_random_block_(parent: &H256) -> Block {
    let mut nonce:u32 = thread_rng().gen();
    let mut timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis();
    let mut bytes32 = [255u8;32];
    bytes32[0]=1;
    bytes32[1]=1;
    let difficulty : H256 = bytes32.into();
    let mut transaction = Vec::<SignTransaction>::new();
    transaction.push(generate_random_signed_transaction());
    let mut MerkleTree = MerkleTree::new(&transaction);



    let newHeader = Header{
        parent: *parent,
        nonce: nonce,
        difficulty: difficulty,
        timestamp: timestamp,
        merkleRoot: MerkleTree.root(),
    };

    let newContent = Content{
        content: transaction,
    };

    let newBlock = Block{
        Header: newHeader,
        Content: newContent,
    };

    return newBlock;
}

pub fn generate_genesis_block(parent: &H256) -> Block {
    //let b:H256 = hex!("00011718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920").into();
    let random = digest::digest(&ring::digest::SHA256,"00011718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920".as_bytes());
    let b = <H256>::from(random);
    let r1:u32 = 0;
    let r2:u128 = 0;
    //let local: DateTime<Local> = Local::now();
    let h:Header = Header{parent:*parent,nonce:r1,difficulty:b,timestamp:r2,merkleRoot:b};
    let t = transaction::generate_genesis_signed_transaction();
    //transaction::pr();
    let mut vect:Vec<SignTransaction> = vec![];
    vect.push(t);
    let c:Content = Content{content:vect};
    let b:Block = Block{Header:h,Content:c};
    b
}


