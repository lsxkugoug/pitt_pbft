// create key pair based on config
use pbft_engine::cryptography;
fn main() {
    cryptography::generate_key_pairs();
}