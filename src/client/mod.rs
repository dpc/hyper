//! HTTP Client

use std::default::Default;
use std::io::{IoResult, BufReader};
use std::io::util::copy;
use std::iter::Extend;

use url::UrlParser;
use url::ParseError as UrlError;

use header::Headers;
use header::common::{ContentLength, Location};
use method::Method;
use net::NetworkConnector;
use status::StatusClass::Redirection;
use {Url, Port, HttpResult};
use HttpError::HttpUriError;

pub use self::request::Request;
pub use self::response::Response;

pub mod request;
pub mod response;

/// A Client to use additional features with Requests.
///
/// Clients can handle things such as: redirect policy.
pub struct Client<S: NetworkConnector> {
    redirect_policy: RedirectPolicy
}

/// Behavior regarding how to handle redirects within a Client.
pub enum RedirectPolicy {
    /// Don't follow any redirects.
    FollowNone,
    /// Follow all redirects.
    FollowAll,
    /// Follow a redirect if the contained function returns true.
    FollowIf(fn(&Url) -> bool)
}

impl<S: NetworkConnector> Default for Client<S> {
    fn default() -> Client<S> {
        Client::new(RedirectPolicy::FollowAll)
    }
}

impl<S: NetworkConnector> Client<S> {

    /// Create a new Client.
    pub fn new(redirect_policy: RedirectPolicy) -> Client<S> {
        Client {
            redirect_policy: redirect_policy
        }
    }

    /// Execute a Get request.
    pub fn get(&mut self, url: Url) -> HttpResult<Response> {
        self.request(RequestOptions {
            method: Method::Get,
            url: url,
            headers: None,
            body: None::<&str>
        })
    }

    /// Execute a Post request.
    pub fn post<'b, B: IntoBody<'b>>(&mut self, url: Url, body: B) -> HttpResult<Response> {
        self.request(RequestOptions {
            method: Method::Post,
            url: url,
            headers: None,
            body: Some(body),
        })
    }

    /// Execute a request using this Client.
    pub fn request<'b, B: IntoBody<'b>>(&mut self, options: RequestOptions<B>) -> HttpResult<Response> {
        // self is &mut because in the future, this function will check
        // self.connection_pool, inserting if empty, when keep_alive = true.

        let RequestOptions { method, mut url, headers, body } = options;
        debug!("client.request {} {}", method, url);

        //let mut redirect_count = 0u;

        let can_have_body = match &method {
            &Method::Get | &Method::Head => false,
            _ => true
        };

        let mut body = if can_have_body {
            body.map(|b| b.into_body())
        } else {
             None // ignore? or return Err?
        };

        loop {
            let mut req = try!(Request::with_stream::<S>(method.clone(), url.clone()));
            headers.as_ref().map(|headers| req.headers_mut().extend(headers.iter()));

            match (can_have_body, body.as_ref()) {
                (true, Some(ref body)) => match body.size() {
                    Some(size) => req.headers_mut().set(ContentLength(size)),
                    None => (), // chunked, Request will add it automatically
                },
                (true, None) => req.headers_mut().set(ContentLength(0)),
                _ => () // neither
            }
            let mut streaming = try!(req.start());
            body.take().map(|mut rdr| copy(&mut rdr, &mut streaming));
            let res = try!(streaming.send());
            if res.status.class() != Redirection {
                return Ok(res)
            }
            debug!("redirect code {} for {}", res.status, url);

            let loc = {
                // punching borrowck here
                let loc = match res.headers.get::<Location>() {
                    Some(&Location(ref loc)) => {
                        Some(UrlParser::new().base_url(&url).parse(loc[]))
                    }
                    None => {
                        debug!("no Location header");
                        // could be 304 Not Modified?
                        None
                    }
                };
                match loc {
                    Some(r) => r,
                    None => return Ok(res)
                }
            };
            url = match loc {
                Ok(u) => {
                    debug!("Location: {}", u);
                    u
                },
                Err(e) => {
                    debug!("Location header had invalid URI: {}", e);
                    return Ok(res);
                }
            };
            match self.redirect_policy {
                // separate branches because they cant be one
                RedirectPolicy::FollowAll => (),
                RedirectPolicy::FollowIf(cond) if cond(&url) => (),
                _ => return Ok(res),
            }
            //redirect_count += 1;
        }
    }
}

