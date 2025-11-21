use std::{fmt::Display, str::FromStr};
use reformy::{form, menu, Form};

#[derive(Debug, Default)]
struct Email(String);

impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Email {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("@") {
            Ok(Email(s.to_string()))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Default, Form)]
enum Role {
    Admin{
        password: String,
    },
    #[default]
    User,
}

#[form]
fn create_user(name: String, age: usize, email: Email) -> String {
    format!("✓ Created user: {} (age: {}, email: {})", name, age, email)
}

#[form]
fn greet_person(first_name: String, last_name: String) -> String {
    format!("Hello, {} {}! Welcome to reformy!", first_name, last_name)
}

#[form]
fn calculate_sum(a: usize, b: usize) -> usize {
    a + b
}


#[form]
fn show_status() -> String {
    "System is running normally ✓".to_string()
}

#[form]
fn get_timestamp() -> String {
    format!("Current time: {:?}", std::time::SystemTime::now())
}

#[derive(Debug, Default, Form)]
struct Person {
    name: String,
    age: usize,
    #[form]
    role: Role,
    email: Email,
}

#[derive(Debug, Default, Form)]
struct Address {
    street: String,
    city: String,
    zip_code: usize,
}

#[form]
fn register_person(#[form] person: Person, #[form] address: Address) -> String {
    format!(
        "✓ Registered {} (age {}, email: {}) at {}, {} - {}",
        person.name, person.age, person.email, address.street, address.city, address.zip_code
    )
}

fn main() {
    menu! {
        show_status,
        get_timestamp,
        
        "User Management" => {
            create_user,
            register_person,
        },
        
        "Math Operations" => {
            calculate_sum,
        },
        
        "Greetings" => {
            greet_person,
        },
    }
}

