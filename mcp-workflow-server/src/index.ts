#!/usr/bin/env node
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';

import { workflowStatusTool } from './tools/workflow-status.js';
import { workflowNext } from './tools/workflow-next.js';
import { workflowDecide } from './tools/workflow-decide.js';
import { workflowConfigure } from './tools/workflow-configure.js';
import { workflowCreatePR } from './tools/workflow-create-pr.js';
import { workflowMonitorReviews } from './tools/workflow-monitor-reviews.js';
import { workflowReplyReview, workflowReplyReviewTool } from './tools/workflow-reply-review.js';
import { workflowRequestReview, workflowRequestReviewTool } from './tools/workflow-request-review.js';
import { workflowManageSubissues } from './tools/workflow-manage-subissues.js';
import { workflowCreateIssue } from './tools/workflow-create-issue.js';
import { gitBranch } from './tools/git-branch.js';
import { gitCommit } from './tools/git-commit.js';
import { gitStash } from './tools/git-stash.js';
import { WorkflowResponse } from './types.js';

const server = new Server(
  {
    name: 'event-modeler-workflow',
    version: '0.1.0',
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

// Register available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  console.error('ListTools handler called - returning tools');
  return {
    tools: [
      {
        name: 'workflow_status',
        description:
          'Get comprehensive status of current git branch, all open PRs, CI status, and detect issues like stale branches or needed rebases',
        inputSchema: {
          type: 'object',
          properties: {},
          required: [],
        },
      },
      {
        name: 'workflow_next',
        description:
          'Get context-aware guidance on what to work on next based on assigned GitHub issues',
        inputSchema: {
          type: 'object',
          properties: {},
          required: [],
        },
      },
      {
        name: 'workflow_decide',
        description:
          'Submit a decision for a previous LLM decision request from workflow_next',
        inputSchema: {
          type: 'object',
          properties: {
            decisionId: {
              type: 'string',
              description: 'The decision ID from the requires_llm_decision response',
            },
            selectedChoice: {
              type: ['string', 'number'],
              description: 'The ID of the selected choice',
            },
            reasoning: {
              type: 'string',
              description: 'Optional reasoning for the decision',
            },
          },
          required: ['decisionId', 'selectedChoice'],
        },
      },
      {
        name: 'workflow_configure',
        description:
          'Configure the MCP workflow server with project-specific settings',
        inputSchema: {
          type: 'object',
          properties: {
            projectNumber: {
              type: 'number',
              description: 'GitHub project number',
            },
            projectId: {
              type: 'string',
              description: 'GitHub project ID (e.g., PVT_...)',
            },
            statusFieldId: {
              type: 'string',
              description: 'ID of the Status field in the project',
            },
            todoOptionId: {
              type: 'string',
              description: 'ID of the Todo status option',
            },
            inProgressOptionId: {
              type: 'string',
              description: 'ID of the In Progress status option',
            },
            doneOptionId: {
              type: 'string',
              description: 'ID of the Done status option',
            },
          },
          required: [],
        },
      },
      {
        name: 'workflow_create_pr',
        description:
          'Create a pull request with smart PR generation from commits and issues',
        inputSchema: {
          type: 'object',
          properties: {
            baseBranch: {
              type: 'string',
              description: 'Base branch for the PR (defaults to main/master)',
            },
            draft: {
              type: 'boolean',
              description: 'Create as draft PR',
            },
          },
          required: [],
        },
      },
      {
        name: 'workflow_monitor_reviews',
        description:
          'Monitor PR reviews across the repository, detect feedback needing attention, and format for LLM action',
        inputSchema: {
          type: 'object',
          properties: {
            includeApproved: {
              type: 'boolean',
              description: 'Include already approved PRs in response',
            },
            includeDrafts: {
              type: 'boolean',
              description: 'Include draft PRs in monitoring',
            },
          },
          required: [],
        },
      },
      workflowReplyReviewTool,
      workflowRequestReviewTool,
      {
        name: 'workflow_manage_subissues',
        description:
          'Manage GitHub sub-issues: link issues to epics, unlink sub-issues, or list all sub-issues for an epic',
        inputSchema: {
          type: 'object',
          properties: {
            action: {
              type: 'string',
              enum: ['link', 'unlink', 'list'],
              description: 'The action to perform',
            },
            epicNumber: {
              type: 'number',
              description: 'The epic issue number',
            },
            issueNumber: {
              type: 'number',
              description: 'The issue number to link/unlink (required for link/unlink actions)',
            },
          },
          required: ['action', 'epicNumber'],
        },
      },
      {
        name: 'workflow_create_issue',
        description:
          'Create GitHub issues with project metadata, optionally linking to epics as sub-issues',
        inputSchema: {
          type: 'object',
          properties: {
            title: {
              type: 'string',
              description: 'Issue title',
            },
            body: {
              type: 'string',
              description: 'Issue body/description',
            },
            epicNumber: {
              type: 'number',
              description: 'Epic issue number to link as sub-issue (optional)',
            },
            type: {
              type: 'string',
              enum: ['bug', 'feature', 'enhancement', 'documentation', 'question'],
              description: 'Issue type (optional)',
            },
            priority: {
              type: 'string',
              enum: ['low', 'medium', 'high', 'urgent'],
              description: 'Issue priority (optional)',
            },
            labels: {
              type: 'array',
              items: { type: 'string' },
              description: 'Labels to add to the issue (optional)',
            },
            assignees: {
              type: 'array',
              items: { type: 'string' },
              description: 'Usernames to assign the issue to (optional)',
            },
          },
          required: ['title', 'body'],
        },
      },
      {
        name: 'git_branch',
        description:
          'Manage Git branches: checkout, create, pull, push, list branches, or start work on an issue',
        inputSchema: {
          type: 'object',
          properties: {
            action: {
              type: 'string',
              enum: ['checkout', 'create', 'pull', 'push', 'list', 'start-work'],
              description: 'The git branch operation to perform',
            },
            branch: {
              type: 'string',
              description: 'Branch name (for checkout, create, push)',
            },
            issueNumber: {
              type: 'number',
              description: 'Issue number to create branch from (for create, start-work)',
            },
            force: {
              type: 'boolean',
              description: 'Force operation even with uncommitted changes',
            },
          },
          required: ['action'],
        },
      },
      {
        name: 'git_commit',
        description:
          'Git commit operations: stage files, create commits with auto-formatting, amend commits, and run pre-commit checks',
        inputSchema: {
          type: 'object',
          properties: {
            action: {
              type: 'string',
              enum: ['stage', 'unstage', 'status', 'commit', 'amend'],
              description: 'The git commit operation to perform',
            },
            files: {
              type: 'array',
              items: { type: 'string' },
              description: 'Files to stage/unstage (optional - defaults to all)',
            },
            message: {
              type: 'string',
              description: 'Commit message (required for commit/amend)',
            },
            issueNumber: {
              type: 'number',
              description: 'Issue number to reference (auto-detected if not provided)',
            },
            all: {
              type: 'boolean',
              description: 'For stage - stage all tracked files',
            },
          },
          required: ['action'],
        },
      },
      {
        name: 'git_stash',
        description:
          'Git stash operations: save, pop, apply, list, drop, clear stashes with auto-generated messages',
        inputSchema: {
          type: 'object',
          properties: {
            action: {
              type: 'string',
              enum: ['list', 'save', 'pop', 'apply', 'drop', 'clear', 'show'],
              description: 'The git stash operation to perform',
            },
            message: {
              type: 'string',
              description: 'Custom message for save action (auto-generated if not provided)',
            },
            stashRef: {
              type: ['string', 'number'],
              description: 'Stash reference for pop/apply/drop/show (index or stash@{n})',
            },
            includeUntracked: {
              type: 'boolean',
              description: 'Include untracked files when saving (--include-untracked)',
            },
            keepIndex: {
              type: 'boolean',
              description: 'Keep staged changes in index when saving (--keep-index)',
            },
            quiet: {
              type: 'boolean',
              description: 'Suppress output for pop/apply operations',
            },
          },
          required: ['action'],
        },
      },
    ],
  };
});

