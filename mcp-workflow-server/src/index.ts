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
import { workflowManageSubissues } from './tools/workflow-manage-subissues.js';
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
      case 'workflow_manage_subissues':
        result = await workflowManageSubissues(request.params.arguments as { action: 'link' | 'unlink' | 'list'; epicNumber: number; issueNumber?: number });
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