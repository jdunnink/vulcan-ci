# Vulcan Chain Parser

KDL workflow parser that converts workflow definitions into database-ready chain and fragment records.

## Status

**Complete** - Fully functional with comprehensive test coverage.

## Usage

Add to your `Cargo.toml` (from a service crate):

```toml
[dependencies]
vulcan-chain-parser = { path = "../../libs/chain-parser" }
```

Or use the workspace dependency:

```toml
[dependencies]
vulcan-chain-parser.workspace = true
```

## Features

- Parse KDL workflow files with version, triggers, and chain definitions
- Recursive import resolution from external URLs
- Circular import detection
- Parallel execution blocks
- Conditional fragment execution
- Machine/worker group assignment
- Trigger type validation

## Example

```rust
use uuid::Uuid;
use vulcan_chain_parser::{ChainParserService, WorkflowContext, ImportFetcher, Result};
use vulcan_core::models::chain::TriggerType;

// Implement a fetcher for resolving imports
struct MyFetcher;

impl ImportFetcher for MyFetcher {
    fn fetch(&self, url: &str) -> Result<String> {
        // Fetch content from URL...
        Ok("fragment { run \"echo imported\" }".to_string())
    }
}

// Create the parser service
let service = ChainParserService::new(MyFetcher);

// Define workflow context
let context = WorkflowContext::new(Uuid::new_v4())
    .with_source(".vulcan/ci.kdl".to_string())
    .with_repository("https://github.com/org/repo".to_string())
    .with_trigger(TriggerType::Push, None);

// Parse a workflow
let content = r#"
version "0.1"
triggers "push" "pull_request"

chain {
    machine "default-worker"

    fragment { run "npm install" }

    parallel {
        fragment { run "npm test" }
        fragment { run "npm lint" }
    }
}
"#;

let result = service.parse(content, &context).unwrap();

// result.chain - NewChain ready for database insertion
// result.fragments - Vec<NewFragment> ready for database insertion
```

## Workflow Format

See [PARSER_SPEC.md](PARSER_SPEC.md) for the complete KDL workflow specification.

### Quick Reference

```kdl
version "0.1"
triggers "push" "pull_request" "tag" "schedule" "manual"

chain {
    // Default worker group for all fragments
    machine "worker-group"

    // Simple fragment
    fragment { run "echo hello" }

    // Fragment with options
    fragment {
        run "npm test"
        machine "large-runner"
        condition "$BRANCH == 'main'"
    }

    // Parallel execution
    parallel {
        fragment { run "test:unit" }
        fragment { run "test:integration" }
    }

    // Import from URL
    fragment {
        from "https://example.com/shared/deploy.kdl"
    }
}
```

## Modules

- **ast** - Abstract syntax tree types (`ParsedChain`, `ParsedFragment`)
- **parser** - Low-level KDL parser with `ImportFetcher` trait
- **service** - High-level `ChainParserService` with context handling
- **error** - Error types (`ParseError`, `Result`)

## API

### `ChainParserService`

- `parse(content, context)` - Parse with trigger validation
- `parse_without_trigger_validation(content, context)` - Parse without checking triggers

### `WorkflowContext`

Builder for providing execution context:

```rust
let context = WorkflowContext::new(tenant_id)
    .with_source(file_path)
    .with_repository(repo_url)
    .with_commit(sha)
    .with_branch(branch)
    .with_trigger(trigger_type, trigger_ref);
```

### `ImportFetcher`

Trait for custom import resolution:

```rust
pub trait ImportFetcher {
    fn fetch(&self, url: &str) -> Result<String>;
}
```
