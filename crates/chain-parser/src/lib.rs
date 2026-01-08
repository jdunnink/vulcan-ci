//! Vulcan Chain Parser.
//!
//! This crate provides functionality to parse KDL workflow definitions
//! and convert them into database-ready chain and fragment records.
//!
//! # Overview
//!
//! The chain parser:
//! - Parses KDL workflow files with version, triggers, and chain definitions
//! - Recursively resolves import fragments from external URLs
//! - Detects circular imports
//! - Validates workflow structure and required fields
//! - Converts parsed AST to database models ready for insertion
//!
//! # Example
//!
//! ```no_run
//! use uuid::Uuid;
//! use vulcan_chain_parser::{ChainParserService, WorkflowContext, ImportFetcher};
//! use vulcan_chain_parser::error::Result;
//! use vulcan_core::models::chain::TriggerType;
//!
//! // Implement your own fetcher for resolving imports
//! struct HttpFetcher;
//!
//! impl ImportFetcher for HttpFetcher {
//!     fn fetch(&self, url: &str) -> Result<String> {
//!         // Fetch URL content...
//!         todo!()
//!     }
//! }
//!
//! // Create the service
//! let service = ChainParserService::new(HttpFetcher);
//!
//! // Define the context
//! let context = WorkflowContext::new(Uuid::new_v4())
//!     .with_source(".vulcan/ci.kdl".to_string())
//!     .with_repository("https://github.com/org/repo".to_string())
//!     .with_trigger(TriggerType::Push, None);
//!
//! // Parse a workflow
//! let content = r#"
//! version "0.1"
//! triggers "push"
//!
//! chain {
//!     machine "default-worker"
//!     fragment { run "npm build" }
//! }
//! "#;
//!
//! // let result = service.parse(content, &context).unwrap();
//! // result.chain and result.fragments are ready for database insertion
//! ```

/// Abstract syntax tree types for parsed workflows.
pub mod ast;
/// Error types for parsing operations.
pub mod error;
/// KDL parser implementation.
pub mod parser;
/// High-level parsing service.
pub mod service;

#[cfg(test)]
mod parser_tests;
#[cfg(test)]
mod service_tests;

// Re-export main types for convenience
pub use error::{ParseError, Result};
pub use parser::{ChainParser, ImportFetcher};
pub use service::{ChainParserService, ParsedWorkflow, WorkflowContext};
