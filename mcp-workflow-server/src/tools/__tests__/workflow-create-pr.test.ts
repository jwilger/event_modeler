import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { workflowCreatePR } from '../workflow-create-pr.js';
import { execSync } from 'child_process';
import { Octokit } from '@octokit/rest';
import * as config from '../../config.js';

// Mock modules
vi.mock('child_process');
vi.mock('@octokit/rest');
vi.mock('../../config.js');

const mockExecSync = execSync as jest.MockedFunction<typeof execSync>;
const mockOctokit = Octokit as jest.MockedClass<typeof Octokit>;
const mockGetProjectConfig = vi.mocked(config.getProjectConfig);

interface MockOctokitInstance {
  pulls: {
    create: ReturnType<typeof vi.fn>;
    list: ReturnType<typeof vi.fn>;
  };
  issues: {
    get: ReturnType<typeof vi.fn>;
    update: ReturnType<typeof vi.fn>;
    addAssignees: ReturnType<typeof vi.fn>;
  };
  graphql: ReturnType<typeof vi.fn>;
}

describe('workflowCreatePR', () => {
  let mockOctokitInstance: MockOctokitInstance;

  beforeEach(() => {
    // Reset all mocks
    vi.clearAllMocks();

    // Setup default mocks
    mockOctokitInstance = {
      pulls: {
        create: vi.fn(),
        list: vi.fn()
      },
      issues: {
        get: vi.fn(),
        update: vi.fn(),
        addAssignees: vi.fn()
      },
      graphql: vi.fn()
    };

    mockOctokit.mockImplementation(() => mockOctokitInstance);

    // Default config mock - complete configuration
    mockGetProjectConfig.mockReturnValue({
      config: {
        github: {
          projectNumber: 1,
          projectId: 'TEST_PROJECT_ID',
          statusFieldId: 'TEST_FIELD_ID',
          statusOptions: {
            todo: 'TODO_ID',
            inProgress: 'IN_PROGRESS_ID',
            done: 'DONE_ID'
          }
        }
      },
      isComplete: true
    });

    // Default execSync mocks
    mockExecSync.mockImplementation((cmd: string) => {
      if (cmd === 'git branch --show-current') return 'feature/test-branch-94';
      if (cmd === 'git status --porcelain') return '';
      if (cmd === 'git config --get remote.origin.url') return 'git@github.com:owner/repo.git';
      if (cmd === 'gh auth token') return 'test-token';
      if (cmd.startsWith('git rev-parse origin/')) throw new Error('Branch not on remote');
      if (cmd.startsWith('git push')) return '';
      if (cmd.startsWith('git log')) return 'hash1\x00Test commit\x00\x00';
      if (cmd.startsWith('git diff')) return ' file1.ts | 10 ++++\n file2.ts | 5 ---\n';
      if (cmd.includes('symbolic-ref')) return 'main';
      return '';
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('PR creation', () => {
    it('should create a PR and auto-assign to creator', async () => {
      // Mock API responses
      mockOctokitInstance.pulls.list.mockResolvedValue({ data: [] });
      mockOctokitInstance.pulls.create.mockResolvedValue({
        data: {
          number: 123,
          html_url: 'https://github.com/owner/repo/pull/123',
          title: 'Test PR',
          draft: false,
          node_id: 'PR_NODE_123'
        }
      });
      mockOctokitInstance.issues.get.mockResolvedValue({
        data: {
          number: 94,
          title: 'Test Issue',
          body: 'Test issue body',
          labels: []
        }
      });
      
      // Mock getCurrentUser call
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'gh api user --jq .login') return 'testuser';
        if (cmd === 'git branch --show-current') return 'feature/test-branch-94';
        if (cmd === 'git status --porcelain') return '';
        if (cmd === 'git config --get remote.origin.url') return 'git@github.com:owner/repo.git';
        if (cmd === 'gh auth token') return 'test-token';
        if (cmd.startsWith('git rev-parse origin/')) throw new Error('Branch not on remote');
        if (cmd.startsWith('git push')) return '';
        if (cmd.startsWith('git log')) return 'hash1\x00Test commit\x00\x00';
        if (cmd.startsWith('git diff')) return ' file1.ts | 10 ++++\n file2.ts | 5 ---\n';
        if (cmd.includes('symbolic-ref')) return 'main';
        return '';
      });

      // Mock GraphQL mutations
      mockOctokitInstance.graphql
        .mockResolvedValueOnce({
          addProjectV2ItemById: {
            item: { id: 'ITEM_123' }
          }
        })
        .mockResolvedValueOnce({
          updateProjectV2ItemFieldValue: {
            projectV2Item: { id: 'ITEM_123' }
          }
        });

      const result = await workflowCreatePR();

      // Verify PR was created
      expect(mockOctokitInstance.pulls.create).toHaveBeenCalledWith({
        owner: 'owner',
        repo: 'repo',
        title: 'Test Issue',
        body: expect.stringContaining('This PR implements Test Issue'),
        head: 'feature/test-branch-94',
        base: 'main',
        draft: false
      });

      // Verify PR was auto-assigned
      expect(mockOctokitInstance.issues.addAssignees).toHaveBeenCalledWith({
        owner: 'owner',
        repo: 'repo',
        issue_number: 123,
        assignees: ['testuser']
      });

      // Verify PR was added to project board
      expect(mockOctokitInstance.graphql).toHaveBeenCalledWith(
        expect.stringContaining('addProjectV2ItemById'),
        {
          projectId: 'TEST_PROJECT_ID',
          contentId: 'PR_NODE_123'
        }
      );

      // Verify status was updated
      expect(mockOctokitInstance.graphql).toHaveBeenCalledWith(
        expect.stringContaining('updateProjectV2ItemFieldValue'),
        {
          projectId: 'TEST_PROJECT_ID',
          itemId: 'ITEM_123',
          fieldId: 'TEST_FIELD_ID',
          value: { singleSelectOptionId: 'IN_PROGRESS_ID' }
        }
      );

      // Check response
      expect(result.requestedData.pr).toEqual({
        number: 123,
        url: 'https://github.com/owner/repo/pull/123',
        title: 'Test PR',
        draft: false
      });
      expect(result.automaticActions).toContain('Assigned PR to @testuser');
      expect(result.automaticActions).toContain('Added PR to project board');
      expect(result.automaticActions).toContain('Set PR status to "In Progress" on project board');
    });

    it('should handle failures in auto-assignment gracefully', async () => {
      // Setup PR creation success
      mockOctokitInstance.pulls.list.mockResolvedValue({ data: [] });
      mockOctokitInstance.pulls.create.mockResolvedValue({
        data: {
          number: 123,
          html_url: 'https://github.com/owner/repo/pull/123',
          title: 'Test PR',
          draft: false,
          node_id: 'PR_NODE_123'
        }
      });

      // Make auto-assignment fail
      mockOctokitInstance.issues.addAssignees.mockRejectedValue(new Error('Assignment failed'));

      const result = await workflowCreatePR();

      // PR should still be created
      expect(result.requestedData.pr).toBeDefined();
      expect(result.requestedData.pr?.number).toBe(123);
      
      // Should report assignment failure
      expect(result.automaticActions).toContain('Could not auto-assign PR: Assignment failed');
    });

    it('should handle incomplete project configuration', async () => {
      // Mock incomplete config
      mockGetProjectConfig.mockReturnValue({
        config: {
          github: {
            projectNumber: 1
            // Missing other required fields
          }
        },
        isComplete: false
      });

      // Mock getCurrentUser call
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'gh api user --jq .login') return 'testuser';
        if (cmd === 'git branch --show-current') return 'feature/test-branch-94';
        if (cmd === 'git status --porcelain') return '';
        if (cmd === 'git config --get remote.origin.url') return 'git@github.com:owner/repo.git';
        if (cmd === 'gh auth token') return 'test-token';
        if (cmd.startsWith('git rev-parse origin/')) throw new Error('Branch not on remote');
        if (cmd.startsWith('git push')) return '';
        if (cmd.startsWith('git log')) return 'hash1\x00Test commit\x00\x00';
        if (cmd.startsWith('git diff')) return ' file1.ts | 10 ++++\n file2.ts | 5 ---\n';
        if (cmd.includes('symbolic-ref')) return 'main';
        return '';
      });

      mockOctokitInstance.pulls.list.mockResolvedValue({ data: [] });
      mockOctokitInstance.pulls.create.mockResolvedValue({
        data: {
          number: 123,
          html_url: 'https://github.com/owner/repo/pull/123',
          title: 'Test PR',
          draft: false,
          node_id: 'PR_NODE_123'
        }
      });

      const result = await workflowCreatePR();

      // PR should be created
      expect(result.requestedData.pr).toBeDefined();
      
      // Should skip project board update
      expect(result.automaticActions).toContain('Project configuration incomplete - skipping project board update');
      expect(mockOctokitInstance.graphql).not.toHaveBeenCalled();
    });

    it('should handle project board addition failures gracefully', async () => {
      mockOctokitInstance.pulls.list.mockResolvedValue({ data: [] });
      mockOctokitInstance.pulls.create.mockResolvedValue({
        data: {
          number: 123,
          html_url: 'https://github.com/owner/repo/pull/123',
          title: 'Test PR',
          draft: false,
          node_id: 'PR_NODE_123'
        }
      });

      // Make project board addition fail
      mockOctokitInstance.graphql.mockRejectedValue(new Error('GraphQL error'));

      const result = await workflowCreatePR();

      // PR should still be created
      expect(result.requestedData.pr).toBeDefined();
      
      // Should report project board failure
      expect(result.automaticActions).toContain('Could not add PR to project board: GraphQL error');
    });

    it('should apply labels from related issue', async () => {
      mockOctokitInstance.pulls.list.mockResolvedValue({ data: [] });
      mockOctokitInstance.pulls.create.mockResolvedValue({
        data: {
          number: 123,
          html_url: 'https://github.com/owner/repo/pull/123',
          title: 'Test PR',
          draft: false,
          node_id: 'PR_NODE_123'
        }
      });
      mockOctokitInstance.issues.get.mockResolvedValue({
        data: {
          number: 94,
          title: 'Test Issue',
          body: 'Test issue body',
          labels: [
            { name: 'bug', color: 'red' },
            { name: 'enhancement', color: 'blue' }
          ]
        }
      });

      const result = await workflowCreatePR();

      // Verify labels were applied
      expect(mockOctokitInstance.issues.update).toHaveBeenCalledWith({
        owner: 'owner',
        repo: 'repo',
        issue_number: 123,
        labels: ['bug', 'enhancement']
      });
      
      expect(result.automaticActions).toContain('Applied 2 labels from issue');
    });
  });

  describe('error handling', () => {
    it('should fail if on main branch', async () => {
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'main';
        return '';
      });

      const result = await workflowCreatePR();

      expect(result.requestedData.error).toContain('Cannot create PR from default branch');
      expect(result.issuesFound).toContain('Error: Cannot create PR from default branch. Please create a feature branch first.');
    });

    it('should fail if there are uncommitted changes', async () => {
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'feature/test';
        if (cmd === 'git status --porcelain') return 'M file.txt';
        return '';
      });

      const result = await workflowCreatePR();

      expect(result.requestedData.error).toContain('uncommitted changes');
      expect(result.issuesFound[0]).toContain('You have uncommitted changes');
    });

    it('should return existing PR if one already exists', async () => {
      mockOctokitInstance.pulls.list.mockResolvedValue({
        data: [{
          number: 99,
          html_url: 'https://github.com/owner/repo/pull/99',
          title: 'Existing PR',
          draft: false
        }]
      });

      const result = await workflowCreatePR();

      expect(result.requestedData.pr).toEqual({
        number: 99,
        url: 'https://github.com/owner/repo/pull/99',
        title: 'Existing PR',
        draft: false
      });
      expect(result.issuesFound).toContain('PR already exists for this branch');
    });
  });
});