/// Post registry using JSON→DTO→Post conversion pipeline.
///
/// All posts are authored as JSON files in `content/posts/*.post.json`,
/// validated against schemas at build time, and converted to typed Post models.
use crate::models::post::*;

// ============================================================================
// POST PROVIDER (JSON-based only)
// ============================================================================

mod generated {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/generated/manifest.rs"
    ));
}

/// Load all posts from JSON files via the generated manifest.
///
/// This function:
/// 1. Loads raw JSON DTOs from `content/posts/*.post.json`
/// 2. Converts each DTO to a typed `Post` model
/// 3. Returns posts in alphabetical order by slug
pub fn posts() -> Vec<Post> {
    generated::load_posts_raw()
        .into_iter()
        .enumerate()
        .map(|(i, jp)| jp.into_post(generated::ALL_SLUGS[i]))
        .collect()
}
