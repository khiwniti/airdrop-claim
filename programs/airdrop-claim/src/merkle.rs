use anchor_lang::prelude::*;
use sha2::{Digest, Sha256};

/// sha256 over concatenated parts (equivalent to solana's hashv).
pub fn sha256_concat(parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().into()
}

/// Leaf: sha256(claimant_pubkey || amount_le).
pub fn leaf_hash(claimant: &Pubkey, amount: u64) -> [u8; 32] {
    sha256_concat(&[claimant.as_ref(), amount.to_le_bytes().as_ref()])
}

/// Sorted-pair (commutative) node hash so proofs are order-agnostic.
pub fn node_hash(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    sha256_concat(&[lo.as_ref(), hi.as_ref()])
}

pub fn verify_proof(root: &[u8; 32], claimant: &Pubkey, amount: u64, proof: &[[u8; 32]]) -> bool {
    let computed = proof
        .iter()
        .fold(leaf_hash(claimant, amount), |node, sibling| {
            node_hash(&node, sibling)
        });
    &computed == root
}

// ---------- off-chain tree construction (host only, never in the BPF build) ----------

/// Builds the tree over `leaves` (CSV order = leaf order) and returns
/// (root, per-leaf proofs). Duplicate-last rule for odd levels, matching
/// `verify_proof`'s commutative hashing. `None` on empty input.
#[cfg(not(target_os = "solana"))]
pub fn build_tree(leaves: &[[u8; 32]]) -> Option<([u8; 32], Vec<Vec<[u8; 32]>>)> {
    if leaves.is_empty() {
        return None;
    }
    let mut levels: Vec<Vec<[u8; 32]>> = vec![leaves.to_vec()];
    while levels.last().unwrap().len() > 1 {
        let prev = levels.last().unwrap();
        let mut next = Vec::with_capacity(prev.len().div_ceil(2));
        for pair in prev.chunks(2) {
            let b = pair.get(1).unwrap_or(&pair[0]);
            next.push(node_hash(&pair[0], b));
        }
        levels.push(next);
    }
    let root = levels.last().unwrap()[0];

    let proofs = (0..leaves.len())
        .map(|start| {
            let mut idx = start;
            let mut proof = Vec::with_capacity(levels.len().saturating_sub(1));
            for level in levels.iter().take(levels.len().saturating_sub(1)) {
                let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
                proof.push(*level.get(sibling_idx).unwrap_or(&level[idx.min(level.len() - 1)]));
                idx /= 2;
            }
            proof
        })
        .collect();
    Some((root, proofs))
}
