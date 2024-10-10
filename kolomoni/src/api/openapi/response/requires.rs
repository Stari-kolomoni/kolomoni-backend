//! Contains required-permission types used in the [`MissingPermissions`] annotations,
//! and the [`And`] operator to combine them.
//!
//! [`MissingPermissions`]: super::MissingPermissions


use std::marker::PhantomData;

use kolomoni_auth::Permission;

/// An [`openapi`] module-internal trait to aid in
/// declaring a required permission for an endpoint's
/// OpenAPI documentation.
///
/// [`openapi`]: crate::api::openapi
pub(super) trait RequiredPermission {
    fn permission() -> Permission;
}

/// An [`openapi`] module-internal trait to aid in
/// declaring a set of required permissions for an endpoint's
/// OpenAPI documentation.
///
/// [`openapi`]: crate::api::openapi
pub(super) trait RequiredPermissionSet<const N: usize> {
    fn permissions() -> [Permission; N];
}


/// Indicates that the caller of the endpoint must have both permissions
/// (as specified by the generics `L` and `R`).
pub struct And<L, R> {
    _marker_l: PhantomData<L>,
    _marker_r: PhantomData<R>,
}


impl<L, R> RequiredPermissionSet<2> for And<L, R>
where
    L: RequiredPermission,
    R: RequiredPermission,
{
    fn permissions() -> [Permission; 2] {
        [L::permission(), R::permission()]
    }
}

impl<L, M, R> RequiredPermissionSet<3> for And<L, And<M, R>>
where
    L: RequiredPermission,
    M: RequiredPermission,
    R: RequiredPermission,
{
    fn permissions() -> [Permission; 3] {
        [L::permission(), M::permission(), R::permission()]
    }
}

impl<L, M, R> RequiredPermissionSet<3> for And<And<L, M>, R>
where
    L: RequiredPermission,
    M: RequiredPermission,
    R: RequiredPermission,
{
    fn permissions() -> [Permission; 3] {
        [L::permission(), M::permission(), R::permission()]
    }
}




/// Given a variant name for [`Permission`], this macro will generate
/// an empty struct with the name `RequiredPermissionNameHere`.
///
/// For example, calling `generate_standalone_requirement_struct!(UserSelfRead)`
/// will result in the following code:
///
/// ```no_run
/// # use kolomoni::api::openapi::RequiredPermission;
/// # use kolomoni_auth::Permission;
/// pub struct RequiresUserSelfRead;
/// impl RequiredPermission for RequiresUserSelfRead {
///     fn name() -> &'static str {
///         Permission::UserSelfRead.name()
///     }
/// }
/// ```
macro_rules! generate_standalone_requirement_struct {
    ($permission_variant:ident) => {
        ::paste::paste! {
            // FIXME this first doc isn't emitted for some reason? not a problem, but it's a bit annoying
            #[doc = concat!(
                "Corresponds to the [`Permission::",
                stringify!($permission_variant),
                "`][kolomoni_auth::Permission::",
                stringify!($permission_variant),
                "] permission.")
            ]
            #[doc =
                "Use in conjunction with [`MissingPermissions`][crate::api::openapi::response::MissingPermissions] \
                to indicate that the permission is required. See its documentation for more information on usage."
            ]
            pub struct $permission_variant;

            impl RequiredPermission for $permission_variant {
                fn permission() -> kolomoni_auth::Permission {
                    kolomoni_auth::Permission::$permission_variant
                }
            }

            impl RequiredPermissionSet<1> for $permission_variant {
                fn permissions() -> [kolomoni_auth::Permission; 1] {
                    [Self::permission()]
                }
            }
        }
    };
}


// These macro calls generate empty structs for all available permissions,
// making them usable as a parameter for the [`FailedAuthenticationResponses`] generic.

generate_standalone_requirement_struct!(UserSelfRead);
generate_standalone_requirement_struct!(UserSelfWrite);
generate_standalone_requirement_struct!(UserAnyRead);
generate_standalone_requirement_struct!(UserAnyWrite);
generate_standalone_requirement_struct!(WordCreate);
generate_standalone_requirement_struct!(WordRead);
generate_standalone_requirement_struct!(WordUpdate);
generate_standalone_requirement_struct!(WordDelete);
generate_standalone_requirement_struct!(SuggestionCreate);
generate_standalone_requirement_struct!(SuggestionDelete);
generate_standalone_requirement_struct!(TranslationCreate);
generate_standalone_requirement_struct!(TranslationDelete);
generate_standalone_requirement_struct!(CategoryCreate);
generate_standalone_requirement_struct!(CategoryRead);
generate_standalone_requirement_struct!(CategoryUpdate);
generate_standalone_requirement_struct!(CategoryDelete);


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generates_correct_impls() {
        // Tests only a subset of the generated structs, because, generally speaking,
        // if one is correct, the rest should be as well.

        assert_eq!(
            UserSelfRead::permission(),
            Permission::UserSelfRead
        );

        assert_eq!(
            UserSelfRead::permissions(),
            [Permission::UserAnyRead]
        );

        assert_eq!(
            And::<UserSelfRead, UserSelfWrite>::permissions(),
            [Permission::UserSelfRead, Permission::UserSelfWrite]
        );

        assert_eq!(
            And::<And<UserSelfRead, UserAnyWrite>, CategoryCreate>::permissions(),
            [
                Permission::UserSelfRead,
                Permission::UserSelfWrite,
                Permission::CategoryCreate
            ]
        );
    }
}
