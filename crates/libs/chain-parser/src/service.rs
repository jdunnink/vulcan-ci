//! Chain Parser Service.
//!
//! This module provides the main service for parsing workflow files
//! and storing them in the database.

use uuid::Uuid;
use vulcan_core::models::chain::{NewChain, TriggerType};
use vulcan_core::models::fragment::{FragmentType, NewFragment};

use crate::ast::{ParsedChain, ParsedFragment, ParsedFragmentType};
use crate::error::{ParseError, Result};
use crate::parser::{ChainParser, ImportFetcher};

/// Input context for parsing a workflow.
#[derive(Debug, Clone)]
pub struct WorkflowContext {
    /// Tenant ID for the chain.
    pub tenant_id: Uuid,
    /// Path to the workflow file.
    pub source_file_path: Option<String>,
    /// Repository URL.
    pub repository_url: Option<String>,
    /// Git commit SHA.
    pub commit_sha: Option<String>,
    /// Git branch name.
    pub branch: Option<String>,
    /// Trigger type (used to match against workflow triggers).
    pub trigger: Option<TriggerType>,
    /// Trigger reference (e.g., tag name, PR number).
    pub trigger_ref: Option<String>,
}

impl WorkflowContext {
    /// Create a new workflow context with just the tenant ID.
    #[must_use]
    pub const fn new(tenant_id: Uuid) -> Self {
        Self {
            tenant_id,
            source_file_path: None,
            repository_url: None,
            commit_sha: None,
            branch: None,
            trigger: None,
            trigger_ref: None,
        }
    }

    /// Set the source file path.
    #[must_use]
    pub fn with_source(mut self, path: String) -> Self {
        self.source_file_path = Some(path);
        self
    }

    /// Set the repository URL.
    #[must_use]
    pub fn with_repository(mut self, url: String) -> Self {
        self.repository_url = Some(url);
        self
    }

    /// Set the commit SHA.
    #[must_use]
    pub fn with_commit(mut self, sha: String) -> Self {
        self.commit_sha = Some(sha);
        self
    }

    /// Set the branch name.
    #[must_use]
    pub fn with_branch(mut self, branch: String) -> Self {
        self.branch = Some(branch);
        self
    }

    /// Set the trigger information.
    #[must_use]
    pub fn with_trigger(mut self, trigger_type: TriggerType, trigger_ref: Option<String>) -> Self {
        self.trigger = Some(trigger_type);
        self.trigger_ref = trigger_ref;
        self
    }
}

/// Result of parsing a workflow, ready for database insertion.
#[derive(Debug)]
pub struct ParsedWorkflow {
    /// The chain to insert.
    pub chain: NewChain,
    /// The fragments to insert.
    pub fragments: Vec<NewFragment>,
}

/// Chain Parser Service.
///
/// Parses workflow files and prepares them for database storage.
pub struct ChainParserService<F: ImportFetcher> {
    parser: ChainParser<F>,
}

impl<F: ImportFetcher> ChainParserService<F> {
    /// Create a new service with the given import fetcher.
    pub fn new(fetcher: F) -> Self {
        Self {
            parser: ChainParser::new(fetcher),
        }
    }

    /// Parse a workflow file and prepare it for database storage.
    ///
    /// This validates that the workflow's triggers match the provided context trigger.
    ///
    /// # Errors
    /// Returns an error if parsing fails or triggers don't match.
    pub fn parse(&self, content: &str, context: &WorkflowContext) -> Result<ParsedWorkflow> {
        let source_url = context.source_file_path.as_deref();
        let parsed = self.parser.parse_workflow(content, source_url)?;

        // Validate trigger matches if provided
        if let Some(trigger) = context.trigger {
            let trigger_str = trigger_type_to_str(trigger);
            if !parsed.triggers.iter().any(|t| t == trigger_str) {
                return Err(ParseError::InvalidTrigger(format!(
                    "workflow does not support trigger '{trigger_str}', only: {:?}",
                    parsed.triggers
                )));
            }
        }

        let chain = self.create_new_chain(&parsed, context);
        let fragments = self.create_new_fragments(&parsed, chain.id);

        Ok(ParsedWorkflow { chain, fragments })
    }

    /// Parse a workflow without trigger validation.
    ///
    /// Use this when you want to parse a workflow regardless of trigger type.
    ///
    /// # Errors
    /// Returns an error if parsing fails.
    pub fn parse_without_trigger_validation(
        &self,
        content: &str,
        context: &WorkflowContext,
    ) -> Result<ParsedWorkflow> {
        let source_url = context.source_file_path.as_deref();
        let parsed = self.parser.parse_workflow(content, source_url)?;

        let chain = self.create_new_chain(&parsed, context);
        let fragments = self.create_new_fragments(&parsed, chain.id);

        Ok(ParsedWorkflow { chain, fragments })
    }

    /// Create a `NewChain` from the parsed chain and context.
    fn create_new_chain(&self, parsed: &ParsedChain, context: &WorkflowContext) -> NewChain {
        let mut chain = NewChain::new(context.tenant_id);
        chain.id = parsed.id;
        chain.default_machine = Some(parsed.default_machine.clone());

        if let Some(ref path) = context.source_file_path {
            chain.source_file_path = Some(path.clone());
        }
        if let Some(ref url) = context.repository_url {
            chain.repository_url = Some(url.clone());
        }
        if let Some(ref sha) = context.commit_sha {
            chain.commit_sha = Some(sha.clone());
        }
        if let Some(ref branch) = context.branch {
            chain.branch = Some(branch.clone());
        }
        if let Some(ref trigger) = context.trigger {
            chain.trigger = Some(*trigger);
        }
        if let Some(ref trigger_ref) = context.trigger_ref {
            chain.trigger_ref = Some(trigger_ref.clone());
        }

        chain
    }

    /// Create `NewFragment` records from parsed fragments.
    fn create_new_fragments(&self, parsed: &ParsedChain, chain_id: Uuid) -> Vec<NewFragment> {
        parsed
            .fragments
            .iter()
            .map(|pf| self.convert_fragment(pf, chain_id))
            .collect()
    }

    /// Convert a parsed fragment to a database fragment.
    fn convert_fragment(&self, parsed: &ParsedFragment, chain_id: Uuid) -> NewFragment {
        let fragment_type = match parsed.fragment_type {
            ParsedFragmentType::Inline => FragmentType::Inline,
            ParsedFragmentType::Group => FragmentType::Group,
        };

        let mut fragment = if fragment_type == FragmentType::Group {
            NewFragment::parallel_group(chain_id, parsed.sequence)
        } else {
            NewFragment::inline(
                chain_id,
                parsed.sequence,
                parsed.run_script.clone().unwrap_or_default(),
            )
        };

        fragment.id = parsed.id;
        fragment.parent_fragment_id = parsed.parent_id;
        fragment.is_parallel = parsed.is_parallel;

        if let Some(ref machine) = parsed.machine {
            fragment.machine = Some(machine.clone());
        }
        if let Some(ref condition) = parsed.condition {
            fragment.condition = Some(condition.clone());
        }
        if let Some(ref source_url) = parsed.source_url {
            fragment.source_url = Some(source_url.clone());
        }

        fragment
    }
}

/// Convert a `TriggerType` to its string representation for matching.
const fn trigger_type_to_str(trigger: TriggerType) -> &'static str {
    match trigger {
        TriggerType::Tag => "tag",
        TriggerType::Push => "push",
        TriggerType::PullRequest => "pull_request",
        TriggerType::Schedule => "schedule",
        TriggerType::Manual => "manual",
    }
}
