# GitHub Project Setup

This document describes the GitHub Project configuration for Event Modeler development.

## Project Overview

**Project**: Event Modeler Development  
**URL**: https://github.com/users/jwilger/projects/9  
**Type**: Repository project (public)  
**Style**: Kanban board with priority-based workflow

## Custom Fields

### Priority (Single Select)
- 🔴 P0: Critical - Must be done immediately
- 🟠 P1: High - Should be done soon  
- 🟡 P2: Medium - Normal priority
- 🟢 P3: Low - Nice to have

### Type (Single Select)
- 🎯 Epic - Major feature or phase
- ✨ Feature - New functionality
- 🐛 Bug - Something broken
- 📝 Documentation - Documentation updates
- 🔧 Maintenance - Refactoring, dependencies, etc

### Complexity (Number)
- 1-5 scale for effort estimation
- 1 = Trivial (< 1 hour)
- 2 = Small (few hours)
- 3 = Medium (1-2 days)
- 4 = Large (3-5 days)
- 5 = Extra Large (1+ week)

## Built-in Fields Used

- **Status**: Todo, In Progress, Done
- **Parent issue**: Shows epic relationships
- **Sub-issues progress**: Shows completion percentage for epics
- **Assignees**: Who's working on it
- **Labels**: Additional categorization
- **Repository**: Source repository

## Recommended Views

### 1. Kanban Board (Board Layout)
- **Group by**: Status
- **Sort by**: Priority (desc), Created (asc)
- **Filter**: Exclude Done items older than 2 weeks
- **Purpose**: Main work tracking view

### 2. Epics Overview (Table Layout)
- **Filter**: Type = Epic OR has sub-issues
- **Columns**: Title, Status, Sub-issues progress, Priority
- **Sort by**: Priority (desc)
- **Purpose**: Track major initiatives

### 3. Ready to Work (Table Layout)
- **Filter**: Status = Todo, No assignee
- **Sort by**: Priority (desc)
- **Purpose**: Help contributors find available work

### 4. Roadmap (Roadmap Layout)
- **Filter**: Type = Epic
- **Group by**: Quarter
- **Purpose**: Long-term planning view

## Automation Recommendations

While GitHub Projects doesn't support built-in automation rules via API, you can:

1. Use GitHub Actions to update project fields when:
   - Issues are opened/closed
   - PRs are created/merged
   - Labels are added/removed

2. The MCP workflow server can update fields via GraphQL API

## MCP Integration

The MCP workflow server can interact with this project to:
- Query work priorities
- Update issue status
- Find next available work
- Track epic progress
- Generate reports

### Key GraphQL Queries

```graphql
# Get project items by priority
{
  node(id: "PROJECT_ID") {
    ... on ProjectV2 {
      items(first: 100, orderBy: {field: POSITION, direction: ASC}) {
        nodes {
          fieldValueByName(name: "Priority") {
            ... on ProjectV2ItemFieldSingleSelectValue {
              name
            }
          }
          content {
            ... on Issue {
              number
              title
              state
            }
          }
        }
      }
    }
  }
}
```

## Workflow Guidelines

1. **New Issues**: Default to "Todo" status
2. **Priority Assignment**: 
   - Epics: Based on roadmap importance
   - Features: Based on epic priority
   - Bugs: Based on severity
3. **Status Transitions**:
   - Todo → In Progress (when assigned)
   - In Progress → Done (when PR merged)
4. **Archival**: Done items auto-archive after 30 days