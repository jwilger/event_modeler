# Event Modeler Implementation Plan

This document tracks current work and critical development processes for the Event Modeler rewrite.

## 🚨 MANDATORY FIRST STEP 🚨

**ALWAYS create a todo list using TodoWrite as your VERY FIRST action when starting work.**

**IMPORTANT**: Since CLAUDE.md is only loaded at conversation start, review the critical rules in [CLAUDE.md](CLAUDE.md):
- Always create feature branches before committing
- Always use `nutype` for domain types
- Never use `.unwrap()` or `.expect()` on validation
- Always run `cargo fmt` and `cargo clippy` before commits

## Current Status

**Last Updated**: 2025-06-22
- Main Project: Phase 6 Step 3 (Slice Headers) - PR #31 ready for merge
- MCP Server: Phase 1 COMPLETE - PR #34 ready for merge

**Active Work**:
- Phase 6 (Incremental Diagram Module Rewrite) IN PROGRESS
  - Step 0-2: ✅ COMPLETE 
  - Step 3: Slice Headers (PR #31 ready for merge)
  - Step 4-26: Not started
- MCP Workflow Server: Phase 1 complete, Phase 2 next

**Completed Phases**: 1-5 (Type System, YAML Parser, Domain Model, Layout Engine, Basic Rendering)

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
- Normal: "Review PLANNING.md, update with current status, determine next tasks, and START implementing them"
- Phase 6: "Review PLANNING.md, update with current status, and await approval before proceeding"

## Phase 6: Incremental Diagram Module Rewrite

**Goal**: Build diagram module incrementally with visual validation at each step.

**Current Progress**:
- ✅ Step 0: Delete and Initialize (PR #28)
- ✅ Step 1: Canvas and Workflow Title (PR #29)
- ✅ Step 2: All Swimlanes (PR #30)
- 🔄 Step 3: Slice Headers (PR #31 ready for merge)

### Remaining Steps (4-26):

**Entity Rendering** (Steps 4-10):
4. View Entities - White boxes
5. Command Entities - Blue boxes
6. Event Entities - Purple boxes
7. Projection Entities - Yellow boxes
8. Query Entities - Blue boxes
9. Automation Entities - Circular icons
10. Entity Connections - Arrows

**Layout & Polish** (Steps 11-26):
11. Layout Algorithm
12-22. [Additional entity features and refinements]
23-25. Test Scenarios Section
26. Final Adjustments

### Phase 6 Requirements:
- ALL code in library (`src/diagram/`), not test binaries
- Dynamic layout for ANY valid .eventmodel file
- Visual style matching example.png (style, not pixels)
- Branch naming: `feature/diagram-step-{number}-{element-name}`

## MCP Workflow Server Development

**Goal**: Replace manual workflow processes with automated MCP server.

### Phase 1: Project Setup & Smart Status ✅ COMPLETE
- Created foundation with `workflow/status` tool
- PR #34 ready for merge

### Phase 2: Intelligent Next Step (Next)
- Context-aware guidance for what to do next
- Replace manual "check PLANNING.md" process

### Phase 3: Smart PR Creation
- Automated PR creation with proper templates
- Immediate review monitoring

### Phase 4: Active Review Monitoring
- Proactive monitoring of all reviews
- Format feedback for LLM action

### Phase 5: Automated Merge & Rebase
- Handle PR chain management automatically
- Replace manual rebase instructions

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
5. Review PLANNING.md, update status, [continue or await approval]

**Phase 6 Exception**: Visual development suspends frequent commits - only commit after visual verification.

## Success Criteria

Phase 6 complete when:
- All 26 steps implemented
- Diagram matches example.png structure
- Dynamic layout works for any .eventmodel
- All tests pass, no clippy warnings

MCP Server complete when:
- All 5 phases implemented
- 90% of manual processes automated
- Can extract to separate repository

## Notes

- See [CLAUDE.md](CLAUDE.md) for architecture principles
- See [README.md](README.md) for contribution guidelines
- Performance is not a priority (per ADR)
- Focus on correctness through types