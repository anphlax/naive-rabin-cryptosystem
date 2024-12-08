mod encoding;

use log::{info, warn};
use num_bigint::BigUint;
use num_bigint::ToBigInt;
use num_bigint::ToBigUint;
use num_bigint::{BigInt, RandBigInt};
use num_prime::{PrimalityTestConfig, RandPrime};
use num_traits::{One, Zero};
use rand::thread_rng;
use rayon::prelude::*;
use std::env;

fn main() {
    // Initialize the logger
    env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();
    info!("Hello, Naive Rabin Cryptosystem Implementation...");

    let bits = 512;
    let (n, p, q) = generate_keypair(bits);

    let message = BigInt::from(42u8);
    let ciphertext = encrypt(&message, &n);
    let plaintext_candidates = decrypt(&ciphertext, &p, &q);

    info!("Public key (n): {}", n);
    info!("Message: {}", message);
    info!("Ciphertext: {}", ciphertext);
    info!("Plaintext candidates: {:?}", plaintext_candidates);
}

fn gcd(a: &BigInt, b: &BigInt) -> BigInt {
    if *b == BigInt::zero() {
        a.clone()
    } else {
        // &(a % b) creates a reference to the new BigInt result.
        // This reference is passed to the recursive call instead of moving the value, avoiding unnecessary allocation.
        gcd(b, &(a % b))
    }
}

fn gen_prime(bit_size: usize) -> BigUint {
    let mut rng = thread_rng();
    let config = Some(PrimalityTestConfig::strict());

    // Enforce BigUInt Type, because the PRNG gives only positive numbers (they are prime, lol)
    let mut prime: BigUint;
    loop {
        prime = rng.gen_prime(bit_size, config);
        // Ensure prime ≡ 3 (mod 4)
        if &prime % BigUint::from(4u8) == BigUint::from(3u8) {
            break;
        }
    }
    prime
}

fn generate_keypair(bit_size: usize) -> (BigInt, BigInt, BigInt) {
    info!("Starting key generation with bit size {}", bit_size);

    // Generate two primes in parallel
    let primes: Vec<BigInt> = (0..2)
        .into_par_iter()
        .map(|_| BigInt::from(gen_prime(bit_size)))
        .collect();

    // Assign p and q from the primes vector
    let (p, q) = (primes[0].clone(), primes[1].clone());

    let n = &p * &q; // Compute modulus n
    (n, p, q)
}

fn encrypt(message: &BigInt, n: &BigInt) -> BigInt {
    (message * message) % n
}

fn decrypt(ciphertext: &BigInt, p: &BigInt, q: &BigInt) -> Vec<BigInt> {
    let n = p * q;
    let candidates = compute_candidates(ciphertext, p, q, &n);

    // just return the candidates for now, later we could experiment with padding
    candidates
}

fn compute_candidates(ciphertext: &BigInt, p: &BigInt, q: &BigInt, n: &BigInt) -> Vec<BigInt> {
    // Compute mp = ciphertext^( (p+1)/4 ) mod p
    // This computes one of the square roots of 'ciphertext' modulo 'p'
    let mp = ciphertext.modpow(&((p + BigInt::one()) / BigInt::from(4)), p);
    // Compute mq = ciphertext^( (q+1)/4 ) mod q
    // This computes one of the square roots of 'ciphertext' modulo 'q'
    let mq = ciphertext.modpow(&((q + BigInt::one()) / BigInt::from(4)), q);

    // Log the results for debugging
    log::debug!("mp (mod p): {}", mp);
    log::debug!("mq (mod q): {}", mq);

    // Compute yp = q^(p-2) mod p
    // This is the modular inverse of 'q' modulo 'p' using Fermat's Little Theorem
    let yp = q.modpow(&(p - BigInt::from(2)), p);
    // Compute yq = p^(q-2) mod q
    // This is the modular inverse of 'p' modulo 'q'
    let yq = p.modpow(&(q - BigInt::from(2)), q);

    // Log the modular inverses
    log::debug!("yp (modular inverse of q mod p): {}", yp);
    log::debug!("yq (modular inverse of p mod q): {}", yq);

    // Combine results using the Chinese Remainder Theorem (CRT):
    // Compute one possible candidate solution r1
    let r1 = (&yp * q * &mp + &yq * p * &mq) % n;
    // Compute the second candidate by subtracting r1 from n
    let r2 = n - &r1;

    // Compute third candidate r3 by negating mp and mq and combining with CRT
    let r3 = (&yp * q * &-mp + &yq * p * &-mq) % n;
    // Compute the fourth candidate by subtracting r3 from n
    let r4 = n - &r3;

    // Log all four candidates for debugging
    log::debug!("Candidates: r1 = {}, r2 = {}, r3 = {}, r4 = {}", r1, r2, r3, r4);

    // Return all four potential roots as a vector
    vec![r1, r2, r3, r4]
}


#[cfg(test)]
mod tests {
    use crate::encoding::{num2str, str2num, DEFAULT_SYMBOLS};
    use super::*;

