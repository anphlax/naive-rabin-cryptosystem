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
    let mp = ciphertext.modpow(&((p + BigInt::one()) / BigInt::from(4)), p);
    let mq = ciphertext.modpow(&((q + BigInt::one()) / BigInt::from(4)), q);

    log::debug!("mp (mod p): {}", mp);
    log::debug!("mq (mod q): {}", mq);

    let yp = q.modpow(&(p - BigInt::from(2)), p);
    let yq = p.modpow(&(q - BigInt::from(2)), q);

    log::debug!("yp (modular inverse of q mod p): {}", yp);
    log::debug!("yq (modular inverse of p mod q): {}", yq);

    let r1 = (&yp * q * &mp + &yq * p * &mq) % n;
    let r2 = n - &r1;
    let r3 = (&yp * q * &-mp + &yq * p * &-mq) % n;
    let r4 = n - &r3;

    log::debug!("Candidates: r1 = {}, r2 = {}, r3 = {}, r4 = {}", r1, r2, r3, r4);

    vec![r1, r2, r3, r4]
}


#[cfg(test)]
mod tests {
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
}
