
use hyper::{Request, Response, Body, header::*};
use crate::token::{create_token, sign_token};
use structopt::StructOpt;


#[derive(StructOpt)]
#[structopt(name = "csrf-token-server", about = "HTTP server that responds with CSRF token in cookie")]
pub struct HttpArgs {
    checksum_secret: String,
    #[structopt(short = "t", long = "token-cookie", default_value = "varnish_token")]
    token_cookie_name: String,

    #[structopt(short = "c", long = "checksum-cookie", default_value = "varnish_token_checksum")]
    checksum_cookie_name: String,

    #[structopt(short = "d", long = "cookie-domain", default_value = "")]
    cookie_domain: String,

    #[structopt(short = "p", long = "server-port", default_value = "9999")]
    port: u16
}

impl HttpArgs {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn http(&self) -> Http {
        Http {
            checksum_secret: self.checksum_secret.clone(),
            cookie_domain: self.cookie_domain.clone(),
            checksum_cookie_name: self.checksum_cookie_name.clone(),
            token_cookie_name: self.token_cookie_name.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Http
{
    checksum_secret: String,
    token_cookie_name: String,
    checksum_cookie_name: String,
    cookie_domain: String
}

impl Http {

    pub fn serve(&self, req: Request<Body>) -> Response<Body> {
        let default_domain = HeaderValue::from_str(&self.cookie_domain).unwrap();

        let domain: &HeaderValue = match self.cookie_domain.len() {
            0 => req.headers().get("HOST").unwrap(),
            _ => &default_domain
        };

        let token = create_token(16);

        Response::builder()
            .header(
                SET_COOKIE,
                format!(
                    "{}={}; Domain={}; Path=/",
                    &self.token_cookie_name,
                    token,
                    domain.to_str().unwrap()
                )
            )
            .header(
                SET_COOKIE,
                format!(
                    "{}={}; Domain={}; Path=/; HttpOnly",
                    &self.checksum_cookie_name,
                    sign_token(&token, &self.checksum_secret),
                    domain.to_str().unwrap()
                )
            )
            .body(Body::from(r#"{"status":"ok"}"#)).unwrap()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn generate_token() -> String {
        return "my_awesome_generated_token".into();
    }

    fn http_responder() -> Http {
        Http {
            checksum_secret: "secret_value".to_string(),
            token_cookie_name: "my_token".to_string(),
            checksum_cookie_name: "my_token_checksum".to_string(),
            cookie_domain: "".to_string()
        }
    }

    #[test]
    fn generates_a_token_value () {
        let response = http_responder().serve(
            Request::builder()
                .method("GET")
                .header("Host", "foo.com")
                .body(Body::from(""))
                .unwrap()
        );

        assert_eq!(15, response.body().content_length().unwrap())
    }
}