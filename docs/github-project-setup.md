# GitHub Project Setup

This document describes the GitHub Project configuration for Event Modeler development.

## Project Overview

**Project**: Event Modeler Development  
**URL**: https://github.com/users/jwilger/projects/9  
**Type**: Repository project (public)  
**Style**: Kanban board with priority-based workflow

## Custom Fields

### Status (Built-in)
- Todo - Not yet started
- In Progress - Currently being worked on
- Done - Completed

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

## Project Workflows

GitHub Projects has limited built-in automations:

### Default Workflows (Enabled)
1. **Item closed** → Status set to "Done"
2. **Pull request merged** → Status set to "Done"

### Optional Workflows (Can Enable)
1. **Item added to project** → Status set to "Todo"
2. **Auto-close issue** → Close issue when status changes to "Done"

### Manual Processes
Since GitHub doesn't provide workflows for these, they must be done manually:
- Moving issues to "In Progress" when assigned
- Setting Type field based on issue characteristics
- Archiving old Done items

The MCP workflow server can help automate some of these through the API.

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

## Priority System

GitHub Projects uses **manual sort order** to determine priority:

### Epic Priority
- Epics are manually sorted in the "Epics Overview" view
- Top-to-bottom order = highest-to-lowest priority
- This determines which epic's work should be done first

### Sub-Issue Priority  
- Sub-issues are manually sorted within their Status column on the Kanban board
- When grouped by parent issue, the order within each group determines priority
- Top-to-bottom order = highest-to-lowest priority

### Work Selection Algorithm
1. Find the highest priority epic (by position in Epics view)
2. Within that epic, find the highest priority "Todo" sub-issue
3. Check that all dependencies are met
4. That's the next item to work on

The Priority field (P0-P3) is for additional context but **manual sort order is the source of truth**.

## Workflow Guidelines

1. **New Issues**: Default to "Todo" status
2. **Priority Management**: 
   - Drag epics in "Epics Overview" to set epic priority
   - Drag sub-issues within Status columns to set task priority
   - Use Priority field (P0-P3) for severity/urgency indicators
3. **Status Transitions**:
   - Todo → In Progress (when assigned)
   - In Progress → Done (when PR merged)
4. **Archival**: Done items auto-archive after 30 days