# MCP Todo Management Specification

This document describes how the MCP workflow server manages todo lists within GitHub issues.

## Overview

The MCP maintains todo checklists in issue bodies to track granular progress on implementation tasks. This provides visibility into work progress and helps the LLM understand what specific task to work on next.

## Todo List Format

Todo lists are maintained as GitHub-flavored markdown checklists in the issue body:

```markdown
## Acceptance Criteria

- [x] Implement basic structure
- [ ] Add validation logic
- [ ] Write unit tests
- [ ] Update documentation
```

## MCP Tool Responsibilities

### `workflow_next` Tool
- Reads issue bodies to find todo checklists
- Identifies the next unchecked item
- Returns specific guidance on what to work on

### `workflow_update_todos` Tool (Future Phase 3)
- Updates todo items as completed
- Adds new todos if subtasks are discovered
- Maintains todo list formatting

## Todo List Patterns

### Standard Implementation Pattern
```markdown
## Implementation Tasks

- [ ] Create initial module structure
- [ ] Implement core functionality
- [ ] Add error handling
- [ ] Write tests
- [ ] Update documentation
- [ ] Run linting and formatting
```

### Bug Fix Pattern
```markdown
## Fix Tasks

- [ ] Reproduce the issue
- [ ] Identify root cause
- [ ] Implement fix
- [ ] Add regression test
- [ ] Verify fix resolves issue
```

### Review Feedback Pattern
```markdown
## Review Feedback

- [ ] Address comment on line 42
- [ ] Refactor suggested function
- [ ] Add requested test case
- [ ] Update documentation per feedback
```

## Integration with Project Board

1. Issues with incomplete todos remain "In Progress"
2. When all todos are checked, issue is ready for:
   - PR creation (if none exists)
   - Status update to "In Review" (if PR exists)
   - Closure (if PR is merged)

## Best Practices

1. **Granular Tasks**: Break work into 1-2 hour chunks
2. **Clear Descriptions**: Each todo should be self-explanatory
3. **Ordered Execution**: Todos should be in logical order
4. **Dynamic Updates**: Add todos as new requirements emerge
5. **Review Integration**: Add todos for addressing review feedback

## Example Workflow

1. User assigns themselves to issue #42
2. MCP reads issue, finds todo list
3. MCP identifies first unchecked item: "Implement rendering for View entity type"
4. LLM works on that specific task
5. When complete, MCP updates todo to checked
6. MCP identifies next unchecked item
7. Process repeats until all todos complete
8. MCP suggests creating PR or updating status

This approach ensures systematic progress through complex tasks while maintaining visibility for both human and AI collaborators.