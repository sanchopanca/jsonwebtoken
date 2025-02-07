> this is a fork of `jsonwebtoken-rustcrypto` to work around dependency issues with `subtle`
> use it like this:
> ```
> jsonwebtoken-rustcrypto = { git = "https://github.com/sanchopanca/jsonwebtoken", rev = "beba387a2eb40ef4e02df1b735a941a5d52c1589" }
> ```
> See also: https://github.com/JadedBlueEyes/jsonwebtoken/pull/2
# jsonwebtoken-rustcrypto

This is a fork of Keats' jsonwebtoken crate that uses RustCrypto crates (`rsa`, `sha2`, `hmac`) instead of Ring. This reduces the amount of work being done in the crate significantly and allows more flexibility in how the user loads RSA keys.

Caveats:

- No ECDSA: I didn't have a need for ECDSA or the time to research and implement EC using RustCrypto crates and I removed it to completely remove the ring dependency.
- HMAC signature verification doesn't use constant time comparison like Keats's original does.

See [JSON Web Tokens](https://en.wikipedia.org/wiki/JSON_Web_Token) for more information on what JSON Web Tokens are.

## Installation

Add the following to Cargo.toml:

```toml
jsonwebtoken-rustcrypto = "1"
serde = {version = "1", features = ["derive"] }
```

The minimum required Rust version is 1.40.

## Algorithms

This library currently supports the following:

- HS256
- HS384
- HS512
- RS256
- RS384
- RS512
- PS256
- PS384
- PS512

### Removed

- ES256
- ES384

## How to use

Complete examples are available in the examples directory: a basic one and one with a custom header.

In terms of imports and structs:

```rust
use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}
```

### Claims

The claims fields which can be validated. (see [validation](#validation))

```rust
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: String,         // Optional. Audience
    exp: usize,          // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize,          // Optional. Issued at (as UTC timestamp)
    iss: String,         // Optional. Issuer
    nbf: usize,          // Optional. Not Before (as UTC timestamp)
    sub: String,         // Optional. Subject (whom token refers to)
}
```

### Header

The default algorithm is HS256, which uses a shared secret.

```rust
let token = encode(&Header::default(), &my_claims, &EncodingKey::from_hmac_secret("secret".as_ref()))?;
```

#### Custom headers & changing algorithm

All the parameters from the RFC are supported but the default header only has `typ` and `alg` set.
If you want to set the `kid` parameter or change the algorithm for example:

```rust
let mut header = Header::new(Algorithm::HS512);
header.kid = Some("blabla".to_owned());
let token = encode(&header, &my_claims, &EncodingKey::from_hmac_secret("secret".as_ref()))?;
```

Look at `examples/custom_header.rs` for a full working example.

### Encoding

```rust
// HS256
let token = encode(&Header::default(), &my_claims, &EncodingKey::from_hmac_secret("secret".as_ref()))?;
// RSA
let token = encode(&Header::new(Algorithm::RS256), &my_claims, &EncodingKey::from_rsa(RSAPrivateKey::new(&mut rng, bits).unwrap())?)?;
```

Encoding a JWT takes 3 parameters:

- a header: the `Header` struct
- some claims: your own struct
- a key/secret

When using HS256, HS2384 or HS512, the key is always a shared secret like in the example above. When using
RSA/EC, the key should always be the content of the private key in the PEM or DER format.

If your key is in PEM format, it is better performance wise to generate the `EncodingKey` once in a `lazy_static` or
something similar and reuse it.

### Decoding

```rust
// `token` is a struct with 2 fields: `header` and `claims` where `claims` is your own struct.
let token = decode::<Claims>(&token, &DecodingKey::from_hmac_secret("secret".as_ref()), &Validation::default())?;
```

`decode` can error for a variety of reasons:

- the token or its signature is invalid
- the token had invalid base64
- validation of at least one reserved claim failed

As with encoding, when using HS256, HS2384 or HS512, the key is always a shared secret like in the example above. When using
RSA/EC, the key should always be the content of the public key in the PEM or DER format.

In some cases, for example if you don't know the algorithm used or need to grab the `kid`, you can choose to decode only the header:

```rust
let header = decode_header(&token)?;
```

This does not perform any signature verification or validate the token claims.

You can also decode a token using the public key components of a RSA key in base64 format.
The main use-case is for JWK where your public key is in a JSON format like so:
Look at the `rsa` crate's docs for instructions.

```json
{
   "kty":"RSA",
   "e":"AQAB",
   "kid":"6a7a119f-0876-4f7e-8d0f-bf3ea1391dd8",
   "n":"yRE6rHuNR0QbHO3H3Kt2pOKGVhQqGZXInOduQNxXzuKlvQTLUTv4l4sggh5_CYYi_cvI-SXVT9kPWSKXxJXBXd_4LkvcPuUakBoAkfh-eiFVMh2VrUyWyj3MFl0HTVF9KwRXLAcwkREiS3npThHRyIxuy0ZMeZfxVL5arMhw1SRELB8HoGfG_AtH89BIE9jDBHZ9dLelK9a184zAf8LwoPLxvJb3Il5nncqPcSfKDDodMFBIMc4lQzDKL5gvmiXLXB1AGLm8KBjfE8s3L5xqi-yUod-j8MtvIj812dkS4QMiRVN_by2h3ZY8LYVGrqZXZTcgn2ujn8uKjXLZVD5TdQ"
}
```

If your key is in PEM format, it is better performance wise to generate the `DecodingKey` once in a `lazy_static` or
something similar and reuse it.

### Validation

```rust
use jsonwebtoken_rustcrypto::{Validation, Algorithm};

// Default validation: the only algo allowed is HS256
let validation = Validation::default();
// Quick way to setup a validation where only the algorithm changes
let validation = Validation::new(Algorithm::HS512);
// Adding some leeway (in seconds) for exp and nbf checks
let mut validation = Validation {leeway: 60, ..Default::default()};
// Checking issuer
let mut validation = Validation {iss: Some("issuer".to_string()), ..Default::default()};
// Setting audience
let mut validation = Validation::default();
validation.set_audience(&"Me"); // string
validation.set_audience(&["Me", "You"]); // array of strings
```

Look at `examples/validation.rs` for a full working example.
