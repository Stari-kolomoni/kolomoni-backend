mod endpoints;
use actix_web::web;
pub use endpoints::*;
mod model_impls;


#[rustfmt::skip]
pub fn categories_router() -> actix_web::Scope {
    web::scope("/category")
        .service(create_category)
        .service(get_all_categories)
        .service(get_specific_category)
        .service(update_specific_category)
        .service(delete_specific_category)
        // TODO
        // .service(link_word_to_category)
        // .service(unlink_word_from_category)
}
