use serde_json::{to_string, json};
use md5::compute;
use rand::{thread_rng, distributions::Alphanumeric, Rng};

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

pub fn create_token(len: u8) -> String
{
    let rng =  thread_rng();
    rng.sample_iter(Alphanumeric).take(len.into()).collect()
}

#[cfg(test)]
mod tests
{
    use super::*;

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
    fn generates_token_of_required_size() {
        assert_eq!(
            16,
            create_token(16).len()
        );
    }
}