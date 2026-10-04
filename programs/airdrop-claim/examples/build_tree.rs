//! Builds a merkle allocation tree from `address,amount` CSV (base units)
//! using the exact code the on-chain program verifies against.
//!
//! Usage: cargo run --example build_tree -- allocations.csv [proofs.json]
//!
//! Writes allocations.json: { root_base58, root_hex, allocations: [{ address,
//! amount, proof: [<hex32>...] }] }. Sum check is left to the caller — fund
//! the campaign vault with at least the printed total.

use std::{env, fs, process, str::FromStr};

use anchor_lang::prelude::Pubkey;
use airdrop_claim::merkle::{build_tree, leaf_hash};

fn die(msg: &str) -> ! {
    eprintln!("error: {msg}");
    process::exit(1);
}

fn main() {
    let mut args = env::args().skip(1);
    let csv_path = args.next().unwrap_or_else(|| die("usage: build_tree <allocations.csv> [out.json]"));
    let out_path = args.next().unwrap_or_else(|| "proofs.json".to_string());

    let raw = fs::read_to_string(&csv_path).unwrap_or_else(|e| die(&format!("read {csv_path}: {e}")));

    let mut rows: Vec<(Pubkey, u64)> = Vec::new();
    for (lineno, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || (lineno == 0 && line.starts_with("address")) {
            continue;
        }
        let mut cols = line.split(',');
        let (addr, amt) = match (cols.next(), cols.next(), cols.next()) {
            (Some(a), Some(m), None) => (a.trim(), m.trim()),
            _ => die(&format!("line {}: expected `address,amount`", lineno + 1)),
        };
        let pubkey = Pubkey::from_str(addr)
            .unwrap_or_else(|_| die(&format!("line {}: invalid pubkey {addr}", lineno + 1)));
        let amount: u64 = amt
            .parse()
            .unwrap_or_else(|_| die(&format!("line {}: invalid u64 amount {amt}", lineno + 1)));
        if amount == 0 {
            die(&format!("line {}: amount must be > 0", lineno + 1));
        }
        if rows.iter().any(|(p, _)| p == &pubkey) {
            die(&format!("line {}: duplicate address {addr}", lineno + 1));
        }
        rows.push((pubkey, amount));
    }
    if rows.is_empty() {
        die("no allocations found");
    }

    let total: u64 = rows.iter().try_fold(0u64, |acc, (_, a)| acc.checked_add(*a))
        .unwrap_or_else(|| die("allocation sum overflows u64"));
    let leaves: Vec<[u8; 32]> = rows.iter().map(|(p, a)| leaf_hash(p, *a)).collect();
    let (root, proofs) = build_tree(&leaves).expect("non-empty");

    let allocations: Vec<serde_json::Value> = rows
        .iter()
        .zip(&proofs)
        .map(|((pk, amt), proof)| {
            serde_json::json!({
                "address": pk.to_string(),
                "amount": amt,
                "proof": proof.iter().map(|n| hex::encode(n)).collect::<Vec<_>>(),
            })
        })
        .collect();

    let out = serde_json::json!({
        "root_base58": bs58::encode(root).into_string(),
        "root_hex": hex::encode(root),
        "leaf_count": leaves.len(),
        "total_amount": total,
        "allocations": allocations,
    });
    fs::write(&out_path, serde_json::to_string_pretty(&out).unwrap())
        .unwrap_or_else(|e| die(&format!("write {out_path}: {e}")));

    println!("leaves : {}", leaves.len());
    println!("total  : {total} base units (fund the vault with at least this)");
    println!("root   : {}", bs58::encode(root).into_string());
    println!("proofs : {out_path}");
}
