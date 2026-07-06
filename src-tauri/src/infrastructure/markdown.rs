use serde::Deserialize;
use yaml_front_matter::YamlFrontMatter;
use pulldown_cmark::{Parser, Options, html};
use crate::domain::error::{DomainError, DomainResult};
use crate::domain::skill::SkillMetadata;

#[derive(Deserialize)]
struct FrontMatterRaw {
    name: String,
    description: String,
    author: Option<String>,
    version: Option<String>,
}

pub fn parse_skill_markdown(content: &str) -> DomainResult<(SkillMetadata, String)> {
    let document = YamlFrontMatter::parse::<FrontMatterRaw>(content)
        .map_err(|e| DomainError::Database(format!("Failed to parse Frontmatter: {}", e)))?;
    
    let raw_meta = document.metadata;
    let metadata = SkillMetadata {
        name: raw_meta.name,
        description: raw_meta.description,
        author: raw_meta.author,
        version: raw_meta.version,
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
}
