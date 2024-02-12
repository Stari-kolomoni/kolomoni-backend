#[macro_export]
macro_rules! begin_transaction {
    ($database:expr) => {{
        let _transaction_result = sea_orm::TransactionTrait::begin($database).await;

        let _transaction_diagnostic = miette::IntoDiagnostic::into_diagnostic(_transaction_result);

        miette::Context::wrap_err(
            _transaction_diagnostic,
            "Failed to begin database transaction.",
        )
    }};
}
