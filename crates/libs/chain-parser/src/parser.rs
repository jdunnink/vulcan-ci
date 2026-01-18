//! KDL parser for workflow and fragment files.
//!
//! This module parses KDL files into the intermediate AST representation.
//! Import resolution is handled separately by the resolver module.

use std::collections::HashSet;

use kdl::{KdlDocument, KdlNode};
use uuid::Uuid;

use crate::ast::{ParsedChain, ParsedFragment};
use crate::error::{ParseError, Result};

/// Fetcher trait for resolving import URLs.
///
/// This allows for different implementations (HTTP, file system, mock for testing).
pub trait ImportFetcher {
    /// Fetch the content at the given URL.
    ///
    /// # Errors
    /// Returns an error if the URL cannot be fetched.
    fn fetch(&self, url: &str) -> Result<String>;
}

/// Parser for KDL workflow files.
pub struct ChainParser<F: ImportFetcher> {
    fetcher: F,
}

impl<F: ImportFetcher> ChainParser<F> {
    /// Create a new parser with the given import fetcher.
    pub const fn new(fetcher: F) -> Self {
        Self { fetcher }
    }

    /// Parse a workflow file from its content.
    ///
    /// # Errors
    /// Returns an error if the content is not valid KDL or doesn't match the workflow schema.
    pub fn parse_workflow(&self, content: &str, source_url: Option<&str>) -> Result<ParsedChain> {
        let doc: KdlDocument = content
            .parse()
            .map_err(|e: kdl::KdlError| ParseError::InvalidSyntax(e.to_string()))?;

        // Parse version
        let version = get_string_value(&doc, "version").ok_or_else(|| ParseError::MissingRequired {
            field: "version",
            context: "workflow root".to_string(),
        })?;

        if version != "0.1" {
            return Err(ParseError::UnsupportedVersion(version));
        }

        // Parse triggers
        let triggers = get_string_args(&doc, "triggers").ok_or_else(|| ParseError::MissingRequired {
            field: "triggers",
            context: "workflow root".to_string(),
        })?;

        // Parse chain node
        let chain_node = doc
            .nodes()
            .iter()
            .find(|n| n.name().value() == "chain")
            .ok_or_else(|| ParseError::MissingRequired {
                field: "chain",
                context: "workflow root".to_string(),
            })?;

        let chain_doc = chain_node.children().ok_or_else(|| ParseError::MissingRequired {
            field: "chain children",
            context: "chain node".to_string(),
        })?;

        // Get default machine
        let default_machine =
            get_string_value(chain_doc, "machine").ok_or_else(|| ParseError::MissingRequired {
                field: "machine",
                context: "chain node".to_string(),
            })?;

        // Track visited URLs for circular import detection
        let mut visited = HashSet::new();
        if let Some(url) = source_url {
            visited.insert(url.to_string());
        }

        // Parse fragments
        let mut fragments = Vec::new();
        let mut sequence = 0;

        for node in chain_doc.nodes() {
            let name = node.name().value();
            if name == "machine" {
                continue; // Already processed
            }

            let parsed = self.parse_node(node, &default_machine, &mut visited, None)?;
            for mut frag in parsed {
                if frag.parent_id.is_none() {
                    frag.sequence = sequence;
                    sequence += 1;
                }
                fragments.push(frag);
            }
        }

        Ok(ParsedChain {
            id: Uuid::new_v4(),
            triggers,
            default_machine,
            fragments,
        })
    }

    /// Parse a fragment file (no chain wrapper, just fragments).
    ///
    /// Used for resolving imports.
    ///
    /// # Errors
    /// Returns an error if the content is not valid KDL or contains invalid nodes.
    pub fn parse_fragment_file(
        &self,
        content: &str,
        source_url: &str,
        default_machine: &str,
        visited: &mut HashSet<String>,
    ) -> Result<Vec<ParsedFragment>> {
        let doc: KdlDocument = content
            .parse()
            .map_err(|e: kdl::KdlError| ParseError::InvalidSyntax(e.to_string()))?;

        let mut fragments = Vec::new();

        for node in doc.nodes() {
            let name = node.name().value();
            if name != "fragment" && name != "parallel" {
                return Err(ParseError::InvalidImportNode(name.to_string()));
            }

            let parsed = self.parse_node(node, default_machine, visited, None)?;
            for mut frag in parsed {
                // Mark all fragments as coming from this import
                if frag.source_url.is_none() {
                    frag.source_url = Some(source_url.to_string());
                }
                fragments.push(frag);
            }
        }

        Ok(fragments)
    }