// Handle tool execution
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name } = request.params;

  try {
    let result: WorkflowResponse;

    switch (name) {
      case 'workflow_status':
        result = await workflowStatusTool();
        break;
      case 'workflow_next':
        result = await workflowNext();
        break;
      case 'workflow_decide':
        result = await workflowDecide(request.params.arguments as { decisionId: string; selectedChoice: string | number; reasoning?: string });
        break;
      case 'workflow_configure':
        result = await workflowConfigure(request.params.arguments || {});
        break;
      case 'workflow_create_pr':
        result = await workflowCreatePR(request.params.arguments || {});
        break;
      case 'workflow_monitor_reviews':
        result = await workflowMonitorReviews(request.params.arguments || {});
        break;
      case 'workflow_reply_review':
        result = await workflowReplyReview(request.params.arguments as { prNumber: number; commentId: number; body: string });
        break;
      case 'workflow_request_review':
        result = await workflowRequestReview(request.params.arguments as { prNumber: number; reviewers?: string[]; skipBots?: boolean });
        break;
      case 'workflow_manage_subissues':
        result = await workflowManageSubissues(request.params.arguments as { action: 'link' | 'unlink' | 'list'; epicNumber: number; issueNumber?: number });
        break;
      case 'workflow_create_issue':
        result = await workflowCreateIssue(request.params.arguments as { title: string; body: string; epicNumber?: number; type?: 'bug' | 'feature' | 'enhancement' | 'documentation' | 'question'; priority?: 'low' | 'medium' | 'high' | 'urgent'; labels?: string[]; assignees?: string[] });
        break;
      case 'git_branch':
        result = await gitBranch(request.params.arguments as { action: 'checkout' | 'create' | 'pull' | 'push' | 'list' | 'start-work'; branch?: string; issueNumber?: number; force?: boolean });
        break;
      case 'git_commit':
        result = await gitCommit(request.params.arguments as { action: 'stage' | 'unstage' | 'status' | 'commit' | 'amend'; files?: string[]; message?: string; issueNumber?: number; all?: boolean });
        break;
      case 'git_stash':
        result = await gitStash(request.params.arguments as { action: 'list' | 'save' | 'pop' | 'apply' | 'drop' | 'clear' | 'show'; message?: string; stashRef?: string | number; includeUntracked?: boolean; keepIndex?: boolean; quiet?: boolean });
        break;
      default:
        throw new Error(`Unknown tool: ${name}`);
    }

    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(result, null, 2),
        },
      ],
    };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(
            {
              requestedData: null,
              automaticActions: [],
              issuesFound: [`Error: ${errorMessage}`],
              suggestedActions: ['Fix the error and try again'],
              allPRStatus: [],
            } satisfies WorkflowResponse,
            null,
            2
          ),
        },
      ],
    };
  }
});

// Start the server
const transport = new StdioServerTransport();
server.connect(transport);

console.error('Event Modeler MCP Workflow Server started');
console.error('Handlers registered');