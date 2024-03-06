use miette::{miette, Context, IntoDiagnostic, Result};

use crate::{entities, shared::WordLanguage};

impl entities::word::Model {
    pub fn language(&self) -> Result<WordLanguage> {
        WordLanguage::from_ietf_language_tag(&self.language)
            .into_diagnostic()
            .wrap_err_with(|| {
                miette!(
                    "Failed to convert IETF language tag to WordLanguage: {}",
                    self.language
                )
            })
    }
}
