mod constants;
use crate::constants::*;
use anyhow::*;
use std::borrow::Borrow;

#[derive(Clone)]
pub enum Trie {
    Empty,
    Leaf {
        hash: Vec<u8>,
        prefix: Vec<u8>,
        key: Vec<u8>,
        value: Vec<u8>,
    },
    Branch {
        hash: Vec<u8>,
        prefix: Vec<u8>,
        children: Vec<Option<Trie>>,
        size: usize,
    },
}

fn hash<T>(_data: T) -> Vec<u8>
    where T: Borrow<Vec<u8>> {
    // TODO
    vec![0x0; DIGEST_LENGTH]
}

fn to_nibbles(data: Vec<u8>) -> Vec<u8> {
    data.iter().flat_map(|byte| vec![byte >> 4, byte & 0x0f]).collect()
}

fn to_sparse_vec(data: Vec<(u8, Trie)>) -> Vec<Option<Trie>> {
    let mut result = vec![None; 16];
    for (index, value) in data {
        result[index as usize] = Some(value);
    }
    result
}

fn leaf_hash(prefix_nibbles: Vec<u8>, value: Vec<u8>) -> Result<Vec<u8>> {
    let is_even = prefix_nibbles.len() % 2 == 0;
    let head = if is_even { vec![0xff] } else { [vec![0x00], to_nibbles(prefix_nibbles[0..1].to_vec())].concat() };
    let tail = if is_even { prefix_nibbles } else { prefix_nibbles[1..].to_vec() };
    if value.len() != DIGEST_LENGTH {
        bail!("value must be a {}-byte digest, but it is {}", DIGEST_LENGTH, hex::encode(value));
    }
    return Ok(hash([head, tail, value].concat()))
}

fn branch_hash(prefix_nibbles: &Vec<u8>, root: &Vec<u8>) -> Result<Vec<u8>> {
    if root.len() != DIGEST_LENGTH {
        bail!("root must be a {}-byte digest, but it is {}", DIGEST_LENGTH, hex::encode(root));
    }

    return Ok(hash([prefix_nibbles.clone(), root.clone()].concat()));
}

fn merkle_root(_children: &Vec<Option<Trie>>) -> Vec<u8> {
    return vec![0x0; DIGEST_LENGTH]; // TODO
}

fn common_prefix(words: Vec<&Vec<u8>>) -> Vec<u8> {
    let mut prefix: Vec<u8> = vec![];
    for (i, word) in words.iter().enumerate() {
        if i == 0 {
            prefix = word.to_vec();
            continue;
        }
        let mut new_prefix: Vec<u8> = vec![];
        for (a, b) in prefix.iter().zip(word.iter()) {
            if a != b {
                break;
            }
            new_prefix.push(*a);
        }
        prefix = new_prefix.clone();
    }
    prefix
}

impl Trie {
    pub fn new() -> Self {
        Trie::Empty
    }
    pub fn leaf(suffix: Vec<u8>, key: Vec<u8>, value: Vec<u8>) -> Result<Self> {
        if hash(&key).strip_suffix(&suffix[..]).is_none() {
            bail!("key doesn't end in suffix");
        }
        Ok(Trie::Leaf {
            hash: leaf_hash(suffix.clone(), hash(&value))?,
            prefix: suffix,
            key,
            value,
        })
    }
    pub fn branch(prefix: Vec<u8>, children: Vec<Option<Trie>>) -> Result<Self> {
        if children.iter().filter(|i| i.is_some()).count() == 1 {
            bail!("Branch must have at *at least 2* children. A Branch with a single child is a Leaf.");
        }

        if children.len() != 16 {
            bail!("Branch must have exactly 16 children, but it has {}", children.len());
        }

        let size = children.iter().map(|i| i.as_ref().map_or(0, |t| t.size())).sum();
        Ok(Trie::Branch {
            hash: branch_hash(&prefix, &merkle_root(&children))?,
            prefix,
            children,
            size,
        })
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Trie::Empty => true,
            _ => false,
        }
    }

    pub fn hash(&self) -> Vec<u8> {
        match self {
            Trie::Empty => NULL_HASH.to_vec(), // TODO: copyless?
            Trie::Leaf { hash, .. } => hash.clone(),
            Trie::Branch { hash, .. } => hash.clone(),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Trie::Empty => 0usize,
            Trie::Leaf { .. } => 1usize,
            Trie::Branch { size, .. } => *size,
        }
    }

    pub fn insert(self, key: Vec<u8>, value: Vec<u8>) -> Result<Self> {
        match self {
            Trie::Empty => Trie::leaf(to_nibbles(key.clone()), key, value),
            Trie::Leaf { prefix, key, value, .. } => {
                let full_path = to_nibbles(key.clone());
                let this_path = prefix.clone();
                let new_path = full_path[full_path.len() - this_path.len()..].to_vec();
                if prefix == new_path {
                    bail!("key {} already in trie", hex::encode(key));
                }
                let common_prefix = common_prefix(vec![&prefix, &new_path]);
                let this_nibble = common_prefix[prefix.len()];
                let new_nibble = new_path[prefix.len()];

                if this_nibble == new_nibble {
                    bail!("bug in common_prefix");
                }

                let prefix_start = prefix.len() + 1;
                Trie::branch(common_prefix, to_sparse_vec(vec![
                    (this_nibble, Trie::leaf(prefix[prefix_start..].to_vec(), key.clone(), value.clone())?),
                    (new_nibble, Trie::leaf(new_path[prefix_start..].to_vec(), key, value)?),
                ]))
            },
            Trie::Branch { .. } => todo!(),
        }
    }
}