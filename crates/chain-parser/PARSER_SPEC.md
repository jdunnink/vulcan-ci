# Chain Parser Specification

## Overview

The chain parser converts KDL workflow files into fully-expanded chains stored in the database. Import fragments are recursively resolved at parse time, resulting in a flat structure of executable fragments.

## File Types

### Workflow Files

Complete workflow definitions with `chain` wrapper:

```kdl
version "0.1"
triggers "push" "pull_request"

chain {
    machine "default-worker"

    fragment { run "npm build" }
    fragment { from "https://github.com/org/repo/test.kdl" }
}
```

### Fragment Files

Reusable fragment collections without `chain` wrapper (for importing):

```kdl
fragment { run "npm test:unit" }
fragment { run "npm test:e2e" }
```

## KDL Schema

### Root Elements

| Node | Required | Description |
|------|----------|-------------|
| `version` | Yes | Schema version (currently `"0.1"`) |
| `triggers` | Yes | Event types that trigger this workflow |
| `chain` | Yes | Container for the workflow definition |

### Chain Node

```kdl
chain {
    machine "worker-group"  // Required: default machine for all fragments

    fragment { ... }
    parallel { ... }
}
```

| Child Node | Description |
|------------|-------------|
| `machine` | Required. Default worker group for fragments |
| `fragment` | Inline or import fragment |
| `parallel` | Group of fragments that execute concurrently |

### Fragment Node

```kdl
fragment {
    run "shell script"           // Inline: script to execute
    from "https://url/file.kdl"  // Import: URL to fetch and expand
    machine "worker-group"       // Optional: override chain default
    condition "$VAR == 'value'"  // Optional: skip if false
}
```

| Child Node | Description |
|------------|-------------|
| `run` | Shell script to execute (mutually exclusive with `from`) |
| `from` | URL to import fragments from (mutually exclusive with `run`) |
| `machine` | Worker group override (optional, inherits from chain) |
| `condition` | Expression that must be true for fragment to execute |

### Parallel Node

```kdl
parallel {
    fragment { run "task A" }
    fragment { run "task B" }
}
```

Children of `parallel` execute concurrently. The workflow waits for all children to complete before proceeding.

## Parsing Algorithm

### 1. Parse Root Document

```
parse_workflow(url: String) -> Chain:
    doc = fetch_and_parse_kdl(url)

    version = doc.get("version")
    triggers = doc.get("triggers")
    chain_node = doc.get("chain")

    default_machine = chain_node.get("machine")
    if default_machine is None:
        error("chain must specify default machine")

    fragments = []
    for child in chain_node.children:
        fragments.extend(parse_node(child, default_machine, visited={url}))

    return Chain(
        triggers=triggers,
        default_machine=default_machine,
        fragments=assign_sequences(fragments)
    )
```

### 2. Parse Node (Recursive)

```
parse_node(node, default_machine, visited) -> List<Fragment>:
    if node.name == "fragment":
        return parse_fragment(node, default_machine, visited)
    elif node.name == "parallel":
        return parse_parallel(node, default_machine, visited)
    else:
        error(f"unknown node type: {node.name}")
```

### 3. Parse Fragment

```
parse_fragment(node, default_machine, visited) -> List<Fragment>:
    from_url = node.get("from")
    run_script = node.get("run")

    if from_url and run_script:
        error("fragment cannot have both 'from' and 'run'")

    if from_url:
        // Import: recursively resolve
        return resolve_import(from_url, default_machine, visited)
    else:
        // Inline fragment
        machine = node.get("machine") or default_machine
        condition = node.get("condition")

        return [Fragment(
            type=Inline,
            run_script=run_script,
            machine=machine,
            condition=condition,
            source_url=None
        )]
```

### 4. Resolve Import (Recursive)

```
resolve_import(url, default_machine, visited) -> List<Fragment>:
    if url in visited:
        error(f"circular import detected: {url}")

    visited = visited | {url}
    doc = fetch_and_parse_kdl(url)

    fragments = []
    for node in doc.children:
        if node.name != "fragment" and node.name != "parallel":
            error("imported files can only contain fragment/parallel nodes")

        for frag in parse_node(node, default_machine, visited):
            frag.source_url = url  // Track import origin
            fragments.append(frag)

    return fragments
```

