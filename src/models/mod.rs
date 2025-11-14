//! Data models for the blog template.

pub mod additional_resources;
pub mod data_access;
pub mod data_preparation;
pub mod discussion;
pub mod overview;
pub mod post;
pub mod statistical_analysis;

pub mod conversion;
pub mod json_dto;

pub use additional_resources::*;
pub use data_access::*;
pub use data_preparation::DataPrepModel;
pub use discussion::*;
pub use overview::*;
pub use post::*;
pub use statistical_analysis::StatsModel;

/// Helper macro for creating Cow::Borrowed string literals in constants
#[macro_export]
macro_rules! cow {
    ($s:expr) => {
        ::std::borrow::Cow::Borrowed($s)
    };
}

/// Helper function to convert static string slices to Vec<Cow<'static, str>>
pub fn cows(xs: &[&'static str]) -> Vec<std::borrow::Cow<'static, str>> {
    xs.iter().map(|&s| std::borrow::Cow::Borrowed(s)).collect()
}
