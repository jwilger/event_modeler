import { describe, it, expect, afterAll } from 'vitest';
import { spawn, ChildProcess } from 'child_process';

describe('MCP Server Integration Tests', () => {
  describe('Tool Registration', () => {
    it('should be able to serialize all tool schemas to JSON', async () => {
      const { workflowRequestReviewTool } = await import('../tools/workflow-request-review.js');
      const { workflowReplyReviewTool } = await import('../tools/workflow-reply-review.js');

      // This would have caught our bug - Zod schemas can't be serialized
      expect(() => JSON.stringify(workflowRequestReviewTool)).not.toThrow();
      expect(() => JSON.stringify(workflowReplyReviewTool)).not.toThrow();

      // Verify the serialized form is valid
      const serialized = JSON.stringify(workflowRequestReviewTool);
      const parsed = JSON.parse(serialized);
      expect(parsed.inputSchema).toBeDefined();
      expect(parsed.inputSchema.type).toBe('object');
    });
  });

  describe('Full Server Startup', () => {
    let serverProcess: ChildProcess;

    afterAll(() => {
      if (serverProcess) {
        serverProcess.kill();
      }
    });

    it('should start without errors and respond to ListTools', async () => {
      return new Promise<void>((resolve, reject) => {
        serverProcess = spawn('node', ['dist/index.js'], {
          cwd: process.cwd(),
          env: { ...process.env },
        });

        let stderr = '';
        let stdout = '';
        serverProcess.stderr?.on('data', (data) => {
          stderr += data.toString();
        });

        serverProcess.stdout?.on('data', (data) => {
          stdout += data.toString();

          // Try to parse each line as JSON-RPC
          const lines = stdout.split('\n');
          for (const line of lines) {
            if (line.trim() && line.includes('"jsonrpc"')) {
              try {
                const response = JSON.parse(line);
                if (response.result && response.result.tools) {
                  expect(response.result.tools).toBeDefined();
                  expect(response.result.tools.length).toBeGreaterThan(0);
                  resolve();
                  return;
                }
              } catch {
                // Not a complete JSON line yet
              }
            }
          }
        });

        // Send a ListTools request via stdin after a brief delay
        setTimeout(() => {
          const request = {
            jsonrpc: '2.0',
            method: 'tools/list',
            params: {},
            id: 1,
          };
          serverProcess.stdin?.write(JSON.stringify(request) + '\n');
        }, 100);

        // Fail if server crashes
        serverProcess.on('error', (error) => {
          reject(new Error(`Server process error: ${error.message}\nStderr: ${stderr}`));
        });

        serverProcess.on('exit', (code) => {
          if (code !== 0 && code !== null) {
            reject(new Error(`Server exited with code ${code}\nStderr: ${stderr}`));
          }
        });

        // Timeout after 5 seconds
        setTimeout(() => {
          reject(
            new Error(`Timeout waiting for server response\nStderr: ${stderr}\nStdout: ${stdout}`)
          );
        }, 5000);
      });
    }, 10000);
  });
});
