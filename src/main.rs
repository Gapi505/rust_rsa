extern crate is_prime;
#[macro_use]
extern crate lazy_static;
use is_prime::*;
use rand::Rng;
use std::{mem, str::{FromStr, Chars}};
use num_bigint::{BigUint,BigInt, ToBigUint, RandBigInt, ToBigInt};
use num_traits::{One, Zero, cast::ToPrimitive};
use num_integer::lcm;
use std::collections::HashMap;


fn main() {
    // Generate a key pair
    let (public,private) = create_key_pair();
    println!("Public key: {:?}", public);
    println!("Private key: {:?}", private);


    // Encrypt a message
    let message = "Hello, world! How are you?".to_string();
    let encrypted_message = encrypt(message, public["e"].clone(), public["n"].clone());
    println!("Encrypted message: {:?}", encrypted_message);

    // Decrypt the message
    let decrypted_message = decrypt(encrypted_message, private["d"].clone(), public["n"].clone());
    println!("Decrypted message: {:?}", decrypted_message);
    // Check if the decrypted message is the same as the original message
}
fn generate_prime(digits: u32) -> BigUint {
    let mut rng = rand::thread_rng();
    let lower_bound = BigUint::from(10u32).pow(digits - 1);
    let upper_bound = BigUint::from(10u32).pow(digits);

    let mut prime = rng.gen_biguint_range(&lower_bound, &upper_bound);
    while !is_prime(&prime.to_string()) {
        prime = rng.gen_biguint_range(&lower_bound, &upper_bound);
    }
    prime
}
fn create_key_pair() -> (HashMap<String, BigUint>, HashMap<String, BigUint>) {
    let mut public = HashMap::new();
    let mut private = HashMap::new();
    // Choose two large prime numbers p and q
    let p = generate_prime(10);
    let q = generate_prime(10);

    // Compute n = pq
    let n = &p * &q;
    public.insert("n".to_string(), n.clone());

    // Compute λ(n) = lcm(p - 1, q - 1)
    let lambda_n = carmichaels_totient(p.clone(), q.clone());
    print!("lambda_n: {}\n", lambda_n);

    // Choose an integer e such that 2 < e < λ(n) and gcd(e, λ(n)) = 1
    let e =generate_prime(5); //&*E;
    public.insert("e".to_string(), e.clone());

    // Determine d as d ≡ e−1 (mod λ(n))
    let d = mod_inverse(e.clone().to_bigint().unwrap(), lambda_n.clone().to_bigint().unwrap()).unwrap().to_biguint().unwrap();
    private.insert("d".to_string(), d.clone());

    (public, private)
}
fn modular_exponent(mut n: BigUint, mut x: BigUint, p: BigUint) -> BigUint {
    let mut ans = BigUint::one();
    if x <= BigUint::zero() { return BigUint::one(); }
    loop {
        if x == BigUint::one() { return (ans.clone() * n.clone()) % p.clone(); }
        if x.clone() & BigUint::one() == BigUint::zero() { 
            n = (n.clone() * n.clone()) % p.clone(); 
            x = x >> 1;
            continue; 
        } else { 
            ans = (ans * n.clone()) % p.clone(); 
            x = x - BigUint::one(); 
        }
    }
}

fn gcd(mut a: BigUint, mut b: BigUint) -> BigUint {
    if a == b { return a; }
    if b > a { mem::swap(&mut a, &mut b); }
    while b > BigUint::zero() { 
        let temp = a.clone();
        a = b.clone();
        b = temp % b;
    }
    return a;
}

fn mod_inverse(a: BigInt, module: BigInt) -> Option<BigInt> {
    let mut mn = (module.clone(), a.clone());
    let mut xy = (BigInt::zero(), BigInt::one());

    while mn.1 != BigInt::zero() {
        let quotient = &mn.0 / &mn.1;
        xy = (xy.1.clone(), &xy.0 - quotient * &xy.1);
        mn = (mn.1.clone(), &mn.0 % &mn.1);
    }

    if mn.0 > BigInt::one() {
        None // Inverse does not exist
    } else {
        let res = xy.0 % &module;
        if res < BigInt::zero() {
            Some(res + &module)
        } else {
            Some(res)
        }
    }
}
fn modular_exponentiation(m: BigUint, e: BigUint, n: BigUint) -> BigUint {
    let mut result = BigUint::one();
    let mut base = m % &n;
    let mut exponent = e;

    while exponent > BigUint::zero() {
        if &exponent % 2.to_biguint().unwrap() == BigUint::one() {
            result = (result * &base) % &n;
        }
        exponent = exponent >> 1;
        base = (&base * &base) % &n;
    }

    result
}

fn carmichaels_totient(p: BigUint, q: BigUint) -> BigUint {
    let one = BigUint::one();
    let lambda_p = &p - &one;
    let lambda_q = &q - &one;
    lcm(lambda_p, lambda_q)
}

fn decrypt_num(c: BigUint, d: BigUint, n: BigUint) -> BigUint {
    modular_exponentiation(c, d, n)
}
fn encrypt_num(m: BigUint, e: BigUint, n: BigUint) -> BigUint {
    modular_exponentiation(m, e, n)
}

fn encrypt(message: String, e: BigUint, n: BigUint) -> Vec<String> {
    let mut encrypted_message = Vec::new();
    let mut chunk = "0".to_string();
    let mut message = message;
    let target_length = message.len() + n.clone().to_string().len() - 3;
    while message.len() < target_length {
        message.push(' ');
    }
    for c in message.chars() {
        if chunk.len() < n.clone().to_string().len() - 6 {
            if chunk == "0".to_string(){
                chunk = String::new();
            }
            if (c as u32) < 100 {
                chunk.push('0');
            }
            chunk = chunk + &(c as u32).to_string();
            continue;
        }
        if chunk.len() < n.clone().to_string().len() - 3 {
            if (c as u32) < 100 {
                chunk.push('0');
            }
            chunk = chunk + &(c as u32).to_string();
            let m = &chunk as &str;
            let m = BigUint::from_str(m).unwrap();
            chunk = "0".to_string();
            let c = encrypt_num(m, e.clone(), n.clone());
            encrypted_message.push(c.to_string());
        }
    }
    encrypted_message
}

fn decrypt(encrypted_message: Vec<String>, d: BigUint, n: BigUint) -> String {
    let mut decrypted_message = String::new();
    for c in encrypted_message {
        let c = c.parse::<BigUint>().unwrap();
        let m = decrypt_num(c.clone(), d.clone(), n.clone());
        let mut m = m.to_string();
        if m.len() % 3 != 0 {
            let mut padding = String::new();
            for _ in 0..(3 - m.len() % 3) {
                padding.push('0');
            }
            m = padding + &m;
        }
        for i in (0..m.len()-2).step_by(3) {
            let c = &m[i..i+3];
            let c = char::from_u32(c.parse::<u32>().unwrap()).unwrap();
            decrypted_message.push(c);
        }
    }
    decrypted_message
}
