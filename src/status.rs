#![allow(non_upper_case_globals)]

//! Status Codes
use std::fmt;
use std::mem::transmute;

// shamelessly lifted from Teepee. I tried a few schemes, this really
// does seem like the best.

/// An HTTP status code (`Status-Code` in RFC 2616).
///
/// This enum is absolutely exhaustive, covering all 500 possible values (100–599).
///
/// For HTTP/2.0, statuses belonging to the 1xx Informational class are invalid.
///
/// As this is a C‐style enum with each variant having a corresponding value, you may use the likes
/// of `Continue as u16` to retreive the value `100u16`. Normally, though, you should not need to do
/// any such thing; just use the status code as a `StatusCode`.
///
/// If you encounter a status code that you do not know how to deal with, you should treat it as the
/// `x00` status code—e.g. for code 123, treat it as 100 (Continue). This can be achieved with
/// `self.class().default_code()`:
///
/// ```rust
/// # use hyper::status::{Continue, StatusCode};
/// assert_eq!(StatusCode::from_code(123).class().default_code(), Continue);
/// ```


pub struct StatusCode( u16 );

macro_rules! def_status_codes(
    ($($name:ident = $val:expr,)+) => (
        $(
            #[allow(non_upper_case_globals)]
            pub const $name: StatusCode = StatusCode ( $val );
        )+
    );
)
def_status_codes!(
    // 100 Continue
    Continue = 100,
    // 101 SwitchingProtocols
    SwitchingProtocols = 101,
    // 102 Processing
    Processing = 102,
    // 200 OK
    Ok = 200,
    // 201 Created
    Created = 201,
    // 202 Accepted
    Accepted = 202,
    // 203 Non-Authoritative Information
    NonAuthoritativeInformation = 203,
    // 204 No Content
    NoContent = 204,
    // 205 Reset Content
    ResetContent = 205,
    // 206 Partial Content
    PartialContent = 206,
    // 207 Multi-Status
    MultiStatus = 207,
    // 208 Already Reported
    AlreadyReported = 208,
    // 226 IM Used
    ImUsed = 226,
    // 300 Multiple Choices
    MultipleChoices = 300,
    // 301 Moved Permanently
    MovedPermanently = 301,
    // 302 Found
    Found = 302,
    // 303 See Other
    SeeOther = 303,
    // 304 Not Modified
    NotModified = 304,
    // 305 Use Proxy
    UseProxy = 305,
    // 306 Switch Proxy
    SwitchProxy = 306,
    // 307 Temporary Redirect
    TemporaryRedirect = 307,
    // 308 Permanent Redirect
    PermanentRedirect = 308,
    // 400 Bad Request
    BadRequest = 400,
    // 401 Unauthorized
    Unauthorized = 401,
    // 402 Payment Required
    PaymentRequired = 402,
    // 403 Forbidden
    Forbidden = 403,
    // 404 Not Found
    NotFound = 404,
    // 405 Method Not Allowed
    MethodNotAllowed = 405,
    // 406 Not Acceptable
    NotAcceptable = 406,
    // 407 Proxy Authentication Required
    ProxyAuthenticationRequired = 407,
    // 408 Request Timeout
    RequestTimeout = 408,
    // 409 Conflict
    Conflict = 409,
    // 410 Gone
    Gone = 410,
    // 411 Length Required
    LengthRequired = 411,
    // 412 Precondition Failed
    PreconditionFailed = 412,
    // 413 Request Entity Too Large
    RequestEntityTooLarge = 413,
    // 414 Request-URI Too Long
    RequestUriTooLong = 414,
    // 415 Unsupported Media Type
    UnsupportedMediaType = 415,
    // 416 Requested Range Not Satisfiable
    RequestedRangeNotSatisfiable = 416,
    // 417 Expectation Failed
    ExpectationFailed = 417,
    // 418 I'm a teapot
    ImATeapot = 418,
    // 419 Authentication Timeout
    AuthenticationTimeout = 419,
    // 422 Unprocessable Entity
    UnprocessableEntity = 422,
    // 423 Locked
    Locked = 423,
    // 424 Failed Dependency
    FailedDependency = 424,
    // 425 Unordered Collection
    UnorderedCollection = 425,
    // 426 Upgrade Required
    UpgradeRequired = 426,
    // 427 (unregistered)
    Code427 = 427,
    // 428 Precondition Required
    PreconditionRequired = 428,
    // 429 Too Many Requests
    TooManyRequests = 429,
    // 430 (unregistered)
    Code430 = 430,
    // 431 Request Header Fields Too Large
    RequestHeaderFieldsTooLarge = 431,
    // 451 Unavailable For Legal Reasons
    UnavailableForLegalReasons = 451,
    // 500 Internal Server Error
    InternalServerError = 500,
    // 501 Not Implemented
    NotImplemented = 501,
    // 502 Bad Gateway
    BadGateway = 502,
    // 503 Service Unavailable
    ServiceUnavailable = 503,
    // 504 Gateway Timeout
    GatewayTimeout = 504,
    // 505 HTTP Version Not Supported
    HttpVersionNotSupported = 505,
    // 506 Variant Also Negotiates
    VariantAlsoNegotiates = 506,
    // 507 Insufficient Storage
    InsufficientStorage = 507,
    // 508 Loop Detected
    LoopDetected = 508,
    // 510 Not Extended
    NotExtended = 510,
    // 511 Network Authentication Required
    NetworkAuthenticationRequired = 511,
)

