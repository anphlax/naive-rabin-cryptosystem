mod encoding;

use log::info;
use num_bigint::{BigInt, RandBigInt};
use num_bigint::BigUint;
use num_bigint::ToBigInt;
use num_bigint::ToBigUint;
use num_prime::{PrimalityTestConfig, RandPrime};
use num_traits::{One, Zero};
use rand::thread_rng;
use std::env;
use rayon::prelude::*;

// TODO: Extend the code to correctly select a plaintext candidate after decryption

fn main() {
    // Initialize the logger
    env::set_var("RUST_LOG", "info");
    env_logger::init();
    info!("Hello, Naive Rabin Cryptosystem Implementation...");

    let bits = 512;
    let (n,p,q) = generate_keypair(bits);

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
        // &(a % b) creates a reference to the new BigInt result. This reference is passed to the recursive call instead of moving the value, avoiding unnecessary allocation.
        gcd(b, &(a % b))
    }
}

fn gen_prime(bit_size: usize) -> BigUint {
    let mut rng = rand::thread_rng();
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

    let mut attempts = 1;

    // Generate two primes in parallel
    let primes: Vec<BigInt> = (0..2)
        .into_par_iter()
        .map(|_| BigInt::from(gen_prime(bit_size)))
        .collect();

    // Assign p and q from the primes vector
    let (p, q) = (primes[0].clone(), primes[1].clone());

    info!("Generated primes p and q after {} attempts", attempts);
    let n = &p * &q; // Compute modulus n
    (n, p, q)
}

fn encrypt(message: &BigInt, n: &BigInt) -> BigInt {
    (message * message) % n
}

fn decrypt(ciphertext: &BigInt, p: &BigInt, q: &BigInt) -> Vec<BigInt> {
    let n = p * q;
    let mp = ciphertext.modpow(&(p + BigInt::one() / BigInt::from(4)), p);
    let mq = ciphertext.modpow(&(q + BigInt::one() / BigInt::from(4)), q);

    let (yp, yq) = (
        q.modpow(&(p - BigInt::from(2)), p),
        p.modpow(&(q - BigInt::from(2)), q),
    );

    let r1 = (&yp * q * &mp + &yq * p * &mq) % &n;
    let r2 = &n - &r1;
    let r3 = (&yp * q * &-mp + &yq * p * &-mq) % &n;
    let r4 = &n - &r3;

    vec![r1, r2, r3, r4]
}
