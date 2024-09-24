pub trait IntoApiModel<ApiModel> {
    fn into_api_model(self) -> ApiModel;
}

pub trait TryIntoApiModel<ApiModel> {
    type Error;

    fn try_into_api_model(self) -> Result<ApiModel, Self::Error>;
}