impl StatusCode {

    /// StatusCode from numeric value
    pub fn from_code(code : u16) -> StatusCode {
        // TODO: checks? Return Option<StatusCode> ?
        StatusCode (code)
    }

    /// Get the standardised `Reason-Phrase` for this status code.
    ///
    /// This is mostly here for servers writing responses, but could potentially have application at
    /// other times.
    ///
    /// The reason phrase is defined as being exclusively for human readers. You should avoid
    /// deriving any meaning from it at all costs.
    ///
    /// Bear in mind also that in HTTP/2.0 the reason phrase is abolished from transmission, and so
    /// this canonical reason phrase really is the only reason phrase you’ll find.
    pub fn canonical_reason(&self) -> Option<&'static str> {
        match *self {
            Continue => Some("Continue"),
            SwitchingProtocols => Some("Switching Protocols"),
            Processing => Some("Processing"),
            Ok => Some("OK"),
            Created => Some("Created"),
            Accepted => Some("Accepted"),
            NonAuthoritativeInformation => Some("Non-Authoritative Information"),
            NoContent => Some("No Content"),
            ResetContent => Some("Reset Content"),
            PartialContent => Some("Partial Content"),
            MultiStatus => Some("Multi-Status"),
            AlreadyReported => Some("Already Reported"),
            ImUsed => Some("IM Used"),
            MultipleChoices => Some("Multiple Choices"),
            MovedPermanently => Some("Moved Permanently"),
            Found => Some("Found"),
            SeeOther => Some("See Other"),
            NotModified => Some("Not Modified"),
            UseProxy => Some("Use Proxy"),
            SwitchProxy => Some("Switch Proxy"),
            TemporaryRedirect => Some("Temporary Redirect"),
            PermanentRedirect => Some("Permanent Redirect"),
            BadRequest => Some("Bad Request"),
            Unauthorized => Some("Unauthorized"),
            PaymentRequired => Some("Payment Required"),
            Forbidden => Some("Forbidden"),
            NotFound => Some("Not Found"),
            MethodNotAllowed => Some("Method Not Allowed"),
            NotAcceptable => Some("Not Acceptable"),
            ProxyAuthenticationRequired => Some("Proxy Authentication Required"),
            RequestTimeout => Some("Request Timeout"),
            Conflict => Some("Conflict"),
            Gone => Some("Gone"),
            LengthRequired => Some("Length Required"),
            PreconditionFailed => Some("Precondition Failed"),
            RequestEntityTooLarge => Some("Request Entity Too Large"),
            RequestUriTooLong => Some("Request-URI Too Long"),
            UnsupportedMediaType => Some("Unsupported Media Type"),
            RequestedRangeNotSatisfiable => Some("Requested Range Not Satisfiable"),
            ExpectationFailed => Some("Expectation Failed"),
            ImATeapot => Some("I'm a teapot"),
            AuthenticationTimeout => Some("Authentication Timeout"),
            UnprocessableEntity => Some("Unprocessable Entity"),
            Locked => Some("Locked"),
            FailedDependency => Some("Failed Dependency"),
            UnorderedCollection => Some("Unordered Collection"),
            UpgradeRequired => Some("Upgrade Required"),
            PreconditionRequired => Some("Precondition Required"),
            TooManyRequests => Some("Too Many Requests"),
            RequestHeaderFieldsTooLarge => Some("Request Header Fields Too Large"),

            UnavailableForLegalReasons => Some("Unavailable For Legal Reasons"),
            InternalServerError => Some("Internal Server Error"),
            NotImplemented => Some("Not Implemented"),
            BadGateway => Some("Bad Gateway"),
            ServiceUnavailable => Some("Service Unavailable"),
            GatewayTimeout => Some("Gateway Timeout"),
            HttpVersionNotSupported => Some("HTTP Version Not Supported"),
            VariantAlsoNegotiates => Some("Variant Also Negotiates"),
            InsufficientStorage => Some("Insufficient Storage"),
            LoopDetected => Some("Loop Detected"),
            NotExtended => Some("Not Extended"),
            NetworkAuthenticationRequired => Some("Network Authentication Required"),
            _ => None,
        }
    }

    /// Determine the class of a status code, based on its first digit.
    pub fn class(&self) -> StatusClass {
        let StatusCode(code) = *self;  // Range of possible values: 100..599.
        // We could match 100..199 &c., but this way we avoid unreachable!() at the end.
        if code < 200 {
            StatusClass::Informational
        } else if code < 300 {
            StatusClass::Success
        } else if code < 400 {
            StatusClass::Redirection
        } else if code < 500 {
            StatusClass::ClientError
        } else {
            StatusClass::ServerError
        }
    }
}

