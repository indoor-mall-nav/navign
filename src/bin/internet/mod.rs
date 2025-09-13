use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use embedded_nal::{AddrType, Dns};
use esp_hal::rng::Rng;
use esp_hal::sha::Digest;
use heapless::{String};
use p256::ecdsa::signature::Signer;
use sha2::Sha256;
use p256::ecdsa::{Signature, SigningKey};
use serde::{Deserialize, Serialize};
use crate::crypto::Nonce;
use crate::shared::constants::NONCE_LENGTH;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestPayload<'a> {
    nonce: &'a str,
    device_signature: &'a str,
}

/// It is the only request directly to the server, all other requests are done via BLE to the device.
///
/// Steps:
/// 1. Generate a random nonce and sign it with the device's private key.
/// 2. Send the nonce and signature to the server.
/// 3. The server verifies the signature with the (stored) device's private key.
/// 4. If the signature is valid, the server loads the beacon's status and return a timestamp.
fn connect_with_server(rng: &mut Rng, device_private_key: SigningKey, beacon_id: &str) {
    let nonce = Nonce::generate(rng);
    let mut nonce_buffer = [0u8; {NONCE_LENGTH * 2}];
    hex::encode_to_slice(nonce.as_bytes(), &mut nonce_buffer).unwrap();
    let nonce_hex = unsafe {
        core::str::from_utf8_unchecked(&nonce_buffer)
    };

    let mut hash = Sha256::new();
    hash.update(nonce.as_bytes());
    let challenge_hash = hash.finalize();

    let device_signature: Signature = device_private_key.sign(&challenge_hash);
    let mut signature_buffer = [0u8; 128];
    hex::encode_to_slice(device_signature.to_bytes(), &mut signature_buffer).unwrap();
    // Soundness: The output is always valid UTF-8 since it's hex-encoded, so
    // we could safely use `from_utf8_unchecked` here to avoid the overhead of
    // checking it again.
    let signature_hex = unsafe {
        core::str::from_utf8_unchecked(&signature_buffer)
    };

    let payload = RequestPayload {
        nonce: nonce_hex,
        device_signature: signature_hex,
    };

    let json_payload = serde_json_core::to_string::<_, { 33 + NONCE_LENGTH * 2 + 128 }>(&payload).unwrap();

    let mut url = String::<64>::new();
    url.push_str("https://navign.7086cmd.me/api/beacon/").unwrap();
    url.push_str(beacon_id).unwrap();

    // Now request to the server with `url` and `json_payload`.

}

pub struct ManualDns;

impl Dns for ManualDns {
    type Error = ();

    fn get_host_by_name(&mut self, host: &str, r#type: AddrType) -> nb::Result<IpAddr, Self::Error> {
        // Here you would implement your DNS resolution logic.
        // For demonstration purposes, we'll return a fixed IP address.
        if host == "navign.7086cmd.me" {
            if r#type == AddrType::IPv4 || r#type == AddrType::Either {
                Ok(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
            } else {
                Ok(IpAddr::V6(Ipv6Addr::LOCALHOST))
            }
        } else {
            Err(nb::Error::Other(()))
        }
    }

    fn get_host_by_address(&mut self, _addr: IpAddr, _result: &mut [u8]) -> nb::Result<usize, Self::Error> {
        // Reverse DNS lookup is not implemented in this example.
        Err(nb::Error::Other(()))
    }
}