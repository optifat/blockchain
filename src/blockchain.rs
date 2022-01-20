use crate::block::Block;
use chrono::Utc;
use std::io;

pub struct Blockchain {
    pub blocks: Vec<Block>,
}

pub const DIFFICULTY_PREFIX: &'static str = "00";

impl Blockchain{

    pub fn new() -> Self{
        Self{
            blocks: Vec::new(),
        }
    }

    pub fn genesis(&mut self){
        let genesis_block = Block {
            id: 0,
            timestamp: Utc::now().timestamp(),
            previous_hash: None,
            data: String::from("genesis!"),
            nonce: 2836,
            hash: "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43".to_string(),
        };

        self.blocks.push(genesis_block);
    }

    pub fn try_add_block(&mut self, block: Block) -> Result<(), io::Error>{
        let latest_block = self.blocks.last().expect("there is at least one block");
        if let Err(error) = self.block_is_valid(&block, latest_block) {
            return Err(error);
        } 
        else {
            self.blocks.push(block);
            return Ok(())
        }
    }
    
    pub fn block_is_valid(&self, block: &Block, previous_block: &Block) -> Result<(), io::Error>{
        if block.previous_hash.is_none(){
            return Err(io::Error::new(io::ErrorKind::Other, "This is genesis block"));
        }
        
        if *(block.previous_hash.as_ref().unwrap()) != previous_block.hash {
            return Err(io::Error::new(io::ErrorKind::Other, format!("block with id: {} has wrong previous hash", block.id)));
        } 
        else if !crate::hash::hash_to_binary_representation(
            &hex::decode(&block.hash).expect("can decode from hex")
        )
        .starts_with(DIFFICULTY_PREFIX){
            return Err(io::Error::new(io::ErrorKind::Other, format!("block with id: {} has invalid difficulty", block.id)));
        } 
        else if block.id != previous_block.id + 1 {
            return Err(io::Error::new(io::ErrorKind::Other, format!(
                "block with id: {} is not the next block after the latest: {}",
                block.id, previous_block.id
            )));
            
        } 
        else if hex::encode(block.calculate_block_hash()) != block.hash{
            return Err(io::Error::new(io::ErrorKind::Other, format!("block with id: {} has invalid hash", block.id)));
        }
        Ok(())
    }

    pub fn chain_is_valid(&self, chain: &[Block]) -> Result<(), io::Error> {
        for i in 0..chain.len() {
            if i == 0 {
                continue;
            }
            let first = chain.get(i - 1).expect("index out of range");
            let second = chain.get(i).expect("index out of range");
            if let Err(error) = self.block_is_valid(second, first) {
                return Err(io::Error::new(io::ErrorKind::Other, error));
            }
        }
        Ok(())
    }

    pub fn choose_chain(&mut self, local: Vec<Block>, remote: Vec<Block>) -> Vec<Block> {
        let local_is_valid = {
            match self.chain_is_valid(&local){
                Ok(_) => true,
                Err(_) => false,
            }
        };
        let remote_is_valid = {
            match self.chain_is_valid(&remote){
                Ok(_) => true,
                Err(_) => false,
            }
        };

        if local_is_valid && remote_is_valid {
            if local.len() > remote.len() {
                return local;
            } 
            else if local.len() < remote.len(){
                return remote;
            }
            else if local.iter().last().unwrap().timestamp < remote.iter().last().unwrap().timestamp{
                return local;
            }
            else{
                return remote;
            }
        } 
        else if remote_is_valid && !local_is_valid {
            return remote;
        } 
        else if !remote_is_valid && local_is_valid {
            return local;
        } 
        else {
            panic!("local and remote chains are both invalid");
        }
    }
}