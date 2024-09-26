/// Usually implemented on a configuration type that can be validated or resolved *infallibly*.
pub trait Resolve {
    type Resolved;

    /// Resolve the configuration into its [`Self::Resolved`] type.
    fn resolve(self) -> Self::Resolved;
}

/// Usually implemented on a configuration type that can be validated or resolved,
/// but where that process can fail (for infallible conversion, see [`Resolve`]).
pub trait TryResolve {
    type Resolved;
    type Error;

    /// Attempt to resolve the configuration into its [`Self::Resolved`] type.
    ///
    /// If the validation / conversion into the resolved type fails,
    /// an err `Err` should be returned to indicate that.
    fn try_resolve(self) -> Result<Self::Resolved, Self::Error>;
}



/// Usually implemented on a configuration type that can be validated or resolved *infallibly*
/// (and where that process requires some additional user-provided context).
pub trait ResolveWithContext<'r> {
    type Context;
    type Resolved;

    /// Resolve the configuration into its [`Self::Resolved`] type
    /// using some `context`.
    fn resolve_with_context(self, context: Self::Context) -> Self::Resolved;
}

/// Usually implemented on a configuration type that can be validated or resolved
/// (and where that process requires some additional user-provided context),
/// but where that process can fail (for infallible conversion, see [`Resolve`]).
pub trait TryResolveWithContext {
    type Context;
    type Resolved;
    type Error;

    /// Attempt to resolve the configuration into its [`Self::Resolved`] type
    /// using some `context`.
    ///
    /// If the validation / conversion into the resolved type fails,
    /// an err `Err` should be returned to indicate that.
    fn try_resolve_with_context(self, context: Self::Context)
        -> Result<Self::Resolved, Self::Error>;
}
