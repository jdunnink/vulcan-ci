//! Tests for the KDL parser.

use std::collections::HashMap;

use crate::ast::ParsedFragmentType;
use crate::error::{ParseError, Result};
use crate::parser::{ChainParser, ImportFetcher};

/// Mock fetcher that returns predefined content for testing.
struct MockFetcher {
    responses: HashMap<String, String>,
}

impl MockFetcher {
    fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    fn with_response(mut self, url: &str, content: &str) -> Self {
        self.responses.insert(url.to_string(), content.to_string());
        self
    }
}

impl ImportFetcher for MockFetcher {
    fn fetch(&self, url: &str) -> Result<String> {
        self.responses.get(url).cloned().ok_or_else(|| ParseError::FetchFailed {
            url: url.to_string(),
            reason: "not found in mock".to_string(),
        })
    }
}

#[test]
fn test_parse_simple_workflow() {
    let content = r#"
version "0.1"
triggers "push"

chain {
    machine "default-worker"

    fragment { run "npm build" }
    fragment { run "npm test" }
}
"#;

    let parser = ChainParser::new(MockFetcher::new());
    let chain = parser.parse_workflow(content, None).unwrap();

    assert_eq!(chain.triggers, vec!["push"]);
    assert_eq!(chain.default_machine, "default-worker");
    assert_eq!(chain.fragments.len(), 2);
    assert_eq!(chain.fragments[0].run_script.as_deref(), Some("npm build"));
    assert_eq!(chain.fragments[1].run_script.as_deref(), Some("npm test"));
}

#[test]
fn test_parse_workflow_with_conditions() {
    let content = r#"
version "0.1"
triggers "push"

chain {
    machine "default-worker"

    fragment {
        run "npm build"
    }
    fragment {
        condition "$BRANCH == 'main'"
        run "npm deploy"
        machine "prod-worker"
    }
}
"#;

    let parser = ChainParser::new(MockFetcher::new());
    let chain = parser.parse_workflow(content, None).unwrap();

    assert_eq!(chain.fragments.len(), 2);
    assert!(chain.fragments[0].condition.is_none());
    assert_eq!(
        chain.fragments[1].condition.as_deref(),
        Some("$BRANCH == 'main'")
    );
    assert_eq!(chain.fragments[1].machine.as_deref(), Some("prod-worker"));
}

#[test]
fn test_parse_parallel_workflow() {
    let content = r#"
version "0.1"
triggers "pull_request"

chain {
    machine "default-worker"

    parallel {
        fragment { run "npm test:unit" }
        fragment { run "npm test:e2e" }
    }
}
"#;

    let parser = ChainParser::new(MockFetcher::new());
    let chain = parser.parse_workflow(content, None).unwrap();

    // Should have: 1 group + 2 children = 3 fragments
    assert_eq!(chain.fragments.len(), 3);

    let group = &chain.fragments[0];
    assert_eq!(group.fragment_type, ParsedFragmentType::Group);
    assert!(group.is_parallel);

    let child1 = &chain.fragments[1];
    let child2 = &chain.fragments[2];
    assert_eq!(child1.parent_id, Some(group.id));
    assert_eq!(child2.parent_id, Some(group.id));
}

#[test]
fn test_parse_with_import() {
    let build_kdl = r#"
fragment { run "npm install" }
fragment { run "npm build" }
"#;

    let workflow = r#"
version "0.1"
triggers "push"

chain {
    machine "default-worker"

    fragment { from "https://example.com/build.kdl" }
    fragment { run "npm deploy" }
}
"#;

    let fetcher = MockFetcher::new().with_response("https://example.com/build.kdl", build_kdl);

    let parser = ChainParser::new(fetcher);
    let chain = parser.parse_workflow(workflow, None).unwrap();

    // Should have: 2 imported + 1 inline = 3 fragments
    assert_eq!(chain.fragments.len(), 3);
    assert_eq!(chain.fragments[0].run_script.as_deref(), Some("npm install"));
    assert_eq!(
        chain.fragments[0].source_url.as_deref(),
        Some("https://example.com/build.kdl")
    );
    assert_eq!(chain.fragments[1].run_script.as_deref(), Some("npm build"));
    assert_eq!(chain.fragments[2].run_script.as_deref(), Some("npm deploy"));
    assert!(chain.fragments[2].source_url.is_none());
}

#[test]
fn test_circular_import_detection() {
    let file_a = r#"
fragment { from "https://example.com/b.kdl" }
"#;

    let file_b = r#"
fragment { from "https://example.com/a.kdl" }
"#;

    let workflow = r#"
version "0.1"
triggers "push"

chain {
    machine "default-worker"

    fragment { from "https://example.com/a.kdl" }
}
"#;

    let fetcher = MockFetcher::new()
        .with_response("https://example.com/a.kdl", file_a)
        .with_response("https://example.com/b.kdl", file_b);

    let parser = ChainParser::new(fetcher);
    let result = parser.parse_workflow(workflow, None);

    assert!(matches!(result, Err(ParseError::CircularImport(_))));
}

#[test]
fn test_mutual_exclusion_error() {
    let content = r#"
version "0.1"
triggers "push"

chain {
    machine "default-worker"

    fragment {
        run "npm build"
        from "https://example.com/build.kdl"
    }
}
"#;

    let parser = ChainParser::new(MockFetcher::new());
    let result = parser.parse_workflow(content, None);

    assert!(matches!(result, Err(ParseError::MutualExclusion)));
}

#[test]
fn test_missing_machine_error() {
    let content = r#"
version "0.1"
triggers "push"

chain {
    fragment { run "npm build" }
}
"#;

    let parser = ChainParser::new(MockFetcher::new());
    let result = parser.parse_workflow(content, None);

    assert!(matches!(
        result,
        Err(ParseError::MissingRequired { field: "machine", .. })
    ));
}

#[test]
fn test_multiple_triggers() {
    let content = r#"
version "0.1"
triggers "push" "pull_request" "tag"

chain {
    machine "default-worker"

    fragment { run "npm build" }
}
"#;

    let parser = ChainParser::new(MockFetcher::new());
    let chain = parser.parse_workflow(content, None).unwrap();

    assert_eq!(chain.triggers, vec!["push", "pull_request", "tag"]);
}
