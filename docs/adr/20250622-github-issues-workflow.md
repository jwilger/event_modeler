# ADR: Use GitHub Issues for Work Tracking

## Date
2025-06-22

## Status
Accepted

## Context
We have been using file-based planning (PLANNING.md and ROADMAP.md) to track development work, phases, and current status. This approach has several limitations:

1. **Manual updates**: Status must be manually updated in markdown files
2. **No dependency tracking**: Relationships between tasks are text-based
3. **Limited automation**: Difficult for tools to parse and understand work state
4. **Version control noise**: Frequent updates to planning files create commit noise
5. **No native workflow integration**: Can't leverage GitHub's built-in features

## Decision
We will migrate to using GitHub Issues with epics and sub-issues for all work tracking, while preserving critical development rules in dedicated documentation files.

### Implementation Details
- **Epics**: Major development phases labeled with "epic"
- **Sub-issues**: Individual work items linked via GitHub's sub-issue feature
- **Dependencies**: Tracked using "Depends on #X" pattern in issue descriptions
- **Critical rules**: Preserved in DEVELOPMENT_PROCESS.md
- **Vision/roadmap**: Kept in ROADMAP.md but referencing GitHub for current status
- **GitHub Project**: Repository-level project board for priority management
  - Manual sort order determines priority (not labels or fields)
  - Epics sorted in "Epics Overview" view
  - Sub-issues sorted within status columns on Kanban board

## Consequences

### Positive
- **Native GitHub integration**: Leverage labels, milestones, projects, and automation
- **MCP compatibility**: Enables the MCP workflow server to query work status via GitHub API
- **Better visibility**: Work status visible in GitHub UI without opening files
- **Dependency tracking**: GitHub's sub-issue feature provides clear relationships
- **Reduced commit noise**: No more commits just to update status
- **Standard workflow**: Aligns with common open-source practices
- **Visual priority management**: Drag-and-drop prioritization in project board
- **Flexible organization**: Multiple views for different perspectives

### Negative
- **External dependency**: Work tracking requires GitHub access
- **Learning curve**: Contributors need to understand GitHub issues workflow
- **API complexity**: MCP server needs to use GraphQL for sub-issue queries
- **Less centralized**: Information spread across issues rather than single file

## Rationale
The migration to GitHub issues aligns with our MCP development goals, where the workflow server needs programmatic access to work status. GitHub's API provides structured data that's easier for automation tools to consume than parsing markdown files.

This approach also scales better as the project grows and enables better collaboration with external contributors who are already familiar with GitHub's issue tracking.

## Related ADRs
- [Use MCP Server Instead of GitHub Integration](20250622-mcp-over-github-integration.md) - The MCP server will interact with GitHub issues