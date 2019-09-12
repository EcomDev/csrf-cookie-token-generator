use serde_json::{to_string, json};
use md5::compute;
use rand::{distributions::{Alphanumeric, Distribution}, RngCore};

fn create_token_string_for_checksum(token: &String, secret: &String) -> String
{
    let value = json!({
        "salt": secret,
        "token": token
    });
    return to_string(&value).unwrap()
}

pub fn sign_token(token: &String, secret: &String) -> String
{
    format!(
        "{:x}",
        compute(create_token_string_for_checksum(token, secret))
    )
}

pub fn create_token(len: usize,  rng: &mut impl RngCore) -> String
{
    let mut key = String::with_capacity(len);


    while key.len() < len {
        key.push(Alphanumeric.sample(rng))
    }

    key
}

#[cfg(test)]
mod tests
{
    use super::*;
    use rand::rngs::mock::StepRng;

    #[test]
    fn generates_token_json_string () {
        let token = String::from("token_to_sign");
        let secret = String::from("some_salt_value");
        assert_eq!(
           r#"{"salt":"some_salt_value","token":"token_to_sign"}"#,
           create_token_string_for_checksum(&token, &secret)
       );
    }

    #[test]
    fn signs_generated_token() {
        let token = String::from("token_value");
        let secret = String::from("secret_key");
        assert_eq!(
            "762ef217eb65d00028f7a102b2b0bbb8",
            sign_token(&token, &secret)
        )
    }

    #[test]
    fn generates_a_token_with_rng() {
        let mut rng = StepRng::new(10, 9000000);
        assert_eq!(
            "AAAAAAAABBBBBBBC",
            create_token(16, &mut rng)
        );
    }
}