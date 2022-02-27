use crate::transaction::{self, SignTransaction};
use crate::block::Block;
use crate::crypto::address;
use crate::ledger::State;

use log::debug;

pub fn is_tx_valid(signed_tx: &SignTransaction) -> bool {
   //verify whether the tx is signed properly
   return transaction::verify(&signed_tx.transaction, &signed_tx.signature, &signed_tx.public_key);
}

pub fn is_blck_valid(block: &Block, parent_state: &State) -> bool {
    for signed_tx in &block.Content.content {
      debug!("current signed_tx {:?}", signed_tx);
       if !is_tx_valid(signed_tx){
          debug!("tx didn't pass signature check!");
          return false;
       }

       //Couple of checks
       //1. Owner match
       //2. Input/Output total match
       //3. Double Spend
       let owner_address = address::address_from_public_key_vec_ref(&signed_tx.public_key);
       let mut total_input_value = 0;
       for input in &signed_tx.transaction.tx_input {
           debug!("current tx_input {:?}", input);
           if !parent_state.state_map.contains_key(&input){
              debug!("tx is double spend as input is not there in State!");
              return false;  
           }
           let output = &parent_state.state_map[&input];
           if output.recipient_address != owner_address {
              debug!("owner of tx input doesn't match to previous tx output");
              debug!("input addreess {:?}", owner_address);
              debug!("output address {:?}", output.recipient_address);
              return false;
           }
           total_input_value = output.value;
       }
       
       let mut total_output_value = 0;
       for output in &signed_tx.transaction.tx_output {
            total_output_value += output.value;
       }

       if total_input_value != total_output_value {
          debug!("Input sum didn't match to output sum for tx");
          return false;
       }
    }

    true
}