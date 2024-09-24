mod endpoints;
mod model_impls;

use actix_web::{web, Scope};
use all::get_all_registered_users;
use current::{
    get_current_user_effective_permissions,
    get_current_user_info,
    get_current_user_roles,
    update_current_user_display_name,
};
pub use endpoints::*;
use registration::register_user;
use specific::{
    add_roles_to_specific_user,
    get_specific_user_effective_permissions,
    get_specific_user_info,
    get_specific_user_roles,
    remove_roles_from_specific_user,
    update_specific_user_display_name,
};



/// Router for all user-related operations.
/// Lives under `/api/v1/users`.
#[rustfmt::skip]
pub fn users_router() -> Scope {
    web::scope("users")
        // all.rs
        .service(get_all_registered_users)
        // registration.rs
        .service(register_user)
        // current.ts
        .service(get_current_user_info)
        .service(get_current_user_roles)
        .service(get_current_user_effective_permissions)
        .service(update_current_user_display_name)
        // specific.rs
        .service(get_specific_user_info)
        .service(get_specific_user_roles)
        .service(get_specific_user_effective_permissions)
        .service(add_roles_to_specific_user)
        .service(remove_roles_from_specific_user)
        .service(update_specific_user_display_name)
}
