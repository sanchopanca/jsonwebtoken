use chrono::Utc;
use jsonwebtoken_rustcrypto::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let my_claims = Claims {
        sub: "b@b.com".to_string(),
        company: "ACME".to_string(),
        exp: Utc::now().timestamp() as usize + 10000,
    };

    let token =
        encode(&Header::default(), &my_claims, &EncodingKey::from_hmac_secret("secret".as_ref()))?;

    println!("Our encoded token: {token}");

    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_hmac_secret("secret".as_ref()),
        &Validation::default(),
    )?;

    assert_eq!(my_claims, token_data.claims);

    println!("Our decoded token: {:?}", token_data);

    Ok(())
}
