use std::future::Future;
use std::marker::PhantomData;

use crate::client::errors::RequestError;
use crate::response::raw::RawResponse;
use crate::response::ResponseValueError;
use crate::BoxFuture;

/// Represents a strongly-typed server response.
///
/// Internally, it stores an async closure that takes the
/// raw HTTP response and parses it into the final value - a [`ResponseValueResult`].
///
/// Errors from previous stages of this request (even errors from executing the HTTP request itself)
/// are bubbled into this struct, which is why [`try_into_value`] is fallible.
///
/// Once again: due to intentional error bubbling, holding this struct **does not** indicate
/// that the HTTP request was executed at all, you need to either inspect
/// the raw response ([`Self::try_into_raw`]) or
/// parse the output value ([`Self::try_into_value`]) to know.
pub struct TypedResponse<V, E>
where
    E: ResponseValueError,
{
    /// Contains a fallible [`RawServerResponse`]. We bubble this potential [`ClientError`]
    /// into this struct to avoid having to try or match on the error multiple times in a single request.
    response: Result<RawResponse, RequestError>,

    /// A boxed async closure that takes a raw HTTP response and parses it into either the strongly-typed result `V`,
    /// the custom error (`E`), an error with reason ([`ErrorReason`]), or a general client error ([`ClientError`]).
    ///
    /// See also: [`ResponseValueResult`].
    parser_closure: Box<dyn FnOnce(RawResponse) -> BoxFuture<'static, Result<V, E>>>,

    _marker_v: PhantomData<V>,
    _marker_e: PhantomData<E>,
}

impl<V, E> TypedResponse<V, E>
where
    E: ResponseValueError,
{
    /// Construct a new lazy typed server response by providing
    /// an async closure that will parse the raw HTTP response
    /// into the expected type.
    ///
    /// The closure will be called lazily when the user requests
    /// the output value. This is because the closure will actually
    /// consume the response.
    pub(crate) fn new<F, Fut>(response: Result<RawResponse, RequestError>, parser_closure: F) -> Self
    where
        F: FnOnce(RawResponse) -> Fut + 'static,
        Fut: Future<Output = Result<V, E>> + Send + 'static,
    {
        let prepared_closure: Box<dyn FnOnce(RawResponse) -> BoxFuture<'static, Result<V, E>>> =
            Box::new(move |data: RawResponse| Box::pin(parser_closure(data)));

        Self {
            response,
            parser_closure: prepared_closure,
            _marker_v: PhantomData,
            _marker_e: PhantomData,
        }
    }

    /// This is a lossy operation, because both the success type `V`
    /// and the parsing closure `F` is lost.
    pub fn try_into_raw(self) -> Result<RawResponse, RequestError> {
        self.response
    }

    /// Consume this response and try to obtain the output value.
    ///
    /// This operation is fallible for many reasons:
    /// - the request may have failed even to execute due to network or serialization errors,
    /// - the request may have been completed, but the operation was not successful
    ///   (e.g. custom error specific to this endpoint, error with a reason provided by server,
    ///   internal server error, ...)
    ///
    /// and so on.
    pub async fn try_into_value(self) -> Result<V, E> {
        let raw_response = match self.response {
            Ok(raw_response) => raw_response,
            Err(client_error) => return Err(E::from_request_error(client_error)),
        };

        (self.parser_closure)(raw_response).await
    }
}



/// Implementors of this trait must provide a single method:
/// [`into_typed_response`](IntoTypedResponse::into_typed_response),
/// which must consume `self` and infallibly convert into a [`TypedServerResponse`].
pub trait IntoTypedResponse<V, E>
where
    E: ResponseValueError,
{
    fn into_typed_response<F, Fut>(self, parser_closure: F) -> TypedResponse<V, E>
    where
        F: FnOnce(RawResponse) -> Fut + 'static,
        Fut: Future<Output = Result<V, E>> + Send + 'static;
}

/// This impl is useful for directly converting the output of
/// an executed request (the return value of e.g. [`PreparedGetRequest::execute_authenticated`])
/// into a [`TypedServerResponse`] without having to try or match the returned [`ClientResult`].
///
/// This is basically a cleaner postfix version of [`TypedServerResponse::new`].
impl<V, E> IntoTypedResponse<V, E> for Result<RawResponse, RequestError>
where
    E: ResponseValueError,
{
    fn into_typed_response<F, Fut>(self, parser_closure: F) -> TypedResponse<V, E>
    where
        F: FnOnce(RawResponse) -> Fut + 'static,
        Fut: Future<Output = Result<V, E>> + Send + 'static,
    {
        TypedResponse::<V, E>::new(self, parser_closure)
    }
}

impl<V, E> IntoTypedResponse<V, E> for RawResponse
where
    E: ResponseValueError,
{
    fn into_typed_response<F, Fut>(self, parser_closure: F) -> TypedResponse<V, E>
    where
        F: FnOnce(RawResponse) -> Fut + 'static,
        Fut: Future<Output = Result<V, E>> + Send + 'static,
    {
        TypedResponse::<V, E>::new(Ok(self), parser_closure)
    }
}
