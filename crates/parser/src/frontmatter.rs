use crate::types::Frontmatter;

/// Extract YAML frontmatter from markdown content
///
/// Frontmatter format:
/// ```markdown
/// ---
/// title: "Post Title"
/// ---
///
/// # Content starts here
/// ```
pub fn extract_frontmatter(content: &str) -> (Option<Frontmatter>, &str) {
    // Check if content starts with "---"
    if !content.starts_with("---") {
        return (None, content);
    }

    // Find the closing "---"
    let rest = &content[3..];
    if let Some(end_pos) = rest.find("\n---\n") {
        let yaml_content = &rest[..end_pos];
        let remaining = &rest[end_pos + 5..]; // Skip past "\n---\n"

        // Parse YAML
        match parse_yaml_frontmatter(yaml_content) {
            Ok(frontmatter) => (Some(frontmatter), remaining),
            Err(_) => {
                // If YAML parsing fails, treat as regular content
                (None, content)
            }
        }
    } else {
        // No closing delimiter found
        (None, content)
    }
}

fn parse_yaml_frontmatter(yaml: &str) -> Result<Frontmatter, serde_yaml::Error> {
    #[derive(serde::Deserialize)]
    struct YamlFrontmatter {
        title: Option<String>,
        slug: Option<String>,
        description: Option<String>,
        author: Option<String>,
        date_iso: Option<String>,
        tags: Option<Vec<String>>,
        family: Option<String>,
        family_label: Option<String>,
        engine: Option<String>,
        covariates: Option<String>,
        outcome_type: Option<String>,
    }

    let parsed: YamlFrontmatter = serde_yaml::from_str(yaml)?;
    Ok(Frontmatter {
        title: parsed.title,
        slug: parsed.slug,
        description: parsed.description,
        author: parsed.author,
        updated_at: parsed.date_iso,
        tags: parsed.tags,
        family: parsed.family,
        family_label: parsed.family_label,
        engine: parsed.engine,
        covariates: parsed.covariates,
        outcome_type: parsed.outcome_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_frontmatter() {
        let markdown = "# Heading\n\nContent";
        let (fm, content) = extract_frontmatter(markdown);
        assert!(fm.is_none());
        assert_eq!(content, markdown);
    }

    #[test]
    fn test_with_frontmatter() {
        let markdown = "---\ntitle: \"Test Post\"\n---\n\n# Heading";
        let (fm, content) = extract_frontmatter(markdown);
        assert!(fm.is_some());
        assert_eq!(fm.unwrap().title, Some("Test Post".to_string()));
        assert_eq!(content, "\n# Heading");
    }
}
