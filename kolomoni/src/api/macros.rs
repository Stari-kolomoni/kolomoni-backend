//! Macros to avoid repeating code (JSON response builders, authentication-related macros).

use actix_web::http::header::HeaderValue;
use chrono::{DateTime, Utc};


/// Given a `last_modification_time`, this function tries to construct
/// a [`HeaderValue`] corresponding to the `Last-Modified` header name.
///
/// The reason this function exists is because the date and time format is a bit peculiar.
///
/// See [Last-Modified documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Last-Modified).
pub fn construct_last_modified_header_value(last_modification_time: &DateTime<Utc>) -> HeaderValue {
    let date_time_formatter = last_modification_time.format("%a, %d %b %Y %H:%M:%S GMT");

    // PANIC SAFETY: Using our date time formatter ensures
    // we only emit visible ASCII characters (32-127), therefore `HeaderValue::from_str`
    // can't panic.
    HeaderValue::from_str(date_time_formatter.to_string().as_str()).unwrap()
}




/// An endpoint handler macro that requires a user to be authenticated.
///
/// It expands to a check that will early-return a `401 Unauthorized`
/// with [`ErrorReason::missing_authentication()`] unless the user is authenticated.
///
/// The associated documentation type to use on the endpoint
/// that uses this macro is [`MissingAuthentication`].
///
/// # Usage
/// The macro expects a single parameter - a [`UserAuthenticationExtractor`] instance.
/// The macro returns an [`AuthenticatedUser`].
///
///
/// # Example
/// ```no_run
/// use kolomoni::api::openapi;
/// use kolomoni::api::errors::EndpointResult;
/// use kolomoni::authentication::UserAuthenticationExtractor;
/// use kolomoni::authentication::AuthenticatedUser;
///
/// #[utoipa::path(
///     get,
///     path = "/",
///     responses(
///         // The [`MissingAuthentication`] OpenAPI responses type goes
///         // hand in hand with this macro.
///         openapi::response::MissingAuthentication
///     )
/// )]
/// #[actix_web::get("/")]
/// async fn fetch_something(
///     // The presence of this extractor **DOES NOT** imply that the body
///     // of this function will execute only when the user is authenticated -
///     // we need to use the extractor instance to check for ourselves; it simply
///     // exposes authentication information to us.
///     authentication_extractor: UserAuthenticationExtractor,
/// ) -> EndpointResult {
///     // ...
///     
///     // This macro invocation will expand to an authentication check
///     // and will early-return if the user is not authenticated.
///     // It returns an [`AuthenticatedUser`].
///     let authenticated_user: AuthenticatedUser = require_user_authentication!(
///         authentication_extractor
///     );
///
///     println!("Authenticated user: {}", authenticated_user.user_id());
///
///     // ...
///     # todo!();
/// }
/// ```
///
///
/// # Side Effects
/// This macro does not perform any database lookups or any other IO,
/// it simply checks whether an access token was provided in the request.
///
///
/// [`MissingAuthentication`]: crate::api::openapi::response::MissingAuthentication
/// [`ErrorReason::missing_authentication()`]: crate::api::errors::ErrorReason::missing_authentication
/// [`UserAuthenticationExtractor`]: crate::authentication::UserAuthenticationExtractor
/// [`AuthenticatedUser`]: crate::authentication::AuthenticatedUser
#[macro_export]
macro_rules! require_user_authentication {
    ($user_auth_extractor:expr) => {
        match $user_auth_extractor.authenticated_user() {
            Some(user) => user,
            None => {
                return $crate::api::errors::EndpointResponseBuilder::unauthorized()
                    .with_error_reason($crate::api::errors::ErrorReason::missing_authentication())
                    .build();
            }
        }
    };
}