impl Copy for StatusCode {}

/// Formats the status code, *including* the canonical reason.
///
/// ```rust
/// # use hyper::status::{ImATeapot, StatusCode};
/// assert_eq!(format!("{}", ImATeapot).as_slice(),
///            "418 I'm a teapot");
/// assert_eq!(format!("{}", StatusCode::from_code(123)).as_slice(),
///            "123 <unknown status code>");
/// ```
///
/// If you wish to just include the number, cast to a u16 instead.
impl fmt::Show for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let StatusCode(code) = *self;
        write!(f, "{} {}", code,
               self.canonical_reason().unwrap_or("<unknown status code>"))
    }
}

// Specified manually because the codegen for derived is slow (at the time of writing on the machine
// of writing, 1.2 seconds) and verbose (though the optimiser cuts it down to size).
impl PartialEq for StatusCode {
    #[inline]
    fn eq(&self, other: &StatusCode) -> bool {
        let StatusCode(self_code) = *self;
        let StatusCode(other_code) = *other;
        self_code == other_code
    }
}

impl Eq for StatusCode {}

// Ditto (though #[deriving(Clone)] only takes about 0.4 seconds).
impl Clone for StatusCode {
    #[inline]
    fn clone(&self) -> StatusCode {
        let StatusCode(code) = *self;
        StatusCode(code)
    }
}

// Of the other common derivable traits, I didn’t measure them, but I guess they would be slow too.

impl FromPrimitive for StatusCode {
    fn from_i64(n: i64) -> Option<StatusCode> {
        if n < 100 || n > 599 {
            None
        } else {
            Some(unsafe { transmute::<u16, StatusCode>(n as u16) })
        }
    }

    fn from_u64(n: u64) -> Option<StatusCode> {
        if n < 100 || n > 599 {
            None
        } else {
            Some(unsafe { transmute::<u16, StatusCode>(n as u16) })
        }
    }
}

impl PartialOrd for StatusCode {
    #[inline]
    fn partial_cmp(&self, other: &StatusCode) -> Option<Ordering> {
        let StatusCode(code) = *self;
        let StatusCode(other_code) = *other;
        code.partial_cmp(&other_code)
    }
}

impl Ord for StatusCode {
    #[inline]
    fn cmp(&self, other: &StatusCode) -> Ordering {
        if *self < *other {
            Less
        } else if *self > *other {
            Greater
        } else {
            Equal
        }
    }
}

impl ToPrimitive for StatusCode {
    fn to_i64(&self) -> Option<i64> {
        let StatusCode(code) = *self;
        Some(code as i64)
    }