    /// Parse a node (fragment or parallel) recursively.
    fn parse_node(
        &self,
        node: &KdlNode,
        default_machine: &str,
        visited: &mut HashSet<String>,
        parent_id: Option<Uuid>,
    ) -> Result<Vec<ParsedFragment>> {
        match node.name().value() {
            "fragment" => self.parse_fragment(node, default_machine, visited, parent_id),
            "parallel" => self.parse_parallel(node, default_machine, visited, parent_id),
            other => Err(ParseError::UnknownNode(other.to_string())),
        }
    }

    /// Parse a fragment node.
    fn parse_fragment(
        &self,
        node: &KdlNode,
        default_machine: &str,
        visited: &mut HashSet<String>,
        parent_id: Option<Uuid>,
    ) -> Result<Vec<ParsedFragment>> {
        let children = node.children();

        let from_url = children.and_then(|c| get_string_value(c, "from"));
        let run_script = children.and_then(|c| get_string_value(c, "run"));

        // Check mutual exclusion
        if from_url.is_some() && run_script.is_some() {
            return Err(ParseError::MutualExclusion);
        }

        if from_url.is_none() && run_script.is_none() {
            return Err(ParseError::NoContent);
        }

        if let Some(url) = from_url {
            // Import: recursively resolve
            self.resolve_import(&url, default_machine, visited, parent_id)
        } else {
            // Inline fragment
            let machine = children
                .and_then(|c| get_string_value(c, "machine"))
                .unwrap_or_else(|| default_machine.to_string());

            let condition = children.and_then(|c| get_string_value(c, "condition"));

            let mut fragment = ParsedFragment::inline(0, run_script.expect("run_script checked above"))
                .with_machine(machine);

            if let Some(cond) = condition {
                fragment = fragment.with_condition(cond);
            }

            if let Some(pid) = parent_id {
                fragment = fragment.with_parent(pid);
            }

            Ok(vec![fragment])
        }
    }

    /// Resolve an import URL recursively.
    fn resolve_import(
        &self,
        url: &str,
        default_machine: &str,
        visited: &mut HashSet<String>,
        parent_id: Option<Uuid>,
    ) -> Result<Vec<ParsedFragment>> {
        // Check for circular imports
        if visited.contains(url) {
            return Err(ParseError::CircularImport(url.to_string()));
        }

        visited.insert(url.to_string());

        // Fetch the content
        let content = self.fetcher.fetch(url)?;

        // Parse as fragment file
        let mut fragments = self.parse_fragment_file(&content, url, default_machine, visited)?;

        // Set parent_id on top-level fragments from this import
        if let Some(pid) = parent_id {
            for frag in &mut fragments {
                if frag.parent_id.is_none() {
                    frag.parent_id = Some(pid);
                }
            }
        }

        Ok(fragments)
    }

    /// Parse a parallel node.
    fn parse_parallel(
        &self,
        node: &KdlNode,
        default_machine: &str,
        visited: &mut HashSet<String>,
        parent_id: Option<Uuid>,
    ) -> Result<Vec<ParsedFragment>> {
        let mut group = ParsedFragment::parallel_group(0);

        if let Some(pid) = parent_id {
            group = group.with_parent(pid);
        }

        let group_id = group.id;
        let mut result = vec![group];

        if let Some(children) = node.children() {
            let mut child_sequence = 0;
            for child_node in children.nodes() {
                let parsed = self.parse_node(child_node, default_machine, visited, Some(group_id))?;
                for mut frag in parsed {
                    // Only set sequence for direct children (not nested)
                    if frag.parent_id == Some(group_id) {
                        frag.sequence = child_sequence;
                        child_sequence += 1;
                    }
                    result.push(frag);
                }
            }
        }

        Ok(result)
    }
}

/// Get a string value from a node's first argument.
fn get_string_value(doc: &KdlDocument, node_name: &str) -> Option<String> {
    doc.nodes()
        .iter()
        .find(|n| n.name().value() == node_name)
        .and_then(|node| node.entries().first())
        .and_then(|entry| entry.value().as_string())
        .map(String::from)
}

/// Get all string arguments from a node.
fn get_string_args(doc: &KdlDocument, node_name: &str) -> Option<Vec<String>> {
    doc.nodes()
        .iter()
        .find(|n| n.name().value() == node_name)
        .map(|node| {
            node.entries()
                .iter()
                .filter_map(|e| e.value().as_string().map(String::from))
                .collect::<Vec<_>>()
        })
        .filter(|args| !args.is_empty())
}
