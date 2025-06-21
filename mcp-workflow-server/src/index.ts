#!/usr/bin/env node
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';

import { workflowStatusTool } from './tools/workflow-status.js';
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
        name: 'workflow/status',
        description:
          'Get comprehensive status of current git branch, all open PRs, CI status, and detect issues like stale branches or needed rebases',
        inputSchema: {
          type: 'object',
          properties: {},
          required: [],
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
      case 'workflow/status':
        result = await workflowStatusTool();
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