pub trait IntoApiModel {
    type ApiModel;

    fn into_api_model(self) -> Self::ApiModel;
}

pub trait TryIntoApiModel {
    type Error;
    type ApiModel;

    fn try_into_api_model(self) -> Result<Self::ApiModel, Self::Error>;
}
