import { describe, it, expect, vi, beforeEach } from 'vitest';
import { workflowCreateIssue } from '../workflow-create-issue.js';

// Mock child_process
vi.mock('child_process', () => ({
  execSync: vi.fn((cmd: string) => {
    if (cmd === 'gh auth token') return 'test-token';
    if (cmd === 'git config --get remote.origin.url') return 'https://github.com/testuser/testrepo.git';
    return '';
  }),
}));

// Mock @octokit/rest
const mockCreate = vi.fn();
const mockGraphql = vi.fn();
vi.mock('@octokit/rest', () => ({
  Octokit: vi.fn(() => ({
    issues: {
      create: mockCreate,
    },
    graphql: mockGraphql,
  })),
}));

// Don't mock utils/github.js - let it use real function with mocked execSync

// Mock config.js
vi.mock('../config.js', () => ({
  getProjectConfig: vi.fn(() => ({
    config: {
      github: {
        projectId: 'PVT_123',
        statusFieldId: 'field_123',
        statusOptions: {
          todo: 'option_todo',
          inProgress: 'option_progress',
          done: 'option_done',
        },
      },
    },
    isComplete: true,
  })),
}));

describe('workflowCreateIssue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockCreate.mockReset();
    mockGraphql.mockReset();
  });

  describe('basic issue creation', () => {
    it('should create issue with title and body', async () => {
      const mockIssue = {
        data: {
          number: 123,
          title: 'Test Issue',
          html_url: 'https://github.com/testuser/testrepo/issues/123',
          state: 'open',
        },
      };

      mockCreate.mockResolvedValue(mockIssue);
      mockGraphql
        .mockResolvedValueOnce({
          repository: { issue: { id: 'issue_node_id', title: 'Test Issue' } },
        })
        .mockResolvedValueOnce({
          addProjectV2ItemById: { item: { id: 'item_123' } },
        })
        .mockResolvedValueOnce({
          updateProjectV2ItemFieldValue: {
            projectV2Item: { id: 'item_123' },
          },
        });

      const result = await workflowCreateIssue({
        title: 'Test Issue',
        body: 'Test description',
      });

      expect(result.requestedData.issue).toEqual({
        number: 123,
        title: 'Test Issue',
        url: 'https://github.com/testuser/testrepo/issues/123',
        state: 'open',
      });

      expect(mockCreate).toHaveBeenCalledWith({
        owner: 'testuser',
        repo: 'testrepo',
        title: 'Test Issue',
        body: 'Test description',
        labels: [],
        assignees: [],
      });

      expect(result.automaticActions).toContain('Created issue #123: Test Issue');
      expect(result.suggestedActions).toContain('View issue: https://github.com/testuser/testrepo/issues/123');
    });

    it('should fail without required title', async () => {
      const result = await workflowCreateIssue({
        title: '',
        body: 'Test description',
      });

      expect(result.requestedData.error).toBe('Missing required parameters: title and body are required');
      expect(result.issuesFound).toContain('Error: Missing required parameters: title and body are required');
    });

    it('should fail without required body', async () => {
      const result = await workflowCreateIssue({
        title: 'Test Issue',
        body: '',
      });

      expect(result.requestedData.error).toBe('Missing required parameters: title and body are required');
      expect(result.issuesFound).toContain('Error: Missing required parameters: title and body are required');
    });
  });

  describe('labels and metadata', () => {
    it('should create issue with type and priority labels', async () => {
      const mockIssue = {
        data: {
          number: 124,
          title: 'Bug Fix',
          html_url: 'https://github.com/testuser/testrepo/issues/124',
          state: 'open',
        },
      };

      mockCreate.mockResolvedValue(mockIssue);
      mockGraphql
        .mockResolvedValueOnce({
          repository: { issue: { id: 'issue_node_id', title: 'Bug Fix' } },
        })
        .mockResolvedValueOnce({
          addProjectV2ItemById: { item: { id: 'item_124' } },
        })
        .mockResolvedValueOnce({
          updateProjectV2ItemFieldValue: {
            projectV2Item: { id: 'item_124' },
          },
        });

      const result = await workflowCreateIssue({
        title: 'Bug Fix',
        body: 'Fix critical bug',
        type: 'bug',
        priority: 'high',
      });

      expect(mockCreate).toHaveBeenCalledWith({
        owner: 'testuser',
        repo: 'testrepo',
        title: 'Bug Fix',
        body: 'Fix critical bug',
        labels: ['bug', 'high'],
        assignees: [],
      });

      expect(result.automaticActions).toContain('Labels: bug, high');
    });

    it('should create issue with custom labels and assignees', async () => {
      const mockIssue = {
        data: {
          number: 125,
          title: 'Feature Request',
          html_url: 'https://github.com/testuser/testrepo/issues/125',
          state: 'open',
        },
      };

      mockCreate.mockResolvedValue(mockIssue);
      mockGraphql
        .mockResolvedValueOnce({
          repository: { issue: { id: 'issue_node_id', title: 'Feature Request' } },
        })
        .mockResolvedValueOnce({
          addProjectV2ItemById: { item: { id: 'item_125' } },
        })
        .mockResolvedValueOnce({
          updateProjectV2ItemFieldValue: {
            projectV2Item: { id: 'item_125' },
          },
        });

      const result = await workflowCreateIssue({
        title: 'Feature Request',
        body: 'Add new feature',
        labels: ['enhancement', 'ui'],
        assignees: ['developer1', 'developer2'],
      });

      expect(mockCreate).toHaveBeenCalledWith({
        owner: 'testuser',
        repo: 'testrepo',
        title: 'Feature Request',
        body: 'Add new feature',
        labels: ['enhancement', 'ui'],
        assignees: ['developer1', 'developer2'],
      });

      expect(result.automaticActions).toContain('Labels: enhancement, ui');
      expect(result.automaticActions).toContain('Assignees: developer1, developer2');
    });
  });

  describe('epic linking', () => {
    it('should link issue to epic when epicNumber provided', async () => {
      const mockIssue = {
        data: {
          number: 126,
          title: 'Sub Issue',
          html_url: 'https://github.com/testuser/testrepo/issues/126',
          state: 'open',
        },
      };

      mockCreate.mockResolvedValue(mockIssue);
      mockGraphql
        .mockResolvedValueOnce({
          repository: { issue: { id: 'epic_node_id', title: 'Epic Issue' } },
        })
        .mockResolvedValueOnce({
          repository: { issue: { id: 'issue_node_id', title: 'Sub Issue' } },
        })
        .mockResolvedValueOnce({
          addSubIssue: {
            issue: { number: 100, title: 'Epic Issue' },
            subIssue: { number: 126, title: 'Sub Issue' },
          },
        })
        .mockResolvedValueOnce({
          repository: { issue: { id: 'issue_node_id', title: 'Sub Issue' } },
        })
        .mockResolvedValueOnce({
          addProjectV2ItemById: { item: { id: 'item_126' } },
        })
        .mockResolvedValueOnce({
          updateProjectV2ItemFieldValue: {
            projectV2Item: { id: 'item_126' },
          },
        });

      const result = await workflowCreateIssue({
        title: 'Sub Issue',
        body: 'Part of epic',
        epicNumber: 100,
      });

      expect(result.requestedData.linkedToEpic).toBe(true);
      expect(result.requestedData.epicNumber).toBe(100);
      expect(result.automaticActions).toContain('Linking issue #126 to epic #100');
      expect(result.automaticActions).toContain('Successfully linked issue #126 to epic #100');
      expect(result.suggestedActions).toContain('View epic: https://github.com/testuser/testrepo/issues/100');
    });

    it('should handle epic linking failure gracefully', async () => {
      const mockIssue = {
        data: {
          number: 127,
          title: 'Sub Issue',
          html_url: 'https://github.com/testuser/testrepo/issues/127',
          state: 'open',
        },
      };

      mockCreate.mockResolvedValue(mockIssue);
      mockGraphql
        .mockRejectedValueOnce(new Error('Epic not found'))
        .mockResolvedValueOnce({
          repository: { issue: { id: 'issue_node_id', title: 'Sub Issue' } },
        })
        .mockResolvedValueOnce({
          addProjectV2ItemById: { item: { id: 'item_127' } },
        })
        .mockResolvedValueOnce({
          updateProjectV2ItemFieldValue: {
            projectV2Item: { id: 'item_127' },
          },
        });

      const result = await workflowCreateIssue({
        title: 'Sub Issue',
        body: 'Part of epic',
        epicNumber: 999,
      });

      expect(result.requestedData.linkedToEpic).toBe(false);
      expect(result.issuesFound).toContain('Could not link to epic #999: Epic not found');
      expect(result.automaticActions).toContain('Warning: Failed to link to epic #999');
    });
  });

  describe('error handling', () => {
    it('should handle GitHub API errors', async () => {
      mockCreate.mockRejectedValue(new Error('GitHub API error'));

      const result = await workflowCreateIssue({
        title: 'Error Issue',
        body: 'Will fail',
      });

      expect(result.requestedData.error).toBe('GitHub API error');
      expect(result.issuesFound).toContain('Error: GitHub API error');
      expect(result.suggestedActions).toContain('Check the error message and try again');
    });
  });
});