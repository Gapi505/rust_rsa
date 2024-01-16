extern crate is_prime;
//#[macro_use]
extern crate lazy_static;
use is_prime::*;
use std::str::FromStr;
use num_bigint::{BigUint,BigInt, ToBigUint, RandBigInt, ToBigInt};
use num_traits::{One, Zero};
use num_integer::lcm;
use std::collections::HashMap;
use dialoguer::{theme::ColorfulTheme, Input, FuzzySelect, Confirm};
use std::fs::File;
use std::io::prelude::*;
use serde_json::json;


fn main() {
    app();
}
fn app(){

    //gets both keys from json files
    let mut public_file = File::open("public_key.json").unwrap_or_else(|_| {
        let (public, _) = create_key_pair(10); // replace 10 with the number of prime digits you want
        save_key_pair(&public, &HashMap::new()).unwrap(); // replace HashMap::new() with your private key if you have it
        File::open("public_key.json").unwrap()
    });
    let mut public_json = String::new();
    public_file.read_to_string(&mut public_json).unwrap();
    let public: HashMap<String, BigUint> = convert_hashmap_sb(&serde_json::from_str(&public_json).unwrap());

    let mut private_file = File::open("private_key.json").unwrap_or_else(|_| {
        let (_, private) = create_key_pair(10); // replace 10 with the number of prime digits you want
        save_key_pair(&HashMap::new(), &private).unwrap(); // replace HashMap::new() with your public key if you have it
        File::open("private_key.json").unwrap()
    });
    let mut private_json = String::new();
    private_file.read_to_string(&mut private_json).unwrap();
    let private: HashMap<String, BigUint> = convert_hashmap_sb(&serde_json::from_str(&private_json).unwrap());

    let mut passwords_file = File::open("passwords.json").unwrap_or_else(|_| {
        let passwords: HashMap<String, String> = HashMap::new(); // replace with your passwords if you have them
        let passwords_json = serde_json::to_string(&passwords).unwrap();
        let mut passwords_file = File::create("passwords.json").unwrap();
        passwords_file.write_all(passwords_json.as_bytes()).unwrap();
        File::open("passwords.json").unwrap()
    });
    let mut passwords_json = String::new();
    passwords_file.read_to_string(&mut passwords_json).unwrap();
    let mut passwords: HashMap<String, String> = serde_json::from_str(&passwords_json).unwrap();

    let actions = &[
        "Generate new key pair",
        "Save password",
        "Get password",
    ];
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("What do you want to do?")
        .default(0)
        .items(&actions[..])
        .interact()
        .unwrap();
    match selection {
        0 => {
            let prime_digits = Input::<u32>::new()
                .with_prompt("How many digits should the prime numbers have?")
                .interact()
                .unwrap();
            let (public, private) = create_key_pair(prime_digits);
            println!("Public key: {:?}", public);
            println!("Private key: {:?}", private);
            save_key_pair(&public, &private).unwrap();

            // resets passwords
            passwords = HashMap::new();
            save_passwords(&passwords).unwrap();
        },
        1 => {
            let name = Input::<String>::new()
                .with_prompt("What is the name of the password?")
                .interact()
                .unwrap();
            let password = Input::<String>::new()
                .with_prompt("What is the password?")
                .interact()
                .unwrap();
            passwords.insert(name, encrypt(password, public["e"].clone(), public["n"].clone()));
            save_passwords(&passwords).unwrap();
        },
        2 => {
            let password_names: Vec<String> = passwords.keys().map(|s| s.to_string()).collect();
            let name = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Which password do you want to get?")
                .default(0)
                .items(&password_names[..])
                .interact()
                .unwrap();
            let name = password_names[name].clone();
            let password = decrypt(passwords[&name].clone(), private["d"].clone(), private["n"].clone());
            println!("Password: {:?}", password);
        },
        _ =>{
            println!("Not implemented yet")
        }
    }
    if Confirm::new()
        .with_prompt("Do you want to do something else?")
        .interact()
        .unwrap()
    {
        app();
    }
}

fn save_key_pair(public: &HashMap<String, BigUint>, private: &HashMap<String, BigUint>) -> std::io::Result<()> {
    let public_json = json!(convert_hasmap_bs(public));
    let private_json = json!(convert_hasmap_bs(private));

    let mut public_file = File::create("public_key.json")?;
    public_file.write_all(public_json.to_string().as_bytes())?;

    let mut private_file = File::create("private_key.json")?;
    private_file.write_all(private_json.to_string().as_bytes())?;

    Ok(())
}
fn convert_hasmap_bs(hashmap: &HashMap<String, BigUint>) -> HashMap<String, String> {
    hashmap.iter().map(|(k, v)| (k.clone(), v.to_string())).collect()
}
fn convert_hashmap_sb(hashmap: &HashMap<String, String>) -> HashMap<String, BigUint> {
    hashmap.iter().map(|(k, v)| (k.clone(), BigUint::from_str(v).unwrap())).collect()
}

fn save_passwords(passwords: &HashMap<String, String>) -> std::io::Result<()> {
    let passwords_json = json!(passwords);

    let mut passwords_file = File::create("passwords.json")?;
    passwords_file.write_all(passwords_json.to_string().as_bytes())?;

    Ok(())
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
fn create_key_pair(prime_digits: u32) -> (HashMap<String, BigUint>, HashMap<String, BigUint>) {
    let mut public = HashMap::new();
    let mut private = HashMap::new();
    // Choose two large prime numbers p and q
    let p = generate_prime(prime_digits);
    let q = generate_prime(prime_digits);

    // Compute n = pq
    let n = &p * &q;
    public.insert("n".to_string(), n.clone());
    private.insert("n".to_string(), n.clone());

    // Compute λ(n) = lcm(p - 1, q - 1)
    let lambda_n = carmichaels_totient(p.clone(), q.clone());

    // Choose an integer e such that 2 < e < λ(n) and gcd(e, λ(n)) = 1
    let e =generate_prime(6); //&*E;
    public.insert("e".to_string(), e.clone());

    // Determine d as d ≡ e−1 (mod λ(n))
    let d = mod_inverse(e.clone().to_bigint().unwrap(), lambda_n.clone().to_bigint().unwrap()).unwrap().to_biguint().unwrap();
    private.insert("d".to_string(), d.clone());

    (public, private)
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

fn encrypt(message: String, e: BigUint, n: BigUint) -> String {
    let mut encrypted_message_vec = Vec::new();
    let mut encrypted_message = String::new();
    let mut chunk = "0".to_string();

    for (i,c) in message.chars().enumerate() {
        if i == message.len() - 1{
            if (c as u32) < 100 {
                chunk.push('0');
            }
            chunk = chunk + &(c as u32).to_string();
            let m = &chunk as &str;
            let m = BigUint::from_str(m).unwrap();
            chunk = "0".to_string();
            let c = encrypt_num(m, e.clone(), n.clone());
            encrypted_message_vec.push(c.to_string());
        }
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
            encrypted_message_vec.push(c.to_string());
        }
    }
    for c in encrypted_message_vec {
        encrypted_message = encrypted_message.clone() + if encrypted_message != "" {" "} else {""} + &c;
    }
    encrypted_message
}

fn decrypt(encrypted_message: String, d: BigUint, n: BigUint) -> String {
    let mut decrypted_message = String::new();
    let encrypted_message_vec: Vec<String> = encrypted_message.split(" ").map(|s| s.to_string()).collect();
    for c in encrypted_message_vec {
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
