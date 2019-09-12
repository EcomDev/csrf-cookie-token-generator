
use hyper::{Request, Response, Body, header::*};
use crate::token::{create_token, sign_token};
use structopt::StructOpt;
use hyper::service::Service;
use std::error::Error;
use std::fmt;
use futures::prelude::*;
use futures::future;
use rand::RngCore;
use rand::prelude::*;
use rand::rngs::SmallRng;


#[derive(Debug)]
pub enum NeverErrors {}

impl fmt::Display for NeverErrors {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}

impl Error for NeverErrors {
    fn description(&self) -> &str {
        match *self {}
    }
}


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

impl HttpArgs
{
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn http(&self) -> Http<SmallRng> {
        Http {
            checksum_secret: self.checksum_secret.clone(),
            cookie_domain: self.cookie_domain.clone(),
            checksum_cookie_name: self.checksum_cookie_name.clone(),
            token_cookie_name: self.token_cookie_name.clone(),
            rng: SmallRng::from_entropy()
        }
    }
}

#[derive(Clone)]
pub struct Http<T>
    where T: RngCore + Sized
{
    checksum_secret: String,
    token_cookie_name: String,
    checksum_cookie_name: String,
    cookie_domain: String,
    rng: T
}

impl<T> Http <T> where T: RngCore + Sized {

    pub fn serve(&mut self, req: Request<Body>) -> Response<Body> {
        let default_domain = HeaderValue::from_str(&self.cookie_domain).unwrap();

        let domain: &HeaderValue = match self.cookie_domain.len() {
            0 => req.headers().get("HOST").unwrap(),
            _ => &default_domain
        };

        let token = create_token(16, &mut self.rng);

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

impl<T> IntoFuture for Http<T>
    where T: RngCore + Sized
{
    type Future = future::FutureResult<Self::Item, Self::Error>;
    type Item = Self;
    type Error = NeverErrors;

    fn into_future(self) -> Self::Future {
        future::ok(self)
    }
}

impl <T> Service for Http<T>
    where T: RngCore + Sized
{
    type ReqBody = Body;
    type ResBody = Body;
    type Error = NeverErrors;
    type Future = future::FutureResult<Response<Self::ResBody>, Self::Error>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        future::ok(self.serve(req))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use rand::rngs::mock::StepRng;
    use cookie::{CookieJar, Cookie};

    fn http_responder(domain: &str, ) -> Http<StepRng> {
        Http {
            checksum_secret: "secret_value".to_string(),
            token_cookie_name: "my_token".to_string(),
            checksum_cookie_name: "my_token_checksum".to_string(),
            cookie_domain: domain.to_string(),
            rng: StepRng::new(10_000, 10_000000)
        }
    }

    fn default_http_responder() -> Http<StepRng> {
        http_responder("")
    }

    #[test]
    fn returns_json_status_ok_on_get_request()
    {
        let body = request_page(
            &mut default_http_responder(),
            create_default_request("foo.com")
        ).body_mut().concat2().wait().unwrap();

        assert_eq!(r#"{"status":"ok"}"#, body.into_bytes())
    }

    #[test]
    fn sets_token_value_cookies_with_request_domain()
    {
        let response = request_page(
            &mut default_http_responder(),
            create_default_request("foo.com")
        );

        let cookies = parse_cookies(&response);

        assert_eq!(
            Cookie::build("my_token", "AAAAAAABBBBBBBCC")
                .domain("foo.com")
                .path("/")
                .finish(),
            *cookies.get("my_token").unwrap()
        );
    }

    #[test]
    fn sets_token_value_checksum_with_http_only_flag_and_requested_domain()
    {
        let response = request_page(
            &mut default_http_responder(),
            create_default_request("foo2.com")
        );

        let cookies = parse_cookies(&response);

        assert_eq!(
            Cookie::build("my_token_checksum", "22002be6cfc130a4ac88385ad8adfa55")
                .domain("foo2.com")
                .path("/")
                .http_only(true)
                .finish(),
            *cookies.get("my_token_checksum").unwrap()
        );
    }

    fn parse_cookies(response: &Response<Body>) -> CookieJar
    {
        let mut cookies = CookieJar::new();

        response.headers()
            .get_all(SET_COOKIE).iter()
            .for_each( | v| cookies.add(
                Cookie::parse(v.to_str().unwrap()).unwrap().into_owned()
            ));

        cookies
    }

    fn request_page(responder: &mut Http<StepRng>, request: Request<Body>) -> Response<Body> {
        match responder.call(
            request
        ).poll().unwrap() {
            Async::Ready(v) => v,
            _ => unreachable!()
        }
    }

    fn create_default_request(host: &str) -> Request<Body> {
        Request::builder()
            .method("GET")
            .header("Host", host)
            .body(Body::empty())
            .unwrap()
    }
}