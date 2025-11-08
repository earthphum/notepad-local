use backend::utils::hash_password;
use std::io::{self, Write};

fn main() {
    println!("🔐 Password Hash Generator");
    println!("This utility generates Argon2 hashes for your passwords.");
    println!();

    loop {
        print!("Enter password to hash (or 'quit' to exit): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let password = input.trim();

        if password.eq_ignore_ascii_case("quit") {
            println!("👋 Goodbye!");
            break;
        }

        if password.is_empty() {
            println!("❌ Password cannot be empty!");
            continue;
        }

        let hash = hash_password(password);
        println!("✅ Generated hash: {}", hash);
        println!("   Add this to your .env file as ADMIN_PASS_HASH={}", hash);
        println!();
    }
}
