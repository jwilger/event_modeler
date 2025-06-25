import { describe, it, expect, vi, beforeEach } from 'vitest';
import { workflowManageSubissues } from '../workflow-manage-subissues.js';

// Mock child_process
vi.mock('child_process', () => ({
  execSync: vi.fn((cmd: string) => {
    if (cmd === 'gh auth token') return 'test-token';
    if (cmd.startsWith('git remote get-url')) return 'https://github.com/testuser/testrepo.git';
    return '';
  }),
}));

// Mock @octokit/rest
const mockGraphql = vi.fn();
vi.mock('@octokit/rest', () => ({
  Octokit: vi.fn(() => ({
    graphql: mockGraphql,
  })),
}));

// Mock github utils
vi.mock('../../utils/github.js', () => ({
  getRepoInfo: vi.fn(() => ({
    owner: 'testuser',
    repo: 'testrepo',
  })),
}));

describe('workflowManageSubissues', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockGraphql.mockReset();
  });

  describe('link action', () => {
    it('should link an issue as a sub-issue to an epic', async () => {
      // Mock getting node IDs
      mockGraphql
        .mockResolvedValueOnce({
          repository: {
            issue: {
              id: 'epic-node-id',
              title: 'Epic Issue',
            },
          },
        })
        .mockResolvedValueOnce({
          repository: {
            issue: {
              id: 'issue-node-id',
              title: 'Sub Issue',
            },
          },
        })
        // Mock circular dependency check
        .mockResolvedValueOnce({
          node: {
            parentIssue: null,
          },
        })
        // Mock linking mutation
        .mockResolvedValueOnce({
          addSubIssue: {
            issue: { number: 68, title: 'Epic Issue' },
            subIssue: { number: 123, title: 'Sub Issue' },
          },
        });

      const result = await workflowManageSubissues({
        action: 'link',
        epicNumber: 68,
        issueNumber: 123,
      });

      expect(result.requestedData.success).toBe(true);
      expect(result.automaticActions).toContain('Successfully linked issue #123 to epic #68');
      expect(mockGraphql).toHaveBeenCalledTimes(4);
    });

    it('should prevent circular dependencies', async () => {
      // Mock getting node IDs
      mockGraphql
        .mockResolvedValueOnce({
          repository: {
            issue: {
              id: 'epic-node-id',
              title: 'Epic Issue',
            },
          },
        })
        .mockResolvedValueOnce({
          repository: {
            issue: {
              id: 'issue-node-id',
              title: 'Sub Issue',
            },
          },
        })
        // Mock circular dependency check - epic has issue as parent
        .mockResolvedValueOnce({
          node: {
            parentIssue: {
              id: 'issue-node-id',
            },
          },
        });

      const result = await workflowManageSubissues({
        action: 'link',
        epicNumber: 68,
        issueNumber: 123,
      });

      expect(result.requestedData.success).toBe(false);
      expect(result.requestedData.error).toContain('would create circular dependency');
      expect(mockGraphql).toHaveBeenCalledTimes(3); // No linking mutation
    });

    it('should handle missing issue number for link action', async () => {
      const result = await workflowManageSubissues({
        action: 'link',
        epicNumber: 68,
      });

      expect(result.requestedData.success).toBe(false);
      expect(result.requestedData.error).toContain('issueNumber is required for link action');
    });
  });

  describe('unlink action', () => {
    it('should unlink a sub-issue from an epic', async () => {
      // Mock getting node IDs
      mockGraphql
        .mockResolvedValueOnce({
          repository: {
            issue: {
              id: 'epic-node-id',
              title: 'Epic Issue',
            },
          },
        })
        .mockResolvedValueOnce({
          repository: {
            issue: {
              id: 'issue-node-id',
              title: 'Sub Issue',
            },
          },
        })
        // Mock unlinking mutation
        .mockResolvedValueOnce({
          removeSubIssue: {
            issue: { number: 68, title: 'Epic Issue' },
            subIssue: { number: 123, title: 'Sub Issue' },
          },
        });

      const result = await workflowManageSubissues({
        action: 'unlink',
        epicNumber: 68,
        issueNumber: 123,
      });

      expect(result.requestedData.success).toBe(true);
      expect(result.automaticActions).toContain('Successfully unlinked issue #123 from epic #68');
      expect(mockGraphql).toHaveBeenCalledTimes(3);
    });
  });

  describe('list action', () => {
    it('should list all sub-issues for an epic', async () => {
      mockGraphql.mockResolvedValueOnce({
        repository: {
          issue: {
            title: 'Epic: Test Epic',
            subIssues: {
              nodes: [
                {
                  number: 123,
                  title: 'Sub Issue 1',
                  state: 'OPEN',
                  url: 'https://github.com/testuser/testrepo/issues/123',
                  assignees: {
                    nodes: [{ login: 'user1' }],
                  },
                },
                {
                  number: 124,
                  title: 'Sub Issue 2',
                  state: 'CLOSED',
                  url: 'https://github.com/testuser/testrepo/issues/124',
                  assignees: {
                    nodes: [],
                  },
                },
              ],
            },
          },
        },
      });

      const result = await workflowManageSubissues({
        action: 'list',
        epicNumber: 68,
      });

      expect(result.requestedData.success).toBe(true);
      expect(result.requestedData.subIssues).toHaveLength(2);
      expect(result.requestedData.subIssues![0]).toEqual({
        number: 123,
        title: 'Sub Issue 1',
        state: 'OPEN',
        url: 'https://github.com/testuser/testrepo/issues/123',
        assignees: ['user1'],
      });
      expect(result.automaticActions).toContain('Found 2 sub-issues for epic #68');
      expect(result.suggestedActions).toContain('🟢 #123: Sub Issue 1 (assigned to: user1)');
      expect(result.suggestedActions).toContain('✅ #124: Sub Issue 2');
    });

    it('should handle epic with no sub-issues', async () => {
      mockGraphql.mockResolvedValueOnce({
        repository: {
          issue: {
            title: 'Epic: Empty Epic',
            subIssues: {
              nodes: [],
            },
          },
        },
      });

      const result = await workflowManageSubissues({
        action: 'list',
        epicNumber: 68,
      });

      expect(result.requestedData.success).toBe(true);
      expect(result.requestedData.subIssues).toHaveLength(0);
      expect(result.automaticActions).toContain('Found 0 sub-issues for epic #68');
    });

    it('should handle non-existent epic', async () => {
      mockGraphql.mockResolvedValueOnce({
        repository: {
          issue: null,
        },
      });

      const result = await workflowManageSubissues({
        action: 'list',
        epicNumber: 999,
      });

      expect(result.requestedData.success).toBe(false);
      expect(result.requestedData.error).toContain('Epic #999 not found');
    });
  });

  describe('error handling', () => {
    it('should handle missing required parameters', async () => {
      const result = await workflowManageSubissues({
        action: 'link',
        epicNumber: 0, // Invalid
      });

      expect(result.requestedData.success).toBe(false);
      expect(result.issuesFound).toHaveLength(1);
    });

    it('should handle unknown action', async () => {
      const result = await workflowManageSubissues({
        action: 'invalid' as 'link' | 'unlink' | 'list',
        epicNumber: 68,
      });

      expect(result.requestedData.success).toBe(false);
      expect(result.requestedData.error).toContain('Unknown action: invalid');
    });

    it('should handle GraphQL errors', async () => {
      mockGraphql.mockRejectedValueOnce(new Error('GraphQL error'));

      const result = await workflowManageSubissues({
        action: 'list',
        epicNumber: 68,
      });

      expect(result.requestedData.success).toBe(false);
      expect(result.requestedData.error).toContain('GraphQL error');
    });
  });
});