use std::borrow::Cow;

use crate::models::json_dto::JsonPost;
use crate::models::post::Post;

impl JsonPost {
    /// Convert JSON DTO to rich Post model
    pub fn into_post(self, slug: &str) -> Post {
        Post {
            slug: Cow::Owned(slug.to_string()),
            title: Cow::Owned(self.title),
            layout: self.layout.map(Cow::Owned),
            metadata: self.metadata,
            overview: self.overview.into_overview_model(),
            data_access: self.data_access.into_data_access_model(),
            data_prep: self.data_preparation,
            statistical_analysis: self.statistical_analysis,
            discussion: self.discussion.into_discussion_model(),
            additional_resources: self.additional_resources.into_resources_model(),
        }
    }
}
