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

  it('should prompt for review when first selecting an issue', async () => {
    const input = {
      decisionId: 'epic-68-next-issue-123456',
      selectedChoice: 123,
      reasoning: 'This issue looks important',
    };

    const result = await workflowDecide(input);

    // Should return a review prompt decision
    expect(result.requestedData.nextSteps).toHaveLength(1);
    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'requires_llm_decision',
      decisionType: 'confirm_issue_start',
      issueNumber: 123,
      title: 'Test Issue',
      choices: [
        {
          id: 'review',
          title: 'Review issue first',
          description: 'Open the issue in browser to review or add details before starting',
        },
        {
          id: 'start',
          title: 'Start work immediately',
          description: 'Proceed with assignment and branch creation',
        },
      ],
    });

    // Check the decision context includes issue details
    expect(result.requestedData.nextSteps[0].decisionContext).toMatchObject({
      prompt: 'Would you like to review this issue before starting work, or proceed immediately?',
      issueDetails: {
        number: 123,
        title: 'Test Issue',
        body: 'This is a test issue\n\nPart of #68',
        url: 'https://github.com/testuser/testrepo/issues/123',
        epicNumber: 68,
      },
    });
  });

  it('should handle review choice', async () => {
    const input = {
      decisionId: 'confirm-start-issue-123-epic-68-123456',
      selectedChoice: 'review',
    };

    const result = await workflowDecide(input);

    // Should return review_issue action
    expect(result.requestedData.nextSteps).toHaveLength(1);
    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'review_issue',
      issueNumber: 123,
      issueUrl: 'https://github.com/testuser/testrepo/issues/123',
      suggestion: 'Please review the issue and add any necessary details. When ready, run workflow_next again to continue.',
    });

    expect(result.suggestedActions).toContain('Open issue #123 in your browser: https://github.com/testuser/testrepo/issues/123');
    expect(result.suggestedActions).toContain('Add any clarifications or implementation notes to the issue');
    expect(result.suggestedActions).toContain('When ready, run workflow_next to continue with this issue');
  });

  it('should handle start work choice', async () => {
    const { Octokit } = await import('@octokit/rest');
    const mockOctokit = vi.mocked(Octokit);
    
    const input = {
      decisionId: 'confirm-start-issue-123-epic-68-123456',
      selectedChoice: 'start',
    };

    const result = await workflowDecide(input);

    // Should proceed with assignment
    expect(result.requestedData.nextSteps).toHaveLength(1);
    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'assign_and_start',
      issueNumber: 123,
      title: 'Test Issue',
      epicNumber: 68,
    });

    // Verify issue was assigned
    const octokitInstance = mockOctokit.mock.results[0].value;
    expect(octokitInstance.issues.addAssignees).toHaveBeenCalledWith({
      owner: 'testuser',
      repo: 'testrepo',
      issue_number: 123,
      assignees: ['testuser'],
    });

    expect(result.automaticActions).toContain('User confirmed to start work immediately');
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
      config: {},
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