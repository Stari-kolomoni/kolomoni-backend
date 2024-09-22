mod category;
mod edit;
mod permission;
mod role;
mod user;
mod user_role;
mod word;
mod word_english;
mod word_english_meaning;
mod word_meaning;
mod word_meaning_translation;
mod word_slovene;
mod word_slovene_meaning;

// TODO refactor query, model and mutation names to no need renaming when re-exported

pub use category::*;
pub use edit::*;
pub use permission::*;
pub use role::*;
pub use user::*;
pub use user_role::*;
pub use word::*;
pub use word_english::*;
pub use word_english_meaning::*;
pub use word_meaning::*;
pub use word_meaning_translation::*;
pub use word_slovene::*;
pub use word_slovene_meaning::*;
