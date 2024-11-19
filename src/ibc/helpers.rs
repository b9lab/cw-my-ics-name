use sha2::{Digest, Sha256};

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub fn compute_voucher_token_id(
    channel_id: &String,
    collection: &String,
    token_id: &String,
) -> String {
    let voucher_id = format!("transfer_name/ibc/{channel_id}/{collection}/{token_id}");
    bytes_to_hex(&Sha256::digest(voucher_id.as_bytes()))
}
