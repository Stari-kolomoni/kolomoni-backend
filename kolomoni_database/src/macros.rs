/// Defines a struct whose sole purpose is wrapping an async [`Stream`],
/// mapping each item using a closure provided by the user.
///
/// # Example
/// For example, let's say we have a stream: [`BoxStream`]`<'a, i32>`,
/// but we want to process each `i32` item, turning it into e.g. `Result<u32, TryFromIntError>`.
///
/// This is possible by wrapping the stream in a custom struct, using the [`pin_project_lite`]
/// crate for pin projection of the wrapped stream, then implementing [`Stream`] on the custom struct.
///
/// *This is precisely what this macro aims to simplify.*
///
/// ```rust,no_run
/// use crate::macros::create_async_stream_wrapper;
/// use std::num::TryFromIntError;
///
/// use futures_core::BoxStream;
///
///
/// type OriginalStreamType<'c> = BoxStream<'c, i32>;
///
/// create_async_stream_wrapper!(
///     pub struct UnsignedIntStream<'c>;
///     transforms OriginalStreamType<'c> => Result<u32, TryFromIntError>:
///         |value| u32::try_from(value)
/// );
///
///
/// fn foo() {
///     // `original_stream` is a boxed stream of `i32` values
///     // (how we get *that* stream is not relevant here).
///     let original_stream: BoxStream<'_, i32> = todo!();
///
///     // The `new` method is generated that takes the original stream.
///     let transformed_stream = UnsignedIntStream::new(original_stream);
///     
///     // ... use `transformed_stream` as a normal async stream ...
/// }
/// ```
///
/// And voila, we have just successfully wrapped a stream,
/// transforming each item using a closure we provided.
///
///
/// [`Stream`]: futures_core::Stream
/// [`BoxStream`]: futures_core::BoxStream
macro_rules! create_async_stream_wrapper {
    (
        $struct_visibility:vis struct $struct_identifier:ident<$struct_lifetime:lifetime>;
        transforms stream $wrapped_type:ty => stream of $resulting_type:ty:
            |$captured_value:ident| $mapper:expr
    ) => {
        pin_project_lite::pin_project! {
            $struct_visibility struct $struct_identifier<$struct_lifetime> {
                #[pin]
                wrapped: $wrapped_type
            }
        }

        impl<$struct_lifetime> $struct_identifier<$struct_lifetime> {
            #[inline]
            fn new(wrapped: $wrapped_type) -> Self {
                Self { wrapped }
            }
        }

        impl<$struct_lifetime> futures_core::Stream for $struct_identifier<$struct_lifetime> {
            type Item = $resulting_type;

            fn poll_next(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Option<Self::Item>> {
                let this = self.project();

                match <$wrapped_type as futures_core::Stream>::poll_next(this.wrapped, cx) {
                    std::task::Poll::Ready($captured_value) => std::task::Poll::Ready($mapper),
                    std::task::Poll::Pending => std::task::Poll::Pending,
                }
            }
        }
    };
}