### 5. Parse Parallel Group

```
parse_parallel(node, default_machine, visited) -> List<Fragment>:
    group = Fragment(type=Group, is_parallel=True)

    children = []
    for child in node.children:
        children.extend(parse_node(child, default_machine, visited))

    // Children get parent_fragment_id set to group.id
    return [group] + children
```

### 6. Assign Sequences

```
assign_sequences(fragments) -> List<Fragment>:
    // Group fragments by parent_fragment_id
    // Assign sequence numbers within each group
    // sequence = 0, 1, 2, ... for siblings
```

## Validation Rules

### Required Fields

| Context | Required |
|---------|----------|
| Workflow file | `version`, `triggers`, `chain`, `chain.machine` |
| Fragment with `run` | `run` must be non-empty |
| Fragment with `from` | `from` must be valid URL |

### Mutual Exclusions

- Fragment cannot have both `run` and `from`
- Fragment must have exactly one of `run` or `from`

### Machine Resolution

1. Fragment-level `machine` (highest priority)
2. Chain-level `machine` (default)
3. Error if neither specified

### Circular Import Detection

Track visited URLs during import resolution. Error if same URL encountered twice in the import chain.

## Output: Database Schema

### Chain Table

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `tenant_id` | UUID | Tenant identifier |
| `source_file_path` | TEXT | Original workflow file path |
| `repository_url` | TEXT | Repository containing workflow |
| `commit_sha` | TEXT | Git commit that triggered |
| `branch` | TEXT | Git branch |
| `trigger` | ENUM | Trigger type (tag, push, pull_request, schedule, manual) |
| `trigger_ref` | TEXT | Trigger reference (tag name, PR number, etc.) |
| `default_machine` | TEXT | Default worker group |

### Fragment Table

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `chain_id` | UUID | Parent chain |
| `parent_fragment_id` | UUID | Parent fragment (for parallel groups) |
| `sequence` | INT | Execution order within siblings |
| `type` | ENUM | `inline` or `group` |
| `run_script` | TEXT | Script to execute (inline only) |
| `machine` | TEXT | Worker group (NULL = use chain default) |
| `is_parallel` | BOOL | Children run concurrently |
| `condition` | TEXT | Condition expression |
| `source_url` | TEXT | URL this fragment was imported from |

## Execution Semantics

### Sequential Execution (Default)

Siblings execute in `sequence` order. Each fragment must complete before the next begins.

```
sequence=0 → sequence=1 → sequence=2
```

### Parallel Execution

Children of a `Group` fragment with `is_parallel=true` execute concurrently:

```
       ┌─ child[0] ─┐
Group ─┼─ child[1] ─┼─ (wait for all) → next sibling
       └─ child[2] ─┘
```

### Conditional Execution

If `condition` is set, evaluate before execution:
- `true` → execute fragment (and children)
- `false` → skip fragment (and children)

Condition expressions can reference environment variables:
- `$BRANCH` — Git branch name
- `$TRIGGER` — Trigger type (push, pull_request, tag, etc.)
- `$COMMIT_SHA` — Git commit SHA
- `$PR_NUMBER` — Pull request number (if applicable)

## Example: Full Expansion

### Input Files

**workflow.kdl:**
```kdl
version "0.1"
triggers "push"

chain {
    machine "default"
    fragment { from "https://example.com/build.kdl" }
    fragment { run "deploy.sh" }
}
```

**build.kdl:**
```kdl
fragment { run "npm install" }
fragment { run "npm build" }
```

### Output (Stored in DB)

```
Chain:
  default_machine: "default"

Fragments:
  [0] type=inline, sequence=0, run_script="npm install", source_url="https://example.com/build.kdl"
  [1] type=inline, sequence=1, run_script="npm build", source_url="https://example.com/build.kdl"
  [2] type=inline, sequence=2, run_script="deploy.sh", source_url=NULL
```

## Error Handling

| Error | Description |
|-------|-------------|
| `InvalidSyntax` | KDL parsing failed |
| `MissingRequired` | Required field not present |
| `InvalidUrl` | Import URL is malformed |
| `FetchFailed` | Could not retrieve import URL |
| `CircularImport` | Import cycle detected |
| `MutualExclusion` | Both `run` and `from` specified |
| `NoMachine` | No machine specified at chain or fragment level |