    #[test]
    fn test_compute_candidates() {
        let (n, p, q) = generate_keypair(512);
        let ciphertext = BigInt::from(123456u32);

        // Generate decryption candidates
        let candidates = compute_candidates(&ciphertext, &p, &q, &n);

        assert_eq!(
            candidates.len(),
            4,
            "Compute_candidates should return exactly four candidates"
        );

        // Ensure candidates are unique
        let unique_candidates: std::collections::HashSet<_> = candidates.iter().collect();
        assert_eq!(
            unique_candidates.len(),
            4,
            "Decryption candidates should be unique"
        );
    }

    #[test]
    fn test_decrypt_candidates() {
        use std::collections::HashSet;

        let (n, p, q) = generate_keypair(512);
        let message = BigInt::from(123u32); // Arbitrary message for testing
        let ciphertext = encrypt(&message, &n);

        // Decrypt the ciphertext
        let candidates = decrypt(&ciphertext, &p, &q);

        // Verify the number of candidates
        assert_eq!(
            candidates.len(),
            4,
            "Decrypt should return exactly 4 candidates"
        );

        // Ensure all candidates are unique
        let unique_candidates: HashSet<_> = candidates.iter().collect();
        assert_eq!(
            unique_candidates.len(),
            4,
            "Decryption candidates should be unique"
        );

        // Verify that each candidate squared modulo n equals the ciphertext
        for candidate in &candidates {
            let squared = (candidate * candidate) % &n;
            assert_eq!(
                squared, ciphertext,
                "Each candidate squared modulo n should equal the ciphertext"
            );
        }
    }

    #[test]
    fn test_encrypt_produces_expected_ciphertext() {
        // Generate a keypair
        let (n, _, _) = generate_keypair(512);

        // Define a known message
        let message = BigInt::from(123u32);

        // Perform encryption
        let ciphertext = encrypt(&message, &n);

        // Manually compute the expected ciphertext
        let expected_ciphertext = (&message * &message) % &n;

        // Verify that the produced ciphertext matches the expected value
        assert_eq!(
            ciphertext, expected_ciphertext,
            "The ciphertext produced by encryption does not match the expected value"
        );
    }

    #[test]
    fn test_encrypt_with_string_encoding() {
        use crate::encoding::str2num; // Ensure str2num is accessible
        use crate::encoding::DEFAULT_SYMBOLS;

        // Generate a keypair
        let (n, _, _) = generate_keypair(512);

        // Define a known string message
        let message_str = "TestMessage123";

        // Encode the string into a number
        let message_num = str2num(message_str, DEFAULT_SYMBOLS)
            .expect("Failed to convert string to number");

        // Encrypt the encoded number
        let ciphertext = encrypt(&message_num, &n);

        // Manually compute the expected ciphertext
        let expected_ciphertext = (&message_num * &message_num) % &n;

        // Verify that the produced ciphertext matches the expected value
        assert_eq!(
            ciphertext, expected_ciphertext,
            "The ciphertext does not match the expected value after encoding"
        );
    }

    #[test]
    fn test_decrypt_exercise_message() {
        use crate::encoding::{num2str, str2num, DEFAULT_SYMBOLS};
        use num_bigint::BigInt;

        // Provided private key components
        let p = BigInt::parse_bytes(
            b"5081134225938911632501879835073274182691064608067531203259",
            10,
        )
            .unwrap();
        let q = BigInt::parse_bytes(
            b"5258660163169151701715131756224662568205137498312501937487",
            10,
        )
            .unwrap();
        let n = &p * &q;

        // Define the plaintext and encode it into a number
        let expected_plaintext = "recommended website";
        let plaintext_num = str2num(expected_plaintext, DEFAULT_SYMBOLS)
            .expect("Failed to convert plaintext to number");

        // Encrypt the plaintext number to generate the ciphertext
        let ciphertext = encrypt(&plaintext_num, &n);

        // Decrypt the ciphertext using the private key
        let candidates = decrypt(&ciphertext, &p, &q);

        // Check if one of the decrypted candidates matches the original plaintext
        let mut found_match = false;
        for candidate in &candidates {
            let decoded_text = num2str(candidate, DEFAULT_SYMBOLS);
            println!("Decrypted candidate: {}", decoded_text);

            if decoded_text == expected_plaintext {
                found_match = true;
                break;
            }
        }

        // Assert that at least one candidate matches the expected plaintext
        assert!(
            found_match,
            "None of the decrypted candidates matched the expected plaintext"
        );
    }

    #[test]
    fn test_encrypt_decrypt_message() {
        use crate::encoding::{num2str, str2num, DEFAULT_SYMBOLS};

        // Generate keypair
        let (n, p, q) = generate_keypair(512);

        // Original plaintext message
        let message_str = "Hello, Rabin!";
        let message_num = str2num(message_str, DEFAULT_SYMBOLS).expect("Failed to convert string to number");

        // Encrypt the message
        let ciphertext = encrypt(&message_num, &n);

        // Decrypt the message
        let candidates = decrypt(&ciphertext, &p, &q);

        // Check if one candidate matches the original message
        let mut found_match = false;
        for candidate in &candidates {
            if let decoded_text = num2str(candidate, DEFAULT_SYMBOLS) {
                println!("Decrypted candidate: {}", decoded_text);
                if decoded_text == message_str {
                    found_match = true;
                    break;
                }
            }
        }

        assert!(
            found_match,
            "None of the decrypted candidates matched the original message"
        );
    }

}
