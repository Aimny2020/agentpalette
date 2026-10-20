use crate::domain::error::{DomainError, DomainResult};
use crate::domain::skill::SkillMetadata;
use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use yaml_front_matter::YamlFrontMatter;

#[derive(Deserialize)]
struct FrontMatterRaw {
    name: String,
    description: String,
    author: Option<String>,
    version: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

fn scalar_to_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

pub fn parse_skill_markdown(content: &str) -> DomainResult<(SkillMetadata, String)> {
    let document = YamlFrontMatter::parse::<FrontMatterRaw>(content)
        .map_err(|e| DomainError::Database(format!("Failed to parse Frontmatter: {}", e)))?;

    let raw_meta = document.metadata;
    let nested_version = raw_meta
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get("version"))
        .and_then(scalar_to_string);
    let metadata = SkillMetadata {
        name: raw_meta.name,
        description: raw_meta.description,
        author: raw_meta.author,
        version: raw_meta.version.or(nested_version),
    };

    let markdown_body = document.content;
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&markdown_body, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Ok((metadata, html_output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skill_markdown_valid() {
        let content = r#"---
name: Test Skill
description: A skill to test parsing
author: Test Author
version: 1.2.3
---
# Hello World
This is a *test*."#;

        let (meta, html) = parse_skill_markdown(content).unwrap();
        assert_eq!(meta.name, "Test Skill");
        assert_eq!(meta.description, "A skill to test parsing");
        assert_eq!(meta.author.as_deref(), Some("Test Author"));
        assert_eq!(meta.version.as_deref(), Some("1.2.3"));
        assert!(html.contains("<h1>Hello World</h1>"));
        assert!(html.contains("<p>This is a <em>test</em>.</p>"));
    }

    #[test]
    fn parses_version_from_nested_metadata() {
        let content = r#"---
name: Oil UI
description: UI design skill
metadata:
  version: "0.23.8"
  compatibility: "core"
  tags: "design, ui"
---
# Oil UI"#;

        let (meta, _) = parse_skill_markdown(content).unwrap();
        assert_eq!(meta.version.as_deref(), Some("0.23.8"));
    }

    #[test]
    fn top_level_version_takes_precedence_over_nested_metadata_version() {
        let content = r#"---
name: Superpowers
description: Pack
version: 6.2.0
metadata:
  version: "9.9.9"
---
# Body"#;

        let (meta, _) = parse_skill_markdown(content).unwrap();
        assert_eq!(meta.version.as_deref(), Some("6.2.0"));
    }

    #[test]
    fn accepts_numeric_and_scalar_metadata_blocks() {
        let numeric = r#"---
name: Numeric
description: Numeric version
metadata:
  version: 2
---
# Body"#;
        let (meta, _) = parse_skill_markdown(numeric).unwrap();
        assert_eq!(meta.version.as_deref(), Some("2"));

        let scalar = r#"---
name: Scalar
description: Scalar metadata
metadata: not-a-map
---
# Body"#;
        let (meta, _) = parse_skill_markdown(scalar).unwrap();
        assert_eq!(meta.version, None);
    }
}
