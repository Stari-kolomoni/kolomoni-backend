use actix_web::body::MessageBody;
use actix_web::HttpResponse;

pub trait DumbResponder {
    type Body: MessageBody + 'static;

    /// Serialize self as JSON and return a `HTTP 200 OK` response with JSON-encoded body.  
    fn into_response(self) -> HttpResponse<Self::Body>;
}

/// Macro that implements two traits:
/// - `actix_web::Responder` that allows you to return this struct in an endpoint handler, and
/// - `DumbResponder`, which is a simpler internal trait that has the `into_response` method that
///   does basically the same as `actix_web::Responder::respond_to`, but without having to provide
///   a reference to `HttpRequest`, making code cleaner.
///
/// The provided struct must already implement `Serialize`.
#[macro_export]
macro_rules! impl_json_responder {
    ($struct:ty) => {
        impl Responder for $struct {
            type Body = BoxBody;

            fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
                HttpResponse::Ok().json(&self)
            }
        }

        impl DumbResponder for $struct {
            type Body = BoxBody;

            fn into_response(self) -> HttpResponse<Self::Body> {
                HttpResponse::Ok().json(&self)
            }
        }
    };
}

/// A shorthand for responding with a given status code and a JSON
/// body containing the `reason` String field.
///
/// First argument is the `actix_web::StatusCode` status code and
/// the second argument is the reason to respond with (must implement `Into<String>`).
#[macro_export]
macro_rules! response_with_reason {
    ($status_code:expr, $reason:expr) => {
        HttpResponseBuilder::new($status_code).json(ErrorReasonResponse::custom_reason($reason))
    };
}
