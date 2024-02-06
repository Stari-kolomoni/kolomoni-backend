use miette::Result;

/// Represents a configuration that can be validated or resolved.
pub trait ResolvableConfiguration {
    type Resolved;

    /// Resolve the configuration into its `Resolved` type.
    /// If the resolution / validation fails, you may return `Err` to indicate
    /// that the configuration is invalid.
    fn resolve(self) -> Result<Self::Resolved>;
}


/// Represents a configuration that can be validated or resolved,
/// but where that process requires some additional context.
pub trait ResolvableConfigurationWithContext {
    type Context;
    type Resolved;

    /// Resolve the configuration into its `Resolved` type.
    /// If the resolution / validation fails, you may return `Err` to indicate
    /// that the configuration is invalid.
    fn resolve(self, context: Self::Context) -> Result<Self::Resolved>;
}