    fn to_u64(&self) -> Option<u64> {
        let StatusCode(code) = *self;
        Some(code as u64)
    }
}

/// The class of an HTTP `Status-Code`.
///
/// [RFC 2616, section 6.1.1 (Status Code and Reason
/// Phrase)](https://tools.ietf.org/html/rfc2616#section-6.1.1):
///
/// > The first digit of the Status-Code defines the class of response. The
/// > last two digits do not have any categorization role.
/// >
/// > ...
/// >
/// > HTTP status codes are extensible. HTTP applications are not required
/// > to understand the meaning of all registered status codes, though such
/// > understanding is obviously desirable. However, applications MUST
/// > understand the class of any status code, as indicated by the first
/// > digit, and treat any unrecognized response as being equivalent to the
/// > x00 status code of that class, with the exception that an
/// > unrecognized response MUST NOT be cached. For example, if an
/// > unrecognized status code of 431 is received by the client, it can
/// > safely assume that there was something wrong with its request and
/// > treat the response as if it had received a 400 status code. In such
/// > cases, user agents SHOULD present to the user the entity returned
/// > with the response, since that entity is likely to include human-
/// > readable information which will explain the unusual status.
///
/// This can be used in cases where a status code’s meaning is unknown, also,
/// to get the appropriate *category* of status.
///
/// For HTTP/2.0, the 1xx Informational class is invalid.
#[deriving(Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum StatusClass {
    /// 1xx: Informational - Request received, continuing process
    Informational = 100,

    /// 2xx: Success - The action was successfully received, understood, and accepted
    Success = 200,

    /// 3xx: Redirection - Further action must be taken in order to complete the request
    Redirection = 300,

    /// 4xx: Client Error - The request contains bad syntax or cannot be fulfilled
    ClientError = 400,

    /// 5xx: Server Error - The server failed to fulfill an apparently valid request
    ServerError = 500,
}

impl StatusClass {
    /// Get the default status code for the class.
    ///
    /// This produces the x00 status code; thus, for `ClientError` (4xx), for example, this will
    /// produce `BadRequest` (400):
    ///
    /// ```rust
    /// # use hyper::status::StatusClass::ClientError;
    /// # use hyper::status::BadRequest;
    /// assert_eq!(ClientError.default_code(), BadRequest);
    /// ```
    ///
    /// The use for this is outlined in [RFC 2616, section 6.1.1 (Status Code and Reason
    /// Phrase)](https://tools.ietf.org/html/rfc2616#section-6.1.1):
    ///
    /// > HTTP status codes are extensible. HTTP applications are not required
    /// > to understand the meaning of all registered status codes, though such
    /// > understanding is obviously desirable. However, applications MUST
    /// > understand the class of any status code, as indicated by the first
    /// > digit, and treat any unrecognized response as being equivalent to the
    /// > x00 status code of that class, with the exception that an
    /// > unrecognized response MUST NOT be cached. For example, if an
    /// > unrecognized status code of 431 is received by the client, it can
    /// > safely assume that there was something wrong with its request and
    /// > treat the response as if it had received a 400 status code. In such
    /// > cases, user agents SHOULD present to the user the entity returned
    /// > with the response, since that entity is likely to include human-
    /// > readable information which will explain the unusual status.
    ///
    /// This is demonstrated thusly (I’ll use 432 rather than 431 as 431 *is* now in use):
    ///
    /// ```rust
    /// # use hyper::status::{StatusCode, BadRequest};
    /// // Suppose we have received this status code.
    /// let status = StatusCode::from_code(432);
    ///
    /// // Uh oh! Don’t know what to do with it.
    /// // Let’s fall back to the default:
    /// let status = status.class().default_code();
    ///
    /// // And look! That is 400 Bad Request.
    /// assert_eq!(status, BadRequest);
    /// // So now let’s treat it as that.
    /// ```
    #[inline]
    pub fn default_code(&self) -> StatusCode {
        unsafe { transmute::<StatusClass, StatusCode>(*self) }
    }
}

impl ToPrimitive for StatusClass {
    fn to_i64(&self) -> Option<i64> {
        Some(*self as i64)
    }

    fn to_u64(&self) -> Option<u64> {
        Some(*self as u64)
    }
}