/// An endpoint handler macro that requires a set of permissions on the caller,
/// while *not necessarily* requiring user authentication.
///
/// It expands to the following pseudocode:
/// - First, blanket permission grants are checked (see [`BLANKET_PERMISSION_GRANT`]);
///   if all of the required permissions are blanket grants, we consider it a success -
///   the macro evaluates to `None`.
/// - Otherwise, the blanket grants aren't enough, so:
///     - If the user is authenticated, the permissions of the user will be merged with
///       the blanket permission grants. We'll then recheck whether the requested permission set is now satisfied:
///         - If the union of the user's permissions and the blanket grants covers all of the required permissions,
///           we consider it a success - the macro invocation will evaluate to `Some(`[`AuthenticatedUser`]`)`.
///         - Otherwise, the macro will early-return a `403 Forbidden`
///           response with [`ErrorReason::missing_permission(...)`].
///     - If the user is not authenticated (and the blanket grant wasn't enough),
///       the macro will early-return a `403 Forbidden` response with [`ErrorReason::missing_permission(...)`].
///
/// The associated documentation type to use on the endpoint
/// that uses this macro is [`MissingPermissions`].
///
/// # Usage
/// The macro expects three comma-separated parameters:
/// - A database connection. This must be of type [`&mut PgConnection`].
///   Note that for e.g. [`PoolConnection<Postgres>`], which is the most common type you'll encounter in endpoint handlers,
///   you can simply [mutably deref it] to [`&mut PgConnection`] with e.g. `&mut pool_connection`.
/// - User authentication extractor ([`UserAuthenticationExtractor`]).
/// - One or more permissions ([`Permission`]) to require, specified as an array (e.g. `[Permission::WordRead, Permission::WordUpdate]`).
///   If you require only one permission, you need not use the square brackets.
///
///
/// # Example
/// ```no_run
/// use kolomoni::api::openapi;
/// use kolomoni::api::openapi::response::requires;
/// use kolomoni::api::errors::EndpointResult;
/// use kolomoni::authentication::UserAuthenticationExtractor;
/// use kolomoni::authentication::AuthenticatedUser;
/// use kolomoni_auth::Permission;
///
/// #[utoipa::path(
///     get,
///     path = "/",
///     responses(
///         // The [`MissingPermissions`] OpenAPI responses type goes
///         // hand in hand with this macro.
///         openapi::response::MissingPermissions<requires::WordRead, 1>
///     )
/// )]
/// #[actix_web::get("/")]
/// async fn fetch_something(
///     state: ApplicationState,
///     // The presence of this extractor **DOES NOT** imply that the body
///     // of this function will execute only when the user is authenticated -
///     // we need to use the extractor instance to check for ourselves; it simply
///     // exposes authentication information to us.
///     authentication_extractor: UserAuthenticationExtractor,
/// ) -> EndpointResult {
///     // ...
///
///     let mut database_connection = state.acquire_database_connection().await?;
///     
///     // This macro invocation will expand to a permission check and will early-return
///     // if the blanket grants (or the user's permissions) don't satisfy the requirements.
///     // It returns an `Option<AuthenticatedUser>`.
///     let optional_authenticated_user = require_permission_with_optional_authentication!(
///         &mut database_connection,
///         authentication_extractor,
///         Permission::WordRead
///     );
///     
///     if let Some(authenticated_user) = optional_authenticated_user {
///         println!(
///             "Authenticated user (has WordRead permission): {}",
///             authenticated_user.user_id()
///         );
///     } else {
///         println!(
///             "User is not authenticated (but has WordRead as a blanket grant)."
///         );
///     }
///
///     // ...
///     # todo!();
/// }
/// ```
///
///
/// # Side Effects
/// This macro *may* perform a database lookup when the blanket grant
/// does not satisfy the requirement. Execution *may* therefore cross
/// an async yield point (`await`).
///
///
/// [`BLANKET_PERMISSION_GRANT`]: kolomoni_auth::BLANKET_PERMISSION_GRANT
/// [`ErrorReason::missing_permission(...)`]: crate::api::errors::ErrorReason::missing_permission
/// [`AuthenticatedUser`]: crate::authentication::AuthenticatedUser
/// [`MissingPermissions`]: crate::api::openapi::response::MissingPermissions
/// [`&mut PgConnection`]: sqlx::PgConnection
/// [`PoolConnection<Postgres>`]: sqlx::pool::PoolConnection
/// [mutably deref it]: https://docs.rs/sqlx/0.8.2/sqlx/pool/struct.PoolConnection.html#impl-AsMut%3C%3CDB+as+Database%3E::Connection%3E-for-PoolConnection%3CDB%3E
/// [`Permission`]: kolomoni_auth::Permission
/// [`UserAuthenticationExtractor`]: crate::authentication::UserAuthenticationExtractor
#[macro_export]
macro_rules! require_permission_with_optional_authentication {
    ($database_connection:expr, $user_auth_extractor:expr, $permission:expr) => {
        match $user_auth_extractor.authenticated_user() {
            Some(__authenticated_user) => Some($crate::require_permissions_on_user!(
                $database_connection,
                __authenticated_user,
                $permission
            )),
            None => {
                if !$user_auth_extractor.is_permission_granted_to_all($permission) {
                    return $crate::api::errors::EndpointResponseBuilder::forbidden()
                        .with_error_reason(
                            $crate::api::errors::ErrorReason::missing_permission($permission),
                        )
                        .build();
                }

                None
            }
        }
    };

    ($database_connection:expr, $user_auth_extractor:expr, [$($permission:expr),+]) => {
        match $user_auth_extractor.authenticated_user() {
            Some(__authenticated_user) => Some($crate::require_permissions_on_user!(
                $database_connection,
                __authenticated_user,
                [$($permission:expr),+]
            )),
            None => {
                let required_permission_set = PermissionSet::from_permissions(
                    &[$($permission:expr),+]
                );

                if !$user_auth_extractor.are_permissions_granted_to_all(&required_permission_set) {
                    return $crate::api::errors::EndpointResponseBuilder::new(
                        actix_web::http::StatusCode::FORBIDDEN,
                    )
                    .with_error_reason(
                        $crate::api::errors::ErrorReason::missing_permission($permission),
                    )
                    .build();
                }

                None
            }
        }
    };
}