/// Options for an individual Request.
///
/// One of these will be built for you if you use one of the convenience
/// methods, such as `get()`, `post()`, etc.
pub struct RequestOptions<'a, B: IntoBody<'a>> {
    /// The url for this request.
    pub url: Url,
    /// If any additional headers should be sent.
    pub headers: Option<Headers>,
    /// The Request Method, such as `Get`, `Post`, etc.
    pub method: Method,
    /// If a request body should be sent.
    pub body: Option<B>,
}

/// A helper trait to allow overloading of the body parameter.
pub trait IntoBody<'a> {
    /// Consumes self into an instance of `Body`.
    fn into_body(self) -> Body<'a>;
}

/// The target enum for the IntoBody trait.
pub enum Body<'a> {
    /// A Reader does not necessarily know it's size, so it is chunked.
    ChunkedBody(&'a mut Reader + 'a),
    /// A String has a size, and uses Content-Length.
    SizedBody(BufReader<'a>, uint),
}

impl<'a> Body<'a> {
    fn size(&self) -> Option<uint> {
        match *self {
            Body::SizedBody(_, len) => Some(len),
            _ => None
        }
    }
}

impl<'a> Reader for Body<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        match *self {
            Body::ChunkedBody(ref mut r) => r.read(buf),
            Body::SizedBody(ref mut r, _) => r.read(buf),
        }
    }
}

impl<'a> IntoBody<'a> for &'a [u8] {
    #[inline]
    fn into_body(self) -> Body<'a> {
        Body::SizedBody(BufReader::new(self), self.len())
    }
}

impl<'a> IntoBody<'a> for &'a str {
    #[inline]
    fn into_body(self) -> Body<'a> {
        self.as_bytes().into_body()
    }
}

impl<'a, R: Reader> IntoBody<'a> for &'a mut R {
    #[inline]
    fn into_body(self) -> Body<'a> {
        Body::ChunkedBody(self)
    }
}



fn get_host_and_port(url: &Url) -> HttpResult<(String, Port)> {
    let host = match url.serialize_host() {
        Some(host) => host,
        None => return Err(HttpUriError(UrlError::EmptyHost))
    };
    debug!("host={}", host);
    let port = match url.port_or_default() {
        Some(port) => port,
        None => return Err(HttpUriError(UrlError::InvalidPort))
    };
    debug!("port={}", port);
    Ok((host, port))
}

#[cfg(test)]
mod tests {
    use header::common::Server;
    use super::{Client, RedirectPolicy};
    use url::Url;

    mock_connector!(MockRedirectPolicy {
        "http://127.0.0.1" =>       "HTTP/1.1 301 Redirect\r\n\
                                     Location: http://127.0.0.2\r\n\
                                     Server: mock1\r\n\
                                     \r\n\
                                    "
        "http://127.0.0.2" =>       "HTTP/1.1 302 Found\r\n\
                                     Location: https://127.0.0.3\r\n\
                                     Server: mock2\r\n\
                                     \r\n\
                                    "
        "https://127.0.0.3" =>      "HTTP/1.1 200 OK\r\n\
                                     Server: mock3\r\n\
                                     \r\n\
                                    "
    })

    #[test]
    fn test_redirect_followall() {
        let mut client: Client<MockRedirectPolicy> = Client::new(RedirectPolicy::FollowAll);

        let res = client.get(Url::parse("http://127.0.0.1").unwrap()).unwrap();
        assert_eq!(res.headers.get(), Some(&Server("mock3".into_string())));
    }

    #[test]
    fn test_redirect_dontfollow() {
        let mut client: Client<MockRedirectPolicy> = Client::new(RedirectPolicy::FollowNone);
        let res = client.get(Url::parse("http://127.0.0.1").unwrap()).unwrap();
        assert_eq!(res.headers.get(), Some(&Server("mock1".into_string())));
    }

    #[test]
    fn test_redirect_followif() {
        fn follow_if(url: &Url) -> bool {
            !url.serialize()[].contains("127.0.0.3")
        }
        let mut client: Client<MockRedirectPolicy> = Client::new(RedirectPolicy::FollowIf(follow_if));
        let res = client.get(Url::parse("http://127.0.0.1").unwrap()).unwrap();
        assert_eq!(res.headers.get(), Some(&Server("mock2".into_string())));
    }

}
