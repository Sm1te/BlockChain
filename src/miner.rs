use crate::network::server::Handle as ServerHandle;
use crate::transaction::{self,Transaction, SignTransaction};
use crate::crypto::hash::{H256, Hashable};
use crate::crypto::merkle::{MerkleTree};
use crate::block::{Block, Header, Content};
use crate::blockchain::Blockchain;
use rand::Rng;
use crate::network::message::Message;
use std::sync::{Arc, Mutex};
use bincode;
//use log::{debug, info};
use log::{info,debug};


use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;

use std::thread;

enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    num_mined:u8,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    server: &ServerHandle,
    blockchain: &Arc<Mutex<Blockchain>>,
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
        num_mined:0,
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                info!("Miner shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i) => {
                info!("Miner starting in continuous mode with lambda {}", i);
                self.operating_state = OperatingState::Run(i);
            }
        }
    }

    fn miner_loop(&mut self) {
        let time_0 = time::Instant::now();
        let mut next_time = 0.01;
        let next_time_inc = 0.01;

        // main mining loop
        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    self.handle_control_signal(signal);
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        self.handle_control_signal(signal);
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // TODO: actual mining
            //let mut blockchain_1 = self.blockchain.lock().unwrap();

            let parent = self.blockchain.lock().unwrap().tip();
            let timestamp = time::SystemTime::now().duration_since(time::SystemTime::UNIX_EPOCH).unwrap().as_millis();
            let difficulty = self.blockchain.lock().unwrap().chain[&parent].Header.difficulty;
            
            //Creating Content
            //It will also be used for Merkel Root for the Header
            let t = transaction::generate_random_signed_transaction();
            let mut vect: Vec<SignTransaction> = vec![];
            vect.push(t);
            let content: Content = Content{content:vect};

            let mut rng = rand::thread_rng();
            let nonce: u32 = rng.gen();
            let merkle_root = H256::from([0; 32]);
            let header = Header{parent,nonce,difficulty,timestamp,merkleRoot:merkle_root};

            let new_block = Block{Header: header,Content: content};
        
          
            //Check whether block solved the puzzle
            //If passed, add it to blockchain
            if new_block.hash() <= difficulty {
                println!("block with hash:{} generated\n",new_block.hash());
                //println!("Number of blocks mined until now:{}\n",self.num_mined+1);
                self.blockchain.lock().unwrap().insert(&new_block);
                let encodedhead: Vec<u8> = bincode::serialize(&new_block).unwrap();
                debug!("Size of block generated is {} bytes\n",encodedhead.len());
                print!("Total number of blocks in blockchain:{}\n",self.blockchain.lock().unwrap().chain.len());
                self.num_mined = self.num_mined +1;
                let time_1 = time::Instant::now();
                println!("mined {}, hash = {:?}, {:?}", self.num_mined+1, new_block.hash(), time_1.checked_duration_since(time_0));
                let mut new_block_hash : Vec<H256> = vec![];
                new_block_hash.push(new_block.hash());
                self.server.broadcast(Message::NewBlockHashes(new_block_hash)); 
            }
            let mut length = self.blockchain.lock().unwrap().chain.keys().len();
            println!("Number of all the blocks mined until now:{:?}\n", length);
            

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
        }
    }
}

