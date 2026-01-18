//! Tests for the chain parser service.

use uuid::Uuid;
use vulcan_core::models::chain::TriggerType;

use crate::error::{ParseError, Result};
use crate::parser::ImportFetcher;
use crate::service::{ChainParserService, WorkflowContext};

/// Mock fetcher that always fails (for testing workflows without imports).
struct MockFetcher;

impl ImportFetcher for MockFetcher {
    fn fetch(&self, url: &str) -> Result<String> {
        Err(ParseError::FetchFailed {
            url: url.to_string(),
            reason: "mock fetcher".to_string(),
        })
    }
}

#[test]
fn test_parse_and_convert_simple_workflow() {
    let content = r#"
version "0.1"
triggers "push"

chain {
    machine "default-worker"

    fragment { run "npm build" }
    fragment { run "npm test" }
}
"#;

    let service = ChainParserService::new(MockFetcher);
    let context = WorkflowContext::new(Uuid::new_v4())
        .with_source(".vulcan/ci.kdl".to_string())
        .with_repository("https://github.com/org/repo".to_string())
        .with_commit("abc123".to_string())
        .with_branch("main".to_string())
        .with_trigger(TriggerType::Push, None);

    let result = service.parse(content, &context).unwrap();

    assert_eq!(result.chain.tenant_id, context.tenant_id);
    assert_eq!(result.chain.default_machine.as_deref(), Some("default-worker"));
    assert_eq!(result.chain.source_file_path.as_deref(), Some(".vulcan/ci.kdl"));
    assert_eq!(result.chain.trigger, Some(TriggerType::Push));

    assert_eq!(result.fragments.len(), 2);
    assert_eq!(result.fragments[0].sequence, 0);
    assert_eq!(result.fragments[1].sequence, 1);
}

#[test]
fn test_trigger_mismatch_error() {
    let content = r#"
version "0.1"
triggers "push"

chain {
    machine "default-worker"

    fragment { run "npm build" }
}
"#;

    let service = ChainParserService::new(MockFetcher);
    let context = WorkflowContext::new(Uuid::new_v4())
        .with_trigger(TriggerType::PullRequest, Some("123".to_string()));

    let result = service.parse(content, &context);

    assert!(matches!(result, Err(ParseError::InvalidTrigger(_))));
}

#[test]
fn test_parse_without_trigger_validation() {
    let content = r#"
version "0.1"
triggers "push"

chain {
    machine "default-worker"

    fragment { run "npm build" }
}
"#;

    let service = ChainParserService::new(MockFetcher);
    let context = WorkflowContext::new(Uuid::new_v4())
        .with_trigger(TriggerType::PullRequest, Some("123".to_string()));

    // This should succeed even though triggers don't match
    let result = service.parse_without_trigger_validation(content, &context);

    assert!(result.is_ok());
}
