# MCP Workflow Server

An MCP (Model Context Protocol) server that helps manage the Event Modeler development workflow by providing intelligent status monitoring and automation tools.

## Features

### Workflow Tools

#### `workflow_status` (Phase 1 - Complete)
Provides comprehensive repository and PR status monitoring:
- **Git Status**: Current branch, uncommitted changes, ahead/behind status
- **PR Monitoring**: CI status, review comments, rebase needs
- **Stale Branch Detection**: Identifies branches created before recent main merges
- **State Persistence**: Tracks changes between invocations to detect new issues

#### `workflow_next` (Phase 2 - Complete)
Intelligent guidance on what to work on next based on assigned GitHub issues.

#### `workflow_create_pr` (Phase 3 - Complete)
Smart PR creation with automatic title and description generation from commits and issues.

#### `workflow_monitor_reviews` (Phase 4 - Complete)
Monitor PR reviews across the repository and detect feedback needing attention.

### Git Tools

#### `git_branch`
Manage Git branches with operations:
- `checkout`: Switch branches with uncommitted change detection
- `create`: Create new branches, optionally from issue numbers
- `pull`: Pull latest changes
- `push`: Push branches to remote
- `list`: List all branches with tracking info
- `start-work`: High-level action to start work on an issue

#### `git_commit`
Git commit operations with smart formatting:
- `stage`: Stage files for commit
- `unstage`: Unstage files
- `status`: Show current Git status
- `commit`: Create commits with auto-generated messages
- `amend`: Amend the last commit

#### `git_stash`
Git stash operations for managing work in progress:
- `list`: Show all stashes
- `save`: Stash changes with auto-generated descriptions
- `pop`: Apply and remove a stash
- `apply`: Apply a stash without removing it
- `drop`: Remove a specific stash
- `clear`: Remove all stashes
- `show`: Display stash contents

Features:
- Auto-generates stash descriptions from current branch and issue context
- Warns before stashing changes related to current issue
- Integrates with branch switching workflow
- Handles stash conflicts gracefully

## Installation & Setup

### Prerequisites
- Node.js 18+ 
- Claude Code CLI
- GitHub CLI (`gh`) for authentication

### Building the Server

```bash
cd mcp-workflow-server
npm install
npm run build
```

### Integration with Claude Code

The MCP server is automatically configured for this project via `.mcp.json` in the project root. When you open this project in Claude Code, it will detect and offer to use the workflow server.

To manually verify the configuration:

```bash
# From project root
claude mcp list
```

You should see the `workflow` server listed.

## Usage

Once integrated, you can use the workflow tools in Claude Code:

```
Use the workflow_status tool to check my current work status
```

The tool will return:
- Current branch and Git status
- Any active PRs and their CI/review status
- Detected issues (stale branches, failing CI, etc.)
- Suggested actions based on the current state

## Development

### Running Tests
```bash
npm test
```

### Development Mode
```bash
npm run dev
```

### Linting & Formatting
```bash
npm run lint
npm run format
```

## Architecture

The server implements the MCP protocol with:
- **Tools**: Exposed functionality (currently `workflow_status`)
- **State Store**: Persistent tracking of PR reviews and branch dates
- **Utilities**: Git and GitHub API integrations

## Roadmap

- **Phase 2**: Intelligent next-step guidance
- **Phase 3**: Smart PR creation
- **Phase 4**: Active review monitoring
- **Phase 5**: Automated merge & rebase operations

See [PLANNING.md](../PLANNING.md) for detailed implementation plans.