use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Merkle proof does not match the campaign root")]
    InvalidMerkleProof,
}
