#[macro_export]
macro_rules! impl_json_responder_on_serializable {
    ($struct:ty, $name:literal) => {
        impl Responder for $struct {
            type Body = BoxBody;

            fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
                match serde_json::to_string(&self) {
                    Ok(body) => HttpResponse::Ok()
                        .content_type(ContentType::json())
                        .body(body),
                    Err(error) => {
                        error!(
                            error = error.to_string(),
                            "Failed to encode {} to JSON.", $name,
                        );
                        HttpResponse::InternalServerError().finish()
                    }
                }
            }
        }
    };
}