/// An endpoint handler macro that requires some permissions on a [`PermissionSet`].
///
/// It expands to a check that verifies that all required permissions exist in
/// a permission set, otherwise early-returning a `403 Forbidden` HTTP response
/// with [`ErrorReason::missing_permission(...)`].
///
/// The associated documentation type to use on the endpoint
/// that uses this macro is [`MissingPermissions`].
///
///
/// # Usage
/// The macro expects two comma-separated parameters:
/// - A [`PermissionSet`] representing permissions to check against
///     (e.g. permissiona the user has, etc.).
/// - One or more [`Permission`]s to require to be in that set, specified as an array
///   (e.g. `[Permission::WordRead, Permission::WordUpdate]`).
///   If you require only one permission, you need not use the square brackets.
///
///
/// # Example
/// ```no_run
/// use kolomoni::api::openapi;
/// use kolomoni::api::openapi::response::requires;
/// use kolomoni::api::errors::EndpointResult;
/// use kolomoni::authentication::UserAuthenticationExtractor;
/// use kolomoni::authentication::AuthenticatedUser;
/// use kolomoni_auth::Permission;
///
/// #[utoipa::path(
///     get,
///     path = "/",
///     responses(
///         openapi::response::MissingAuthentication,
///         openapi::response::MissingPermissions<requires::UserSelfRead, 1>
///     )
/// )]
/// #[actix_web::get("/")]
/// async fn fetch_something(
///     state: ApplicationState,
///     // The presence of this extractor **DOES NOT** imply that the body
///     // of this function will execute only when the user is authenticated -
///     // we need to use the extractor instance to check for ourselves; it simply
///     // exposes authentication information to us.
///     authentication_extractor: UserAuthenticationExtractor,
/// ) -> EndpointResult {
///     // ...
///
///     let mut database_connection = state.acquire_database_connection().await?;
///     
///     let authenticated_user = require_user_authentication!(authentication_extractor);
///     let user_permissions = authenticated_user
///         .fetch_transitive_permissions(&mut database_connection)
///         .await?;
///     
///     require_permission_in_set!(user_permissions, Permission::UserSelfRead);
///
///     // ...
///     # todo!();
/// }
/// ```
///
///
/// [`ErrorReason::missing_permission(...)`]: crate::api::errors::ErrorReason::missing_permission
/// [`MissingPermissions`]: crate::api::openapi::response::MissingPermissions
/// [`Permission`]: kolomoni_auth::Permission
/// [`PermissionSet`]: kolomoni_auth::PermissionSet
#[macro_export]
macro_rules! require_permission_in_set {
    ($permission_set:expr, $required_permission:expr) => {
        if !$permission_set.has_permission_or_is_blanket_granted($required_permission) {
            return $crate::api::errors::EndpointResponseBuilder::forbidden()
                .with_error_reason(
                    $crate::api::errors::ErrorReason::missing_permission($required_permission),
                )
                .build();
        }
    };

    ($permission_set:expr, [$($required_permission:expr),+]) => {
        if !$permission_set.has_permission_or_is_blanket_granted($required_permission) {
            return $crate::api::errors::EndpointResponseBuilder::new(
                actix_web::http::StatusCode::FORBIDDEN,
            )
            .with_error_reason(
                $crate::api::errors::ErrorReason::missing_permissions_from_slice(
                    &[$($required_permission:expr),+]
                ),
            )
            .build();
        }
    };
}


