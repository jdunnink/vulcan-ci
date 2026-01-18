# Trigger Feature Gap Analysis

This document tracks the feature gap between Vulcan CI's trigger support and GitHub Actions.

## Current Implementation

```kdl
triggers "push" "pull_request" "tag"
```

Simple list of event type strings with no configuration options.

### Supported Trigger Types

| Type | Description |
|------|-------------|
| `push` | Triggered on git push |
| `pull_request` | Triggered on PR events |
| `tag` | Triggered on tag creation |
| `schedule` | Triggered on schedule (not configurable) |
| `manual` | Triggered manually |

## GitHub Actions Feature Comparison

### Branch Filtering

**GitHub Actions:**
```yaml
on:
  push:
    branches:
      - main
      - 'releases/**'
    branches-ignore:
      - 'feature/**'
```

**Vulcan CI:** Not supported

**Priority:** High - Essential for most CI workflows

---

### Path Filtering

**GitHub Actions:**
```yaml
on:
  push:
    paths:
      - 'src/**'
      - '*.rs'
    paths-ignore:
      - 'docs/**'
      - '*.md'
```

**Vulcan CI:** Not supported

**Priority:** High - Reduces unnecessary builds significantly

---

### Tag Filtering

**GitHub Actions:**
```yaml
on:
  push:
    tags:
      - 'v*'
      - '!v*-beta'
```

**Vulcan CI:** Not supported (only has generic `tag` trigger)

**Priority:** Medium - Important for release workflows

---

### Pull Request Event Types

**GitHub Actions:**
```yaml
on:
  pull_request:
    types:
      - opened
      - synchronize
      - reopened
      - closed
      - labeled
      - unlabeled
      - ready_for_review
      - converted_to_draft
```

**Vulcan CI:** Not supported - triggers on all PR events

**Priority:** Medium - Useful for fine-grained control

---

### Schedule (Cron)

**GitHub Actions:**
```yaml
on:
  schedule:
    - cron: '0 2 * * *'
    - cron: '0 14 * * 1-5'
```

**Vulcan CI:** `schedule` type exists but cron expression not configurable

**Priority:** Medium - Required for nightly builds, cleanup jobs

---

### Manual Trigger with Inputs

**GitHub Actions:**
```yaml
on:
  workflow_dispatch:
    inputs:
      environment:
        description: 'Target environment'
        required: true
        type: choice
        options:
          - staging
          - production
      dry_run:
        description: 'Run without deploying'
        type: boolean
        default: false
```

**Vulcan CI:** `manual` type exists but no input support

**Priority:** High - Essential for deployment workflows

---

### Reusable Workflows

**GitHub Actions:**
```yaml
on:
  workflow_call:
    inputs:
      config_path:
        required: true
        type: string
    secrets:
      deploy_key:
        required: true
    outputs:
      artifact_url:
        value: ${{ jobs.build.outputs.url }}
```

**Vulcan CI:** Not supported (imports are static file includes)

**Priority:** Low - Current import system covers basic reuse

---

### Repository Dispatch (Custom Webhooks)

**GitHub Actions:**
```yaml
on:
  repository_dispatch:
    types: [deploy, rollback]
```

**Vulcan CI:** Not supported

**Priority:** Low - Advanced use case

---

### Concurrency Control

**GitHub Actions:**
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

**Vulcan CI:** Not supported

**Priority:** Medium - Prevents redundant builds

---

### Other GitHub Actions Events

Not currently planned for Vulcan CI:

- `issues` / `issue_comment`
- `release`
- `deployment` / `deployment_status`
- `check_run` / `check_suite`
- `create` / `delete`
- `fork` / `watch` / `star`
- `discussion` / `discussion_comment`
- `project` / `project_card` / `project_column`
- `registry_package`
- `workflow_run`

## Proposed KDL Schema

### Phase 1: Branch/Path/Tag Filtering

```kdl
triggers {
    push {
        branches "main" "develop" "releases/**"
        branches-ignore "feature/**"
        paths "src/**" "Cargo.toml"
        paths-ignore "docs/**" "*.md"
    }
    pull_request {
        branches "main"
        paths "src/**"
    }
    tag {
        patterns "v*"
        ignore "v*-beta" "v*-rc*"
    }
}
```

### Phase 2: Event Types and Schedule

```kdl
triggers {
    pull_request {
        types "opened" "synchronize" "reopened"
        branches "main"
    }
    schedule cron="0 2 * * *"
}
```

### Phase 3: Manual Inputs

```kdl
triggers {
    manual {
        input "environment" {
            type "choice"
            options "staging" "production"
            required true
        }
        input "dry_run" {
            type "boolean"
            default false
        }
    }
}
```

### Phase 4: Concurrency

```kdl
concurrency {
    group "$WORKFLOW-$BRANCH"
    cancel-in-progress true
}
```

## Implementation Priority

| Priority | Feature | Effort |
|----------|---------|--------|
| 1 | Branch filtering | Medium |
| 2 | Path filtering | Medium |
| 3 | Manual inputs | Medium |
| 4 | Cron schedules | Low |
| 5 | Tag patterns | Low |
| 6 | PR event types | Low |
| 7 | Concurrency control | Medium |
| 8 | Reusable workflow inputs/outputs | High |

## Migration Considerations

When implementing these features:

1. **Backward Compatibility**: Simple `triggers "push" "pull_request"` syntax should continue to work
2. **Database Schema**: May need new tables for trigger configurations, manual inputs
3. **Trigger Matching**: Service layer needs logic to evaluate branch/path patterns
4. **UI Integration**: Manual inputs require frontend support for dispatch UI
