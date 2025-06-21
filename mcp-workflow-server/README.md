# Event Modeler MCP Workflow Server

An MCP (Model Context Protocol) server that manages the Event Modeler development workflow, providing deterministic guidance and automated GitHub/Git operations.

## Overview

This MCP server automates and streamlines the development workflow for Event Modeler by:
- Providing proactive monitoring of PR status, CI checks, and review feedback
- Automating PR chain management and rebasing
- Giving deterministic next-step instructions
- Handling GitHub API interactions directly

## Features

### Phase 1: Project Setup & Smart Status (Current)
- Intelligent status monitoring of git and GitHub state
- Detection of stale branches and needed rebases
- Comprehensive PR status overview

### Planned Features
- Phase 2: Context-aware next step guidance
- Phase 3: Automated PR creation with monitoring
- Phase 4: Active review and CI monitoring
- Phase 5: Automated merge handling and PR chain management

## Installation

```bash
cd mcp-workflow-server
npm install
npm run build
```

## Usage

The server provides MCP tools that can be accessed through any MCP-compatible client:

- `workflow/status` - Get comprehensive status of current branch, PRs, and CI

## Development

```bash
# Run in development mode with hot reload
npm run dev

# Run tests
npm test

# Lint code
npm run lint

# Format code
npm run format
```

## Architecture

Every MCP tool response includes:
```typescript
{
  requestedData: {...},      // What was asked for
  automaticActions: [...],   // What MCP did automatically  
  issuesFound: [...],        // Problems detected
  suggestedActions: [...],   // What LLM should do
  allPRStatus: [...]         // Always include PR overview
}
```

This proactive monitoring architecture ensures that every interaction provides comprehensive context for decision-making.