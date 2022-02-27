use serde::{Serialize,Deserialize};
use ring::signature::{Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use bincode::serialize;
use crate::crypto::hash::{self,H256, Hashable};
use crate::crypto::address::{self, H160};
use ring::digest;
use crate::crypto::key_pair;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use ring::signature::{UnparsedPublicKey, ED25519};
use bincode;

#[allow(non_snake_case)]

#[derive(Serialize, Deserialize, Debug, Default,Clone, Eq, PartialEq, Hash)]
pub struct UtxoInput{
    pub prev_hash: H256,
    pub index: u8,
}
#[derive(Serialize, Deserialize, Debug, Clone,Copy)]
pub struct UtxoOutput{
    pub recipient_address: H160,
    pub value: u32,
}

#[derive(Serialize, Deserialize, Default, Debug,Clone)]
pub struct Transaction {
    //Imagine we have input and output and also with a signature in the transaction
    pub tx_input: Vec<UtxoInput>,
    pub tx_output: Vec<UtxoOutput>,
}

//We set clone here because we found in the followint assignment we have to use it as .clone() function
//This SignTransaction we set here is for following project, I have no idea of how to use it now, the types of items inside of
//it is from Readme file.
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct SignTransaction{
    pub transaction: Transaction,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>
}

// we inplement the hashable function for Transaction structure, and it should be still working in the following project.
impl Hashable for Transaction{
    fn hash(&self) -> H256 {
        //we serialize the transaction firstly and then make it the the format we want
        let serialized_t: Vec<u8> = bincode::serialize(&self).unwrap();
        return digest::digest(&ring::digest::SHA256, &serialized_t[..]).into();
    }
}

/* we inplement the hashable function for SignTransaction structure, and it should be still working in the following project.*/
impl Hashable for SignTransaction{
    fn hash(&self) -> H256 {
        let serialized_st: Vec<u8> = bincode::serialize(&self).unwrap();
        return digest::digest(&ring::digest::SHA256, &serialized_st[..]).into();
    }
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    let message_bytes = serialize(&t).unwrap();//serialize it first
    let sig = key.sign(&message_bytes[..]);//sign the key and generate keypair
    //make   t.signature = true;
    return sig;
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, signature_bytes: &Vec<u8>,public_key_bytes: &Vec<u8>) -> bool {
    let t_bytes: Vec<u8> = serialize(&t).unwrap();
    //those following are I changed from the ring website, and actually I did not understand how it works, but I think this is the right process
    //let peer_public_key_bytes = public_key.as_ref();
    let peer_public_key =ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key_bytes);
    let result = peer_public_key.verify(&t_bytes[..],signature_bytes);//the result need to check
    return result.is_ok();
}

pub fn generate_random_transaction() -> Transaction {
    let input = vec![UtxoInput{prev_hash: hash::generate_random_hash(), index: 0}];
    let output = vec![UtxoOutput{recipient_address: address::generate_random_address(), value: 0}];
    
    Transaction{tx_input: input, tx_output: output}
}

pub fn generate_genesis_transaction() -> Transaction {
    let input = vec![UtxoInput{prev_hash: H256::from([0;32]), index: 0}];
    let output = vec![UtxoOutput{recipient_address: H160::from([0;20]), value: 0}];
    
    Transaction{tx_input: input, tx_output: output}
}

pub fn generate_random_signed_transaction() -> SignTransaction {
    let t = generate_random_transaction();
    let key = key_pair::random();
    let sig = sign(&t, &key);
    let signed_tx = SignTransaction{transaction:t,
                                      signature:sig.as_ref().to_vec(),
                                      public_key:key.public_key().as_ref().to_vec()};
    signed_tx
}

pub fn generate_genesis_signed_transaction() -> SignTransaction {

    let t = generate_genesis_transaction();
    let key = Ed25519KeyPair::from_pkcs8([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 187, 131, 74, 161, 134, 11, 240, 6, 188, 109, 18, 108, 124, 219, 167, 164, 215, 125, 168, 79, 204, 194, 232, 91, 58, 186, 181, 230, 212, 78, 163, 28, 161, 35, 3, 33, 0, 233, 72, 146, 218, 220, 235, 17, 123, 202, 112, 119, 63, 134, 105, 134, 71, 34, 185, 71, 193, 59, 66, 43, 137, 50, 194, 120, 234, 97, 132, 235, 159].as_ref().into()).unwrap();
    let sig = sign(&t, &key);
    let signed_tx = SignTransaction{transaction:t,signature:sig.as_ref().to_vec(),public_key:key.public_key().as_ref().to_vec()};
    signed_tx
}

#[cfg(any(test, test_utilities))]
mod tests {
    use super::*;
    use crate::crypto::key_pair;
    

    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, &signature.as_ref().to_vec(), &key.public_key().as_ref().to_vec()));
    }
}
