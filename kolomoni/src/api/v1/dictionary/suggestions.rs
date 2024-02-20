use actix_web::{post, web, Scope};
use kolomoni_auth::Permission;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    api::errors::EndpointResult,
    authentication::UserAuthenticationExtractor,
    require_authentication,
    require_permission,
    state::ApplicationState,
};


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, ToSchema)]
pub struct TranslationSuggestionRequest {
    english_word_id: String,
    slovene_word_id: String,
}

#[post("")]
pub async fn suggest_a_translation(
    state: ApplicationState,
    authentication: UserAuthenticationExtractor,
    request_body: web::Json<TranslationSuggestionRequest>,
) -> EndpointResult {
    let authenticated_user = require_authentication!(authentication);
    require_permission!(
        state,
        authenticated_user,
        Permission::SuggestionCreate
    );

    // TODO Continue from here.


    todo!();
}


#[rustfmt::skip]
pub fn suggested_translations_router() -> Scope {
    web::scope("/suggestion")
        .service(suggest_a_translation)
}
