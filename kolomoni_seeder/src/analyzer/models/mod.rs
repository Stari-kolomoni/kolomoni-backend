pub mod category;
pub mod english_word;
pub mod english_word_meaning;
pub mod slovene_word;
pub mod slovene_word_meaning;
pub mod translation;


pub trait ToOutputModel {
    type OutputModel;

    fn to_output_model(&self) -> Self::OutputModel;
}


pub trait TryToOutputModelWithContext {
    type Context<'c>;
    type OutputModel;
    type Error;

    fn try_to_output_model<'a>(
        &self,
        context: &'a Self::Context<'a>,
    ) -> Result<Self::OutputModel, Self::Error>;
}



macro_rules! create_internal_id_type {
    ($struct_name:ident) => {
        #[doc = "`kolomoni_seeder`-internal ID new-type. "]
        #[doc = "*Not related to e.g. category UUIDs that are present in the API.*"]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $struct_name {
            pub(crate) id: u64,
        }

        impl $struct_name {
            #[inline]
            #[allow(dead_code)]
            pub(crate) fn new(id: u64) -> Self {
                Self { id }
            }

            #[inline]
            #[allow(dead_code)]
            pub fn generate() -> Self {
                Self {
                    id: fastrand::u64(..),
                }
            }

            #[inline]
            #[allow(dead_code)]
            pub(crate) fn into_inner(self) -> u64 {
                self.id
            }
        }

        impl std::fmt::Display for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.id.fmt(f)
            }
        }
    };
}


create_internal_id_type!(InternalCategoryId);
create_internal_id_type!(InternalEnglishWordId);
create_internal_id_type!(InternalEnglishWordMeaningId);
create_internal_id_type!(InternalSloveneWordId);
create_internal_id_type!(InternalSloveneWordMeaningId);
create_internal_id_type!(InternalTranslationId);
