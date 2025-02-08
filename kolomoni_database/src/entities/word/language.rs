/// A word language (Slovene or English).
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WordLanguage {
    Slovene,
    English,
}

impl WordLanguage {
    /// Given a Slovene (`sl`) or English (`en`) IETF BCP 47 language tag
    /// (as defined by [RFC 5646](https://www.rfc-editor.org/rfc/rfc5646.html)),
    /// this function parses the tag and returns a variant of [`WordLanguage`],
    /// if appropriate.
    ///
    /// The returned value will be `None` if the provided string is not valid language
    /// tag or is not a Slovene or English tag.
    ///
    ///
    /// ```
    /// # use crate::entities::WordLanguage;
    ///
    /// assert_eq!(
    ///     WordLanguage::from_ietf_bcp_47_language_tag("en"),
    ///     Some(WordLanguage::English)
    /// );
    ///
    /// assert_eq!(
    ///     WordLanguage::from_ietf_bcp_47_language_tag("sl"),
    ///     Some(WordLanguage::Slovene)
    /// );
    ///
    /// assert_eq!(
    ///     WordLanguage::from_ietf_bcp_47_language_tag("abcd"),
    ///     None
    /// );
    /// ```
    pub fn from_ietf_bcp_47_language_tag(language_tag: &str) -> Option<Self> {
        match language_tag {
            "sl" => Some(Self::Slovene),
            "en" => Some(Self::English),
            _ => None,
        }
    }

    /// Convert the given word language into its corresponding IETF BCP 47 language tag
    /// (as defined by [RFC 5646](https://www.rfc-editor.org/rfc/rfc5646.html)).
    ///
    /// This is `sl` for [`WordLanguage::Slovene`] and `en` for [`WordLanguage::English`].
    pub fn to_ietf_bcp_47_language_tag(self) -> &'static str {
        match self {
            WordLanguage::Slovene => "sl",
            WordLanguage::English => "en",
        }
    }
}
