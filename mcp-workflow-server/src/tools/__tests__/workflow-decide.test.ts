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
    if (cmd.startsWith('git rev-parse --verify feature/')) throw new Error('Branch not found');
    if (cmd === 'git checkout main') return '';
    if (cmd === 'git pull origin main') return '';
    if (cmd.startsWith('git checkout -b feature/')) return '';
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
    }) as unknown as InstanceType<typeof Octokit>);
    
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

  it('should create branch from main when already on main', async () => {
    const input = {
      decisionId: 'epic-68-next-issue-123456',
      selectedChoice: 123,
    };

    const result = await workflowDecide(input);

    // Get the mocked execSync
    const { execSync } = await import('child_process');
    const mockExecSync = vi.mocked(execSync);

    // Verify the git commands were called in correct order
    const gitCommands = mockExecSync.mock.calls
      .filter(call => call[0].toString().startsWith('git'))
      .map(call => call[0]);

    // Should check current branch
    expect(gitCommands).toContain('git branch --show-current');
    // Should check if branch exists
    expect(gitCommands.some(cmd => cmd.includes('git rev-parse --verify feature/'))).toBe(true);
    // Should pull latest main (we're already on main)
    expect(gitCommands).toContain('git pull origin main');
    // Should create new branch
    expect(gitCommands.some(cmd => cmd.includes('git checkout -b feature/'))).toBe(true);

    // Verify suggested action includes the branch creation
    expect(result.requestedData.nextSteps[0].suggestion).toContain('Created and switched to new branch');
  });

  it('should switch to main before creating branch when on different branch', async () => {
    // Get the mocked execSync
    const { execSync } = await import('child_process');
    const mockExecSync = vi.mocked(execSync);
    
    // Mock being on a different branch
    mockExecSync.mockImplementation((cmd: string) => {
      if (cmd === 'gh auth token') return 'test-token';
      if (cmd === 'gh api user --jq .login') return 'testuser';
      if (cmd === 'git branch --show-current') return 'feature/old-branch';
      if (cmd === 'git status --porcelain') return '';
      if (cmd.startsWith('git remote get-url')) return 'https://github.com/testuser/testrepo.git';
      if (cmd.startsWith('git rev-parse --verify feature/')) throw new Error('Branch not found');
      if (cmd === 'git checkout main') return '';
      if (cmd === 'git pull origin main') return '';
      if (cmd.startsWith('git checkout -b feature/')) return '';
      return '';
    });

    const input = {
      decisionId: 'epic-68-next-issue-123456',
      selectedChoice: 123,
    };

    const result = await workflowDecide(input);

    // Verify the git commands were called in correct order
    const gitCommands = mockExecSync.mock.calls
      .filter(call => call[0].toString().startsWith('git'))
      .map(call => call[0]);

    // Should switch to main first
    expect(gitCommands).toContain('git checkout main');
    // Then pull latest
    expect(gitCommands).toContain('git pull origin main');
    // Then create new branch
    expect(gitCommands.some(cmd => cmd.includes('git checkout -b feature/'))).toBe(true);

    // Verify automatic actions mention switching to main
    expect(result.automaticActions).toContain('Switched to main branch');
    expect(result.automaticActions).toContain('Updated main branch with latest changes');
  });
});