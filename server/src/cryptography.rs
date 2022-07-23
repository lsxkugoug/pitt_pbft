use std::fs;
use std::fs::File;
use std::io::Read;
use crate::{config, message, constants};
use hex;
use p256::ecdsa::{SigningKey, VerifyingKey};
use p256::EncodedPoint;
use rand_core::OsRng;
use serde::{Serialize, Deserialize};
use p256::ecdsa::signature::{Signer, Signature, Verifier};

// generate client and servers key pair, and store them in key_pairs
pub fn generate_key_pairs() {
    // check key_pairs folder, wether it is clean, if not error, note: "./" path means project root path
    let paths = fs::read_dir("./key_pairs").unwrap();


    for path in paths {
        // println!("Name: {}", path.unwrap().path().display())
        panic!("Before generate key pair, please guarantee key_pair folder is empty");
    }
    // create related folder
    fs::create_dir("./key_pairs/client").expect("failed to create ./key_pairs/client");
    fs::create_dir("./key_pairs/server").expect("failed to create ./key_pairs/server");

    // generate clients' key pair, based on client number
    for i in 0..config::CLIENT_NUM {
        let folder_path = format!("./key_pairs/client/{}", &i);
        fs::create_dir(&folder_path).expect("failed to create ./key_pairs/client");
        let pri_k = SigningKey::random(&mut OsRng);
        let pub_k = VerifyingKey::from(&pri_k);
        let pub_k = VerifyingKey::to_encoded_point(&pub_k, false).to_bytes();
        let pri_k_path = format!("{}{}",&folder_path, "/pri_key");
        let pub_k_path = format!("{}{}",&folder_path, "/pub_key");
        // write private key and public key
        fs::write(pri_k_path, pri_k.to_bytes()).unwrap();
        fs::write(pub_k_path, pub_k).unwrap();
    }

    // generate servers' key pair, based on server number
    for i in 0..config::SERVER_NUM {
        let folder_path = format!("./key_pairs/server/{}", &i);
        fs::create_dir(&folder_path).expect("failed to create ./key_pairs/server");
        let pri_k = SigningKey::random(&mut OsRng);
        let pub_k = VerifyingKey::from(&pri_k);
        let pub_k = VerifyingKey::to_encoded_point(&pub_k, false).to_bytes();
        let pri_k_path = format!("{}{}",&folder_path, "/pri_key");
        let pub_k_path = format!("{}{}",&folder_path, "/pub_key");
        // write private key and public key
        fs::write(pri_k_path, pri_k.to_bytes()).unwrap();
        fs::write(pub_k_path, pub_k).unwrap();
    }

}

// note: The keys is a static vec, inited by fn constants::init_constants()
pub fn load_private_key(path: String) -> SigningKey {
    let mut f = File::open(&path).expect("no file found");
    let metadata = fs::metadata(&path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    SigningKey::from_bytes(&buffer).expect(&format!("read private key: {} wrong", &path))
}

pub fn load_public_key(path: String) -> VerifyingKey{
    let mut f = File::open(&path).expect("no file found");
    let metadata = fs::metadata(&path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    let point = EncodedPoint::from_bytes(&buffer).expect(&format!("read public key: {} wrong", &path));
    VerifyingKey::from_encoded_point(&point).expect(&format!("read public key: {} wrong", &path))
}

pub fn sign_msg(private_key: &SigningKey, content: &[u8]) -> Vec<u8> {
    let sig_ret = Signer::sign(private_key, content);
    return Vec::<u8>::from(sig_ret.as_ref())
}

pub fn verify_sig(public_key: &VerifyingKey, msg: &[u8], signed_value: &[u8]) -> bool {

    let sig_instance = match Signature::from_bytes(signed_value) {
        Ok(sig_instance) => sig_instance,
        Err(_) => {
            print!("failed convert bytes stream to signature instance");
            return false
        },
    };

    // VerifyingKey::verify(&self, msg, signature)
    if let Err(e) = public_key.verify(msg, &sig_instance) {
        println!("{}", e);
        return false
    }

    true

}


#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use crate::{message, constants, cryptography::{sign_msg, verify_sig, load_private_key}};

    #[test]
    fn generate_key_pairs_test() {
        super::generate_key_pairs();
    }

    // #[test]
    // fn sign_verify_test() {
    //     // simulate I am 0 server, receive client0's client request
    //     constants::init_constants(0);

    //     // 1. simulate client0 generate request and sign it.
    //     use std::time::{Duration, SystemTime};
    //     // firstly convert a object to bytes then sign it
    //     let send_msg = message::Client_msg{
    //         msg_type: constants::CLIENT_REQUEST,
    //         who_send: 0,
    //         operation: "this is operation".to_string(),
    //         time_stamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
    //     };
    //     let msg_bytes = bincode::serialize(&send_msg).unwrap();
    //     let signature = sign_msg(&load_private_key("/key_pairs/client/0/pri_key".to_string()), &msg_bytes);
    //     let result = verify_sig(constants::get_client_pub(0), &msg_bytes, &signature);
        

    // }
}