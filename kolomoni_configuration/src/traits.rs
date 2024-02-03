use miette::Result;

pub trait ResolvableConfiguration {
    type Resolved;

    fn resolve(self) -> Result<Self::Resolved>;
}


pub trait ResolvableConfigurationWithContext {
    type Context;
    type Resolved;

    fn resolve(self, context: Self::Context) -> Result<Self::Resolved>;
}