/// An endpoint handler macro that, given an authenticated user, verifies that that
/// user has the required permission or permissions.
///
/// It expands to a check that does a union between blanket permission grants and the
/// user's transitive permissions, verifying that the user actually has the required permissions.
///
/// The associated documentation type to use on the endpoint
/// that uses this macro is [`MissingPermissions`]
/// (though [`MissingAuthentication`] will likely also be needed, since you'll need to obtain
/// [`AuthenticatedUser`] somehow, likely with the [`require_user_authentication`] macro).
///
/// # Usage
/// The macro expects three comma-separated parameters:
/// - A database connection. This must be of type [`&mut PgConnection`].
///   Note that for e.g. [`PoolConnection<Postgres>`], which is the most common type you'll encounter in endpoint handlers,
///   you can simply [mutably deref it] to [`&mut PgConnection`] with e.g. `&mut pool_connection`.
/// - An [`AuthenticatedUser`] instance (you may use e.g. [`require_user_authentication`] to obtain it, though in that case,
///   it would be cleaner to use the [`require_user_authentication_and_permissions`] macro instead).
/// - One or more [`Permission`]s to require, specified as an array (e.g. `[Permission::WordRead, Permission::WordUpdate]`).
///   If you require only one permission, you need not use the square brackets.
///
///
/// # Example
/// ```no_run
/// use kolomoni::api::openapi;
/// use kolomoni::api::openapi::response::requires;
/// use kolomoni::api::errors::EndpointResult;
/// use kolomoni::authentication::UserAuthenticationExtractor;
/// use kolomoni::authentication::AuthenticatedUser;
/// use kolomoni_auth::Permission;
///
/// #[utoipa::path(
///     get,
///     path = "/",
///     responses(
///         openapi::response::MissingAuthentication,
///         openapi::response::MissingPermissions<requires::WordRead, 1>
///     )
/// )]
/// #[actix_web::get("/")]
/// async fn fetch_something(
///     state: ApplicationState,
///     // The presence of this extractor **DOES NOT** imply that the body
///     // of this function will execute only when the user is authenticated -
///     // we need to use the extractor instance to check for ourselves; it simply
///     // exposes authentication information to us.
///     authentication_extractor: UserAuthenticationExtractor,
/// ) -> EndpointResult {
///     // ...
///
///     let mut database_connection = state.acquire_database_connection().await?;
///     
///
///     let authenticated_user: AuthenticatedUser = require_user_authentication!(
///         authentication_extractor
///     );
///
///     // ...
///     
///     // Note that this is a bad example - if you need to require user authentication
///     // alongside some permission, you should prefer using
///     // `require_user_authentication_and_permissions` instead.
///     require_permissions_on_user!(
///         &mut database_connection,
///         authenticated_user,
///         Permission::WordRead
///     );
///
///     // ...
///     # todo!();
/// }
/// ```
///
///
/// # Side Effects
/// This macro *may* perform a database lookup if the blanket grant
/// does not satisfy the requirement. Execution *may* therefore cross
/// an async yield point (`await`).
///
///
///
/// [`require_user_authentication_and_permissions`]: crate::require_user_authentication_and_permissions
/// [`Permission`]: kolomoni_auth::Permission
/// [`MissingAuthentication`]: crate::api::openapi::response::MissingAuthentication
/// [`AuthenticatedUser`]: crate::authentication::AuthenticatedUser
/// [`MissingPermissions`]: crate::api::openapi::response::MissingPermissions
/// [`&mut PgConnection`]: sqlx::PgConnection
/// [`PoolConnection<Postgres>`]: sqlx::pool::PoolConnection
/// [mutably deref it]: https://docs.rs/sqlx/0.8.2/sqlx/pool/struct.PoolConnection.html#impl-AsMut%3C%3CDB+as+Database%3E::Connection%3E-for-PoolConnection%3CDB%3E
#[macro_export]
macro_rules! require_permissions_on_user {
    ($database_connection:expr, $authenticated_user:expr, $required_permission:expr) => {{
        if !$authenticated_user
            .transitively_has_permission($database_connection, $required_permission)
            .await?
        {
            return $crate::api::errors::EndpointResponseBuilder::forbidden()
                .with_error_reason(
                    $crate::api::errors::ErrorReason::missing_permission($required_permission),
                )
                .build();
        }

        $authenticated_user
    }};

    ($database_connection:expr, $authenticated_user:expr, [$($required_permission:expr),+]) => {{
        let required_permission_set = PermissionSet::from_permissions(
            &[$($required_permission:expr),+]
        );


        if !$authenticated_user
            .transitively_has_permissions($database_connection, required_permission_set)
            .await?
        {
            return $crate::api::errors::EndpointResponseBuilder::new(
                actix_web::http::StatusCode::FORBIDDEN,
            )
            .with_error_reason(
                $crate::api::errors::ErrorReason::missing_permissions(required_permission_set),
            )
            .build();
        }

        $authenticated_user
    }};
}



