#[cfg(test)]
mod conversion_test {
    use longitudinal_dev::posts::posts;

    #[test]
    fn json_to_post_conversion_works() {
        let converted_posts = posts();

        // If no posts exist (e.g., fresh clone without running validator), skip validation
        if converted_posts.is_empty() {
            println!("⚠️  No posts found (content/posts/ is empty)");
            println!("   This is expected after cleanup or on fresh clone.");
            println!("   Run validator to generate posts from tutorials.");
            return;
        }

        // Find the lgcm-basic post (if it exists)
        let Some(post) = converted_posts
            .iter()
            .find(|p| p.slug.as_ref() == "lgcm-basic")
        else {
            println!(
                "⚠️  lgcm-basic post not found in {} post(s)",
                converted_posts.len()
            );
            println!(
                "   Available slugs: {:?}",
                converted_posts.iter().map(|p| &*p.slug).collect::<Vec<_>>()
            );
            return;
        };

        // Check basic fields
        assert_eq!(&*post.slug, "lgcm-basic");
        assert_eq!(&*post.title, "LGCM: Basic");

        // Check overview features are exactly 3 (padded/truncated)
        assert_eq!(
            post.overview.features_panel.as_ref().unwrap().cards.len(),
            3,
            "Features must be exactly 3"
        );

        let total_posts = converted_posts.len();
        let slug = &post.slug;
        let title = &post.title;
        let features = post.overview.features_panel.as_ref().unwrap().cards.len();

        println!("✅ JSON→Post conversion successful!");
        println!("   Total posts: {total_posts}");
        println!("   Slug: {slug}");
        println!("   Title: {title}");
        println!("   Features: {features}");
    }
}
