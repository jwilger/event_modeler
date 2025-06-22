# Development Process

This document contains critical development rules and patterns for Event Modeler development.

## 🚨 MANDATORY FIRST STEP 🚨

**ALWAYS create a todo list using TodoWrite as your VERY FIRST action when starting work.**

**IMPORTANT**: Since CLAUDE.md is only loaded at conversation start, review the critical rules:
- Always create feature branches before committing
- Always use `nutype` for domain types
- Never use `.unwrap()` or `.expect()` on validation
- Always run `cargo fmt` and `cargo clippy` before commits

## 🔴 CRITICAL Development Rules 🔴

### Visual Development Rule (Phase 6)
1. Do NOT commit until satisfied with visual output comparison
2. Test locally: Generate SVG → Convert to PNG → Compare with gold standard
3. Only commit after visual match is achieved

### Branch Management Rule
**ALWAYS** update main before creating new branches:
```bash
git checkout main
git pull origin main
git checkout -b feature/your-branch-name
```

### PR Process Rule
1. Create PRs as ready (not draft) to trigger automated reviews
2. Monitor for ALL reviews (Copilot, user, or any other reviewer)
3. Address ALL feedback before merge
4. Use GraphQL API for review replies (prefix with `[Response from Claude Code]`)
5. PR merge = approval to proceed

### Todo List Management Rule
The LAST task on every todo list must be:
- Normal: "Check GitHub issues for next work, update status, and START implementing"
- Phase 6: "Check GitHub issues, update status, and await approval before proceeding"

## PR-Driven Development Workflow

### Quick Reference:
1. Update main → Create feature branch
2. Implement → Commit frequently → Push
3. Create PR immediately after first push (NOT draft)
4. Monitor reviews → Address feedback
5. When merged → Update main → Next feature

### 🚨 CRITICAL: Rebase After Squash Merges
When base PR merges, IMMEDIATELY rebase downstream PRs:
```bash
git checkout feature/<downstream>
git fetch origin
git rebase origin/main
# Resolve conflicts with --theirs
git push --force-with-lease
gh pr edit <PR-number> --base main
```

## Todo List Pattern

Every todo list must follow this pattern:
1. Implementation task
2. Run build and tests; commit and push if passing
3. Check PR status after push
4. [Repeat 1-3 for each task]
5. Check GitHub issues, update status, [continue or await approval]

**Phase 6 Exception**: Visual development suspends frequent commits - only commit after visual verification.

## Testing Requirements

### For Rust Code:
- Run `cargo test --workspace` before commits
- Run `cargo fmt --all` and `cargo clippy --workspace --all-targets`
- All tests must pass, no clippy warnings

### For TypeScript/MCP Code:
- Run `npm test` in the mcp-workflow-server directory
- Run `npm run lint` and fix any issues
- Ensure TypeScript builds without errors

## Branch Naming Conventions

- Feature branches: `feature/descriptive-name`
- Phase 6 steps: `feature/diagram-step-{number}-{element-name}`
- MCP phases: `feature/mcp-phase-{number}-{description}`
- Fixes: `fix/issue-description`

## Notes

- Performance is not a priority (per ADR)
- Focus on correctness through types
- See [CLAUDE.md](CLAUDE.md) for architecture principles
- See [README.md](README.md) for contribution guidelines