/// An endpoint handler macro that requires a set of permissions on the caller,
/// while *also* requiring user authentication.
///
/// It expands to the following pseudocode:
/// - If the user is not authenticated, the macro will early-return a `401 Unauthorized`
///   with [`ErrorReason::missing_authentication()`].
/// - Otherwise the union of the user's transitive permissions and the blanket permission grant
///   is compared to the required permission (or permissions).
///     - If the requirement is satisfied (user + blanket grant covers all required permissions),
///       the operation is considered a success, and the macro invocation evaluates to [`AuthenticatedUser`].
///     - If the requirement is not satisfied, the macro will early-return a `403 Forbidden` response
///       with [`ErrorReason::missing_permission(...)`].
///
/// The associated documentation type to use on the endpoint
/// that uses this macro is [`MissingAuthentication`] + [`MissingPermissions`].
///
/// # Usage
/// The macro expects three comma-separated parameters:
/// - A database connection. This must be of type [`&mut PgConnection`].
///   Note that for e.g. [`PoolConnection<Postgres>`], which is the most common type you'll encounter in endpoint handlers,
///   you can simply [mutably deref it] to [`&mut PgConnection`] with e.g. `&mut pool_connection`.
/// - User authentication extractor ([`UserAuthenticationExtractor`]).
/// - One or more [`Permission`]s to require, specified as an array (e.g. `[Permission::WordRead, Permission::WordUpdate]`).
///   If you require only one permission, you need not use the square brackets.
///
///
/// # Example
/// ```no_run
/// use kolomoni::api::openapi;
/// use kolomoni::api::openapi::response::requires;
/// use kolomoni::api::errors::EndpointResult;
/// use kolomoni::authentication::UserAuthenticationExtractor;
/// use kolomoni::authentication::AuthenticatedUser;
/// use kolomoni_auth::Permission;
///
/// #[utoipa::path(
///     get,
///     path = "/",
///     responses(
///         // The `MissingAuthentication` + `MissingPermissions` OpenAPI response types
///         // go hand in hand with this macro.
///         openapi::response::MissingAuthentication,
///         openapi::response::MissingPermissions<requires::WordRead, 1>
///     )
/// )]
/// #[actix_web::get("/")]
/// async fn fetch_something(
///     state: ApplicationState,
///     // The presence of this extractor **DOES NOT** imply that the body
///     // of this function will execute only when the user is authenticated -
///     // we need to use the extractor instance to check for ourselves; it simply
///     // exposes authentication information to us.
///     authentication_extractor: UserAuthenticationExtractor,
/// ) -> EndpointResult {
///     // ...
///
///     let mut database_connection = state.acquire_database_connection().await?;
///     
///     // This macro invocation will expand to a permission check and will early-return
///     // if the requirements are not satisfied. It returns an `AuthenticatedUser`.
///     let authenticated_user: AuthenticatedUser = require_user_authentication_and_permissions!(
///         &mut database_connection,
///         authentication_extractor,
///         Permission::WordRead
///     );
///     
///     println!(
///         "Authenticated user (has WordRead permission): {}",
///         authenticated_user.user_id()
///     );
///
///     // ...
///     # todo!();
/// }
/// ```
///
///
/// # Side Effects
/// This macro *may* perform a database lookup when the blanket grant
/// does not satisfy the requirement. Execution *may* therefore cross
/// an async yield point (`await`).
///
///
/// [`MissingAuthentication`]: crate::api::openapi::response::MissingAuthentication
/// [`ErrorReason::missing_authentication()`]: crate::api::errors::ErrorReason::missing_authentication
/// [`ErrorReason::missing_permission(...)`]: crate::api::errors::ErrorReason::missing_permission
/// [`AuthenticatedUser`]: crate::authentication::AuthenticatedUser
/// [`MissingPermissions`]: crate::api::openapi::response::MissingPermissions
/// [`&mut PgConnection`]: sqlx::PgConnection
/// [`PoolConnection<Postgres>`]: sqlx::pool::PoolConnection
/// [mutably deref it]: https://docs.rs/sqlx/0.8.2/sqlx/pool/struct.PoolConnection.html#impl-AsMut%3C%3CDB+as+Database%3E::Connection%3E-for-PoolConnection%3CDB%3E
/// [`Permission`]: kolomoni_auth::Permission
/// [`UserAuthenticationExtractor`]: crate::authentication::UserAuthenticationExtractor
#[macro_export]
macro_rules! require_user_authentication_and_permissions {
    ($database_connection:expr, $authentication_extractor:expr, $required_permission:expr) => {{
        let __authenticated_user = $crate::require_user_authentication!($authentication_extractor);

        $crate::require_permissions_on_user!(
            $database_connection,
            __authenticated_user,
            $required_permission
        )
    }};

    ($database_connection:expr, $authentication_extractor:expr, [$($required_permission:expr),+]) => {{
        let __authenticated_user = $crate::require_user_authentication!($authentication_extractor);

        $crate::require_permissions_on_user!(
            $database_connection,
            __authenticated_user,
            [$($required_permission:expr),+]
        )
    }};
}
