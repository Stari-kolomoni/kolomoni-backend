#[macro_export]
macro_rules! impl_json_responder {
    ($struct:ty, $name:literal) => {
        impl Responder for $struct {
            type Body = BoxBody;

            fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
                HttpResponse::Ok().json(&self)
            }
        }
    };
}

/// Result unwrapping macro that can be used to handle `Result`s
/// at the endpoint-level.
///
/// This macro can have one, two or three parameters.
/// The first parameter
#[macro_export]
macro_rules! unwrap_result_with_log_and_http_500 {
    ($result:expr) => {
        match $result {
            Ok(inner) => inner,
            Err(error) => {
                error!(
                    error = error.to_string(),
                    "Errored at root of endpoint."
                );

                return HttpResponse::InternalServerError().finish();
            }
        }
    };

    ($result:expr, $username:expr) => {
        match $result {
            Ok(inner) => inner,
            Err(error) => {
                error!(
                    error = error.to_string(),
                    username = $username,
                    $error_message,
                );

                return HttpResponse::InternalServerError().finish();
            }
        }
    };

    ($result:expr, $error_message:literal) => {
        match $result {
            Ok(inner) => inner,
            Err(error) => {
                error!(error = error.to_string(), $error_message,);

                return HttpResponse::InternalServerError().finish();
            }
        }
    };

    ($result:expr, $username:expr, $error_message:literal) => {
        match $result {
            Ok(inner) => inner,
            Err(error) => {
                error!(
                    error = error.to_string(),
                    username = $username,
                    $error_message,
                );

                return HttpResponse::InternalServerError().finish();
            }
        }
    };
}
