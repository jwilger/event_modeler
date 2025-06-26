import { describe, it, expect } from 'vitest';

describe('Schema Validation Tests', () => {
  it('should be able to serialize all tool schemas to JSON', async () => {
    // Import all tools that export tool definitions
    const toolModules = [
      '../tools/workflow-request-review.js',
      '../tools/workflow-reply-review.js',
    ];

    for (const modulePath of toolModules) {
      const module = await import(modulePath);

      // Find exported tool definitions
      const toolExports = Object.entries(module).filter(([key]) => key.endsWith('Tool'));

      for (const [, tool] of toolExports) {
        if (typeof tool === 'object' && tool !== null && 'inputSchema' in tool) {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          const inputSchema = (tool as any).inputSchema;

          // Verify it's a plain object, not a Zod schema
          expect(inputSchema).not.toHaveProperty('_def'); // Zod schemas have _def
          expect(inputSchema).not.toHaveProperty('parse'); // Zod schemas have parse

          // Verify it's a valid JSON Schema structure
          expect(inputSchema).toHaveProperty('type');
          expect(inputSchema.type).toBe('object');
          if ('properties' in inputSchema) {
            expect(typeof inputSchema.properties).toBe('object');
          }

          // Verify it can be serialized - this is the key test that would have caught our bug
          expect(() => JSON.stringify(tool)).not.toThrow();

          // Verify serialization doesn't produce [object Object]
          const serialized = JSON.stringify(tool);
          expect(serialized).not.toContain('[object Object]');
        }
      }
    }
  });

  it('should validate MCP tool schema requirements', async () => {
    const { workflowRequestReviewTool } = await import('../tools/workflow-request-review.js');

    // MCP requires these properties
    expect(workflowRequestReviewTool).toHaveProperty('name');
    expect(workflowRequestReviewTool).toHaveProperty('description');
    expect(workflowRequestReviewTool).toHaveProperty('inputSchema');

    // Input schema must be JSON Schema format
    const schema = workflowRequestReviewTool.inputSchema;
    expect(schema).toHaveProperty('type');
    expect(schema.type).toBe('object');
    if ('properties' in schema) {
      expect(schema.properties).toBeDefined();
    }
  });
});
