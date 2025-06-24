import { describe, it, expect, vi, beforeEach } from 'vitest';
import { workflowDecide } from '../workflow-decide.js';
import { getProjectConfig } from '../../config.js';

// Mock child_process
vi.mock('child_process', () => ({
  execSync: vi.fn((cmd: string) => {
    if (cmd === 'gh auth token') return 'test-token';
    if (cmd === 'gh api user --jq .login') return 'testuser';
    if (cmd === 'git branch --show-current') return 'main';
    if (cmd === 'git status --porcelain') return '';
    if (cmd.startsWith('git remote get-url')) return 'https://github.com/testuser/testrepo.git';
    return '';
  }),
}));

// Mock @octokit/rest
vi.mock('@octokit/rest', () => ({
  Octokit: vi.fn(() => ({
    issues: {
      get: vi.fn().mockResolvedValue({
        data: {
          number: 123,
          title: 'Test Issue',
          body: 'This is a test issue\n\nPart of #68',
          assignees: [],
        },
      }) as any,
      addAssignees: vi.fn().mockResolvedValue({}) as any,
    },
    graphql: vi.fn().mockResolvedValue({
      user: {
        projectV2: {
          items: {
            nodes: [{
              id: 'PVTI_test',
              content: { number: 123 },
            }],
          },
        },
      },
    }) as any,
  })),
}));

// Mock config
vi.mock('../../config.js', () => ({
  getProjectConfig: vi.fn(() => ({
    config: {
      github: {
        projectNumber: 9,
        projectId: 'PVT_test',
        statusFieldId: 'PVTSSF_test',
        statusOptions: {
          todo: 'PVTSSO_todo',
          inProgress: 'PVTSSO_inprogress',
          done: 'PVTSSO_done',
        },
      },
    },
    isComplete: true,
  })),
}));

// Mock github utils
vi.mock('../../utils/github.js', () => ({
  getRepoInfo: vi.fn(() => ({
    owner: 'testuser',
    repo: 'testrepo',
  })),
}));

describe('workflowDecide', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should immediately assign and start work when selecting an issue', async () => {
    const { Octokit } = await import('@octokit/rest');
    const mockOctokit = vi.mocked(Octokit);
    
    const input = {
      decisionId: 'epic-68-next-issue-123456',
      selectedChoice: 123,
      reasoning: 'This issue looks important',
    };

    const result = await workflowDecide(input);

    // Should return assign_and_start action
    expect(result.requestedData.nextSteps).toHaveLength(1);
    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'assign_and_start',
      issueNumber: 123,
      title: 'Test Issue',
      epicNumber: 68,
      issueUrl: 'https://github.com/testuser/testrepo/issues/123',
      issueBody: 'This is a test issue\n\nPart of #68',
    });

    // Verify issue was assigned
    const octokitInstance = mockOctokit.mock.results[0].value;
    expect(octokitInstance.issues.addAssignees).toHaveBeenCalledWith({
      owner: 'testuser',
      repo: 'testrepo',
      issue_number: 123,
      assignees: ['testuser'],
    });

    // Check the context includes issue details
    expect(result.requestedData.context.issueDetails).toMatchObject({
      number: 123,
      title: 'Test Issue',
      body: 'This is a test issue\n\nPart of #68',
      url: 'https://github.com/testuser/testrepo/issues/123',
      epicNumber: 68,
    });
  });

  it('should handle when issue is already assigned to user', async () => {
    const { Octokit } = await import('@octokit/rest');
    const mockOctokit = vi.mocked(Octokit);
    
    // Mock issue already assigned to user
    mockOctokit.mockImplementationOnce(() => ({
      issues: {
        get: vi.fn().mockResolvedValue({
          data: {
            number: 123,
            title: 'Test Issue',
            body: 'This is a test issue\n\nPart of #68',
            assignees: [{ login: 'testuser' }],
          },
        }),
        addAssignees: vi.fn().mockResolvedValue({}),
      },
      graphql: vi.fn().mockResolvedValue({
        user: {
          projectV2: {
            items: {
              nodes: [{
                id: 'PVTI_test',
                content: { number: 123 },
              }],
            },
          },
        },
      }),
    }) as any);
    
    const input = {
      decisionId: 'epic-68-next-issue-123456',
      selectedChoice: 123,
    };

    const result = await workflowDecide(input);

    // Should still proceed but not call addAssignees
    expect(result.requestedData.nextSteps).toHaveLength(1);
    expect(result.requestedData.nextSteps[0].action).toBe('assign_and_start');

    // Verify issue was NOT reassigned
    const octokitInstance = mockOctokit.mock.results[0].value;
    expect(octokitInstance.issues.addAssignees).not.toHaveBeenCalled();
  });

  it('should handle invalid decision ID format', async () => {
    const input = {
      decisionId: 'invalid-format',
      selectedChoice: 123,
    };

    const result = await workflowDecide(input);

    expect(result.issuesFound).toContain('Error: Invalid decision ID format');
  });

  it('should handle missing configuration', async () => {
    vi.mocked(getProjectConfig).mockReturnValueOnce({
      config: {
        github: {
          projectNumber: undefined,
          projectId: undefined,
          statusFieldId: undefined,
          statusOptions: {
            todo: undefined,
            inProgress: undefined,
            done: undefined,
          },
        },
      },
      isComplete: false,
    });

    const input = {
      decisionId: 'epic-68-next-issue-123456',
      selectedChoice: 123,
    };

    const result = await workflowDecide(input);

    expect(result.issuesFound).toContain('Error: Configuration is incomplete. Please run workflow_configure first.');
  });
});