# ADR: Use MCP Server Instead of GitHub @claude Integration

## Date
2025-06-22

## Status
Accepted

## Context
We initially attempted to implement automated development workflows using GitHub Actions with the @claude integration. This approach would have allowed Claude to automatically respond to GitHub issues and PRs when mentioned.

The official claude-code-action GitHub Action only supports the standard Claude API, not Claude Max subscriptions. While a forked version (grll/claude-code-action) was found that supports OAuth authentication for Claude Max accounts, this approach proved problematic.

## Decision
We will implement an MCP (Model Context Protocol) server for development automation instead of using the GitHub @claude integration.

## Consequences

### Positive
- **Leverages existing Claude Max subscription**: The MCP implementation works through Claude Code, which already uses the Claude Max account, avoiding duplicate API costs
- **No credential management overhead**: MCP runs locally and uses the existing Claude Code authentication, eliminating the need to manage OAuth tokens in GitHub secrets
- **Full control over workflow**: Custom MCP implementation allows us to tailor the automation exactly to our needs
- **No expiration issues**: Local MCP server doesn't suffer from the OAuth token expiration problems that plague the GitHub integration

### Negative
- **Local execution required**: Unlike GitHub Actions, MCP requires Claude Code to be running locally
- **More implementation work**: We need to build the MCP server ourselves rather than using pre-built actions
- **No automatic triggers**: Developers must manually invoke the MCP tools rather than having automatic responses to GitHub events

## Rationale
The decision to implement an MCP server is driven by the challenges outlined in the "Consequences" section, including the high maintenance burden of OAuth token management and the economic inefficiency of using the standard Claude API alongside a Claude Max subscription.

By leveraging the MCP approach, we can achieve the same automation goals while avoiding these issues and working within the constraints of the Claude Max subscription model.