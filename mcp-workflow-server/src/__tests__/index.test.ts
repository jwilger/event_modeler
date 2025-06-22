import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';

// Mock the modules
vi.mock('@modelcontextprotocol/sdk/server/index.js');
vi.mock('@modelcontextprotocol/sdk/server/stdio.js');
vi.mock('../tools/workflow-status.js');

describe('MCP Server Index', () => {
  let mockServer: any;
  let handlers: Map<any, any>;

  beforeEach(() => {
    handlers = new Map();
    mockServer = {
      setRequestHandler: vi.fn((schema, handler) => {
        handlers.set(schema, handler);
      }),
      connect: vi.fn(),
    };
    
    vi.mocked(Server).mockImplementation(() => mockServer);
    
    // Clear module cache to ensure fresh import
    vi.resetModules();
  });

  it('should register workflow_status tool with correct name', async () => {
    // Import the module to trigger registration
    await import('../index.js');

    // Get the ListToolsRequestSchema handler
    const listToolsHandler = handlers.get(ListToolsRequestSchema);
    expect(listToolsHandler).toBeDefined();

    // Call the handler to get the tools list
    const result = await listToolsHandler();

    // Verify the workflow_status tool is registered with correct name
    expect(result.tools).toHaveLength(1);
    expect(result.tools[0]).toMatchObject({
      name: 'workflow_status',
      description: expect.stringContaining('comprehensive status'),
      inputSchema: {
        type: 'object',
        properties: {},
        required: [],
      },
    });
  });

  it('should handle workflow_status tool execution', async () => {
    const { CallToolRequestSchema } = await import('@modelcontextprotocol/sdk/types.js');
    const { workflowStatusTool } = await import('../tools/workflow-status.js');
    
    // Mock the workflow status tool
    vi.mocked(workflowStatusTool).mockResolvedValue({
      requestedData: { gitStatus: null, currentBranch: 'main', openPRCount: 0 },
      automaticActions: [],
      issuesFound: [],
      suggestedActions: [],
      allPRStatus: [],
    });

    // Import the module to trigger registration
    await import('../index.js');

    // Get the CallToolRequestSchema handler
    const callToolHandler = handlers.get(CallToolRequestSchema);
    expect(callToolHandler).toBeDefined();

    // Call the handler with workflow_status tool
    const result = await callToolHandler({
      params: { name: 'workflow_status' },
    });

    // Verify the response
    expect(result.content).toHaveLength(1);
    expect(result.content[0].type).toBe('text');
    expect(JSON.parse(result.content[0].text)).toMatchObject({
      requestedData: expect.any(Object),
      automaticActions: expect.any(Array),
    });
  });

  it('should return error for unknown tool', async () => {
    const { CallToolRequestSchema } = await import('@modelcontextprotocol/sdk/types.js');
    
    // Import the module to trigger registration
    await import('../index.js');

    // Get the CallToolRequestSchema handler
    const callToolHandler = handlers.get(CallToolRequestSchema);
    
    // Call with unknown tool
    const result = await callToolHandler({
      params: { name: 'unknown_tool' },
    });

    // Verify error response
    expect(result.content[0].type).toBe('text');
    const response = JSON.parse(result.content[0].text);
    expect(response.issuesFound).toContain('Error: Unknown tool: unknown_tool');
  });
});