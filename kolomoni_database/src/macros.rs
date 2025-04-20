/// Defines a struct whose sole purpose is wrapping and transforming an async [`Stream`],
/// i.e. mapping each item using a closure provided by the user.
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
/// use futures_core::stream::BoxStream;
/// use std::num::TryFromIntError;
///
/// use kolomoni_database::macros::create_mapped_async_stream;
///
///
/// type OriginalStreamType<'c> = BoxStream<'c, i32>;
///
/// create_mapped_async_stream!(
///     pub struct UnsignedIntStream<'c>;
///     transforms stream OriginalStreamType<'c> => stream of Result<u32, TryFromIntError>:
///         |value| value.map(TryFrom::try_from)
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
#[macro_export]
macro_rules! create_mapped_async_stream {
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

pub use create_mapped_async_stream;



/// Expands to an expression that tries to deserialize the given `serde_json::Value` as
/// a given `Deserialize`-implementing type.
///
/// This is preferred to a classic [`serde_json::from_value`] in most cases when deserializing
/// a JSON from a database query, because this way we can generate much nicer errors.
/// In debug mode, this macro will actually return the deserialization error of a pretty-printed
/// JSON, allowing us to see the line information. In release mode, the expanded code is essentially
/// just a [`serde_json::from_value`] with some default message formatting.
///
/// # Example
/// The input to this macro consists of an input variable, the expected output type,
/// and, optionally, a closure to generate a better error message string.
///
/// First off, the simplest use:
/// ```rust,no_run
/// use kolomoni_database::macros::deserialize_json_from_value;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// pub struct SomeTargetStruct {
///     some_field: i64
/// }
///
///
/// fn main() -> Result<(), String> {
///     // Consider this any valid `serde_json::Value`.
///     // This deserialization would fail in our case, as `null`
///     // doesn't match `SomeTargetStruct`, but that's not the part we're trying to showcase.
///     let json_value = serde_json::Value::Null;
///
///     let target_struct = deserialize_json_from_value!(
///         json_value => SomeTargetStruct
///     )?;
///
///     println!("{:?}", target_struct);
///
///     Ok(())
/// }
/// ```
///
/// You may also specify a custom error message closure:
/// ```rust,no_run
/// use kolomoni_database::macros::deserialize_json_from_value;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// pub struct SomeTargetStruct {
///     some_field: i64
/// }
///
///
/// fn main() -> Result<(), String> {
///     // Consider this any valid `serde_json::Value`.
///     // This deserialization would fail in our case, as `null`
///     // doesn't match `SomeTargetStruct`, but that's not the part we're trying to showcase.
///     let json_value = serde_json::Value::Null;
///
///     // Consider this some external context we may want to use to format our error message.
///     let some_context: u64 = 1234;
///
///     let target_struct = deserialize_json_from_value!(
///         json_value => SomeTargetStruct,
///         with message: |error| {
///             format!(
///                 "failed to parse this example ({}): {:?}",
///                 some_context,
///                 error
///             )
///         }
///     )?;
///
///     println!("{:?}", target_struct);
///
///     Ok(())
/// }
/// ```
///
/// ---
///
/// If unspecified, the default message formatter is:
/// ```rust,ignore
/// format!(
///     "failed to parse JSON value as valid {}: {}",
///     your-destination-type-quoted-here,
///     deserialization-error
/// )
/// ```
#[macro_export]
macro_rules! deserialize_json_from_value {
    ($value:expr => $value_type:ty) => {{
        #[cfg(debug_assertions)]
        {
            match <$value_type as serde::de::Deserialize>::deserialize(&$value) {
                Ok(deserialized_value) => Ok(deserialized_value),
                Err(deserialization_error) => {
                    // Pseudocode:
                    // - pretty print the input (so we get better line-column diagnostics),
                    // - attempt to deserialize again (this will fail)
                    // - output an error with the better diagnostics.
                    let pretty_printed_value: String = serde_json::to_string_pretty(&$value)
                        .expect("failed to pretty-print valid JSON value");

                    let pretty_deserialization_error: serde_json::Error = serde_json::from_str(
                        &pretty_printed_value,
                    )
                    .expect_err(
                        "when re-deserializing the pretty-printed JSON value, no error occurred?!?!",
                    );

                    Err(format!(
                        "failed to parse JSON value as valid {}: {}",
                        stringify!($value_type),
                        pretty_deserialization_error
                    ))
                }
            }
        }

        #[cfg(not(debug_assertions))]
        {
            serde_json::from_value::<$value_type>($value).map_err(|error| {
                format!(
                    "failed to parse JSON value as valid {}: {}",
                    stringify!($value_type),
                    error
                )
            })
        }
    }};

    ($value:expr => $value_type:ty; with message: |$deserialization_error:ident| $message_expr:expr) => {{
        #[cfg(debug_assertions)]
        {
            match <$value_type as serde::de::Deserialize>::deserialize(&$value) {
                Ok(deserialized_value) => Ok(deserialized_value),
                Err(_) => {
                    // Pseudocode:
                    // - pretty print the input (so we get better line-column diagnostics),
                    // - attempt to deserialize again (this will fail)
                    // - output an error with the better diagnostics.
                    let pretty_printed_value: String = serde_json::to_string_pretty(&$value)
                        .expect("failed to pretty-print valid JSON value");

                    let pretty_deserialization_error: serde_json::Error = serde_json::from_str::<
                        $value_type,
                    >(
                        &pretty_printed_value
                    )
                    .expect_err(
                        "when re-deserializing the pretty-printed JSON value, no error occurred?!?!",
                    );

                    let message_generator_closure = |$deserialization_error| $message_expr;

                    Err(format!(
                        "{}\n{}",
                        message_generator_closure(pretty_deserialization_error,),
                        pretty_printed_value
                    ))
                }
            }
        }

        #[cfg(not(debug_assertions))]
        {
            serde_json::from_value::<$value_type>($value).map_err(|error| {
                let message_generator_closure = |$deserialization_error| $message_expr;

                message_generator_closure(error)
            })
        }
    }};
}


pub use deserialize_json_from_value;
