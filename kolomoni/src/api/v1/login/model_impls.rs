use kolomoni_core::api_models::{UserLoginRefreshResponse, UserLoginResponse};

use crate::impl_json_response_builder;


impl_json_response_builder!(UserLoginResponse);
impl_json_response_builder!(UserLoginRefreshResponse);
