use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use merkle_hash::{Encodable, MerkleTree};

pub struct Fingerprint {
    pub paths: Vec<PathBuf>,
    pub hash_file: PathBuf,
}

impl Fingerprint {
    pub fn compute(&self) -> String {
        hash_dirs(&self.paths[..]).to_hex_string()
    }

    pub fn compute_and_store(&self) {
        let mut file = File::create(&self.hash_file).unwrap();
        file.write_all(self.compute().as_bytes()).unwrap()
    }

    pub fn load(&self) -> Option<String> {
        File::open(&self.hash_file)
            .map_or_else(
                |err| match err.kind() {
                    std::io::ErrorKind::NotFound => None,
                    other_error => panic!(
                        "Error while opening hash file {:?}: {:?}",
                        &self.hash_file, other_error
                    ),
                },
                Some,
            )
            .map(|mut file| {
                let mut hash = String::new();
                file.read_to_string(&mut hash).map(|_| hash).unwrap()
            })
    }

    pub fn stored_matches_actual(&self) -> bool {
        self.load()
            .map(|stored_hash| self.compute() == stored_hash)
            .unwrap_or(false)
    }
}

fn hash_dir<P>(path: P) -> Vec<u8>
where
    P: AsRef<Path>,
{
    let tree = MerkleTree::builder(path.as_ref().as_os_str().to_string_lossy())
        .build()
        .unwrap();
    tree.root.item.hash
}

fn hash_dirs<P>(paths: &[P]) -> Vec<u8>
where
    P: AsRef<Path>,
{
    paths
        .iter()
        .map(hash_dir)
        .reduce(|acc, hash| acc.iter().zip(hash.iter()).map(|(x, y)| x ^ y).collect())
        .expect("at least one path should be provided")
}
