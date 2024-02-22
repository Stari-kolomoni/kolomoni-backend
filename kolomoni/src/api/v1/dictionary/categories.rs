use actix_web::{delete, get, patch, post, web, Scope};

use crate::api::errors::EndpointResult;




#[post("")]
pub async fn create_category() -> EndpointResult {
    todo!();
}




#[get("")]
pub async fn get_all_categories() -> EndpointResult {
    todo!();
}




#[get("/{category_id}")]
pub async fn get_specific_category() -> EndpointResult {
    todo!();
}




#[patch("/{category_id}")]
pub async fn update_specific_category() -> EndpointResult {
    todo!();
}




#[delete("/{category_id}")]
pub async fn delete_specific_category() -> EndpointResult {
    todo!();
}




#[post("/{category_id}/word-link/{word_uuid}")]
pub async fn link_word_to_category() -> EndpointResult {
    todo!();
}




#[delete("/{category_id}/word-link/{word_uuid}")]
pub async fn unlink_word_from_category() -> EndpointResult {
    todo!();
}




#[rustfmt::skip]
pub fn categories_router() -> Scope {
    web::scope("/category")
        .service(create_category)
        .service(get_all_categories)
        .service(get_specific_category)
        .service(update_specific_category)
        .service(delete_specific_category)
        .service(link_word_to_category)
        .service(unlink_word_from_category)
}
