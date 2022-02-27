use super::hash::{Hashable, H256};
use ring::digest;
/// This is my Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    hashes: Vec<H256>,//store tree as arraylist
    leaves: usize,//store how many leaves here instead of how many nodes
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable, {
        let mut array = Vec::new();
        //let mut array_size = Vec::new();
        let mut size = data.len();

        if size!=0 {
            for item in data.iter(){
                array.push(item.hash());
            }//push all the leaves nodes
        
            //for odd number of lowest layer, we choose this way to solve them
            let mut odd = size % 2;// this indicate wether we need to add leaf
            if odd == 1 {
                array.push(array[array.len()-1]);
                size+=1;
            }
    
            let mut base = 0;//start from 0 and will add to the number of next layer
            let mut last_base = size;//the number we want to add to base
            let mut half = size/2;// check which layer we are or whether we come to the root

            while half>0 {
                odd =half%2;//if it is 1, it means that the level now has odd number hashes.
        
                for i in 0..half{
                /*
                Basically this part is used for connect to node to their parent and put the parent into the higher layer
                the ctx = digest::Context::new(&digest::SHA256) was learned by github, I thought it was great and I use it to deal with
                the parent part, I just take it into my code; */
                let mut ctx = digest::Context::new(&digest::SHA256);
                ctx.update(array[base + i*2].as_ref());
                ctx.update(array[base + i*2 + 1].as_ref());
                let hash_two = ctx.finish();
                array.push(hash_two.into());
                }

                if odd == 1 && half != 1 {
                    array.push(array[array.len()-1]);
                    half += 1;//for odd number of upper layer, we choose this way to solve them
                }

                base += last_base;
                last_base = half;//this will be and must be even number
                half /= 2;//update values
            }
        }else {
        array.push([0u8; 32].into());}

        return MerkleTree{hashes: array,leaves:size};
    }


    pub fn root(&self) -> H256 {
        return self.hashes[self.hashes.len()-1];
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
    
        let mut proof: Vec<H256> = Vec::new();
        let mut base = self.leaves;//leaves size
        let mut dividor = 2;//for divisio, since go to upper layer, the number should be divided will increase with power of 2
        let mut sum = 0;//combine with index, indicating how many nodes in array has been skiped
        let mut number = index;//index for find parent node

        while base != 1 {
            //if you have a odd index, its pair must be the index-1, beacause it is order like 0-1, 2-3, 4-5
            //so if it is a even one, it should be index+1 to find its brother
            if number % 2 == 1 {
                proof.push(self.hashes[number-1]);
            }else{
                proof.push(self.hashes[number+1]);
            }
            sum = sum + base;
            number = sum + number / dividor;
            base /= 2;
            if base != 1 && base % 2 ==1{
                base += 1;//we have add all the nodes to make this structure even and this number caculation is easier for us to update values.
            }
            
            
            dividor *= 2;// 2 4 8 16, if we reverse, every time we need to keep this order

        }
        return proof;//That contains the nodes of all brothers and parents' brothers
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    
    let mut verify = false;//check for the result
    let mut data = *datum;//take what we need to verified with and combine it with its brother
    let leaves = leaf_size;

    if leaves > 1{
        for i in 0..proof.len(){
            let hash = <[u8;32]>::from(data);
            let brother = <[u8;32]>::from(proof[i]);
            let parent = [&hash[..],&brother[..]].concat();
            data = ring::digest::digest(&ring::digest::SHA256, &parent[..]).into();
        }//this is a loop used to get the root, and finally we will reach the top of the root with those hashes
    }
        if data == *root{
            verify =true;
        }
        verify
    
    
    

}

#[cfg(test)]
mod tests {
    use crate::crypto::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }

    macro_rules! gen_merkle_tree_assignment2 {
        () => {{
            vec![
                (hex!("0000000000000000000000000000000000000000000000000000000000000011")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000022")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000033")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000044")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000055")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000066")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000077")).into(),
                (hex!("0000000000000000000000000000000000000000000000000000000000000088")).into(),
            ]
        }};
    }

    #[test]
    fn assignment2_merkle_root() {
        let input_data: Vec<H256> = gen_merkle_tree_assignment2!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6e18c8441bc8b0d1f0d4dc442c0d82ff2b4f38e2d7ca487c92e6db435d820a10")).into()
        );
    }
}
