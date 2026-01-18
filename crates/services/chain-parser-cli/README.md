# Vulcan Chain Parser CLI

Command-line tool for validating and inspecting KDL workflow files.

## Status

**Complete** - Fully functional.

## Installation

```bash
cargo install --path crates/services/chain-parser-cli
```

Or run directly:

```bash
cargo run -p vulcan-chain-parser-cli -- <workflow.kdl>
```

## Usage

```
vulcan-parse - Validate and inspect Vulcan CI workflow files

USAGE:
    vulcan-parse <workflow.kdl> [OPTIONS]

ARGS:
    <workflow.kdl>    Path to the KDL workflow file to parse

OPTIONS:
    --base-path <dir>    Base directory for resolving imports (default: file's directory)
    --quiet              Only output errors, no success details
    --help               Print this help message
```

## Examples

Validate a workflow file:

```bash
vulcan-parse .vulcan/ci.kdl
```

Validate with a specific import base path:

```bash
vulcan-parse .vulcan/ci.kdl --base-path .vulcan/fragments
```

Validate silently (for CI scripts):

```bash
vulcan-parse .vulcan/ci.kdl --quiet && echo "Valid"
```

## Output

On success, the CLI displays the parsed chain and fragment structure:

```
Parsed workflow successfully!

Chain:
  ID: 550e8400-e29b-41d4-a716-446655440000
  Default Machine: Some("default-worker")

Fragments (3):
  [0] ID: 550e8400-e29b-41d4-a716-446655440001
      Type: Inline
      Sequence: 0
      Script: npm install

  [1] ID: 550e8400-e29b-41d4-a716-446655440002
      Type: Group
      Sequence: 1

  [2] ID: 550e8400-e29b-41d4-a716-446655440003
      Type: Inline
      Sequence: 0
      Parent: 550e8400-e29b-41d4-a716-446655440002
      Script: npm test
```

On error, the CLI prints the error message and exits with code 1:

```
Parse error in '.vulcan/ci.kdl': Missing required 'version' field
```

## Import Resolution

The CLI resolves imports by extracting the filename from URLs and looking for the file in the base path directory. For example:

- Import URL: `https://github.com/org/shared/checkout.kdl`
- With `--base-path ./fragments`
- Resolves to: `./fragments/checkout.kdl`
