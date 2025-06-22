# MCP Workflow Server

An MCP (Model Context Protocol) server that helps manage the Event Modeler development workflow by providing intelligent status monitoring and automation tools.

## Features

### Phase 1: Workflow Status (Complete)
The `workflow_status` tool provides comprehensive repository and PR status monitoring:

- **Git Status**: Current branch, uncommitted changes, ahead/behind status
- **PR Monitoring**: CI status, review comments, rebase needs
- **Stale Branch Detection**: Identifies branches created before recent main merges
- **State Persistence**: Tracks changes between invocations to detect new issues

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