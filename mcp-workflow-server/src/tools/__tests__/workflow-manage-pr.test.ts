import { describe, it, expect, vi, beforeEach } from 'vitest';
import { workflowManagePR } from '../workflow-manage-pr.js';
import * as githubUtils from '../../utils/github.js';
import * as authUtils from '../../utils/auth.js';
import { Octokit } from '@octokit/rest';

interface WorkflowManagePRResponse {
  prChains?: Array<{
    prNumber: number;
    title: string;
    status: string;
    needsRebase: boolean;
    isMergeable: boolean;
    dependsOn?: number[];
  }>;
  actionsPerformed?: Array<{
    prNumber: number;
    action: string;
    success: boolean;
    details: string;
  }>;
  manualInterventionNeeded?: Array<{
    prNumber: number;
    issue: string;
    suggestion: string;
  }>;
  error?: string;
}

vi.mock('../../utils/github.js');
vi.mock('../../utils/auth.js');
vi.mock('@octokit/rest');

describe('workflowManagePR', () => {
  const mockOctokit = {
    pulls: {
      list: vi.fn(),
      get: vi.fn(),
    },
  };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(Octokit).mockImplementation(() => mockOctokit as unknown as Octokit);
    vi.mocked(authUtils.getGitHubToken).mockReturnValue('test-token');
    vi.mocked(githubUtils.getRepoInfo).mockReturnValue({
      owner: 'testowner',
      repo: 'testrepo',
    });
  });

  it('should analyze PRs and return chain information', async () => {
    const mockPRs = [
      {
        number: 1,
        title: 'Test PR 1',
        body: 'First test PR',
        head: { ref: 'feature/test-1' },
        base: { ref: 'main' },
      },
      {
        number: 2,
        title: 'Test PR 2',
        body: 'Depends on #1',
        head: { ref: 'feature/test-2' },
        base: { ref: 'main' },
      },
    ];

    const mockPRDetails = [
      {
        number: 1,
        mergeable_state: 'clean',
        mergeable: true,
      },
      {
        number: 2,
        mergeable_state: 'behind',
        mergeable: false,
      },
    ];

    mockOctokit.pulls.list.mockResolvedValue({ data: mockPRs });
    mockOctokit.pulls.get
      .mockResolvedValueOnce({ data: mockPRDetails[0] })
      .mockResolvedValueOnce({ data: mockPRDetails[1] });

    const result = await workflowManagePR({
      action: 'analyze',
    });

    const responseData = result.requestedData as WorkflowManagePRResponse;
    expect(responseData.prChains).toHaveLength(2);
    expect(responseData.prChains![0]).toMatchObject({
      prNumber: 1,
      title: 'Test PR 1',
      status: 'ready',
      needsRebase: false,
      isMergeable: true,
    });
    expect(responseData.prChains![1]).toMatchObject({
      prNumber: 2,
      title: 'Test PR 2',
      status: 'needs_rebase',
      needsRebase: true,
      isMergeable: false,
      dependsOn: [1],
    });
    expect(result.automaticActions).toContain('Found 2 open PRs');
    expect(result.automaticActions).toContain('Performed analysis of PR chains and dependencies');
  });

  it('should handle rebase action for specific PR', async () => {
    mockOctokit.pulls.list.mockResolvedValue({ data: [] });

    const result = await workflowManagePR({
      action: 'rebase',
      prNumber: 123,
    });

    const responseData = result.requestedData as WorkflowManagePRResponse;
    expect(responseData.actionsPerformed).toHaveLength(1);
    expect(responseData.actionsPerformed![0]).toMatchObject({
      prNumber: 123,
      action: 'skipped',
      success: false,
      details: 'Rebase implementation pending',
    });
    expect(responseData.manualInterventionNeeded).toHaveLength(1);
    expect(responseData.manualInterventionNeeded![0]).toMatchObject({
      prNumber: 123,
      issue: 'complex_rebase',
      suggestion: 'Manual rebase required - implementation pending',
    });
  });

  it('should require PR number for rebase action', async () => {
    mockOctokit.pulls.list.mockResolvedValue({ data: [] });

    const result = await workflowManagePR({
      action: 'rebase',
    });

    expect(result.issuesFound).toContain('PR number required for rebase action');
  });

  it('should handle update_chains action', async () => {
    mockOctokit.pulls.list.mockResolvedValue({ data: [] });

    const result = await workflowManagePR({
      action: 'update_chains',
      targetBranch: 'main',
    });

    const responseData = result.requestedData as WorkflowManagePRResponse;
    expect(responseData.actionsPerformed).toHaveLength(0);
    expect(responseData.manualInterventionNeeded).toHaveLength(0);
    expect(result.automaticActions).toContain('Found 0 open PRs');
  });

  it('should handle unknown action', async () => {
    mockOctokit.pulls.list.mockResolvedValue({ data: [] });

    const result = await workflowManagePR({
      // @ts-expect-error - Testing invalid action
      action: 'invalid_action',
    });

    const responseData = result.requestedData as WorkflowManagePRResponse;
    expect(responseData.error).toBe('Unknown action: invalid_action');
    expect(result.issuesFound).toContain('Error: Unknown action: invalid_action');
  });

  it('should extract dependencies from PR body', async () => {
    const mockPRs = [
      {
        number: 3,
        title: 'Complex PR',
        body: 'This PR depends on #1 and closes #2',
        head: { ref: 'feature/complex' },
        base: { ref: 'main' },
      },
    ];

    const mockPRDetail = {
      number: 3,
      mergeable_state: 'clean',
      mergeable: true,
    };

    mockOctokit.pulls.list.mockResolvedValue({ data: mockPRs });
    mockOctokit.pulls.get.mockResolvedValue({ data: mockPRDetail });

    const result = await workflowManagePR({
      action: 'analyze',
    });

    const responseData = result.requestedData as WorkflowManagePRResponse;
    expect(responseData.prChains![0].dependsOn).toEqual([1, 2]);
  });

  it('should generate appropriate suggestions', async () => {
    const mockPRs = [
      {
        number: 1,
        title: 'Ready PR',
        body: 'Ready to merge',
        head: { ref: 'feature/ready' },
        base: { ref: 'main' },
      },
      {
        number: 2,
        title: 'Needs rebase',
        body: 'Behind main',
        head: { ref: 'feature/behind' },
        base: { ref: 'main' },
      },
    ];

    const mockPRDetails = [
      {
        number: 1,
        mergeable_state: 'clean',
        mergeable: true,
      },
      {
        number: 2,
        mergeable_state: 'behind',
        mergeable: true,
      },
    ];

    mockOctokit.pulls.list.mockResolvedValue({ data: mockPRs });
    mockOctokit.pulls.get
      .mockResolvedValueOnce({ data: mockPRDetails[0] })
      .mockResolvedValueOnce({ data: mockPRDetails[1] });

    const result = await workflowManagePR({
      action: 'analyze',
    });

    expect(result.suggestedActions).toContain('1 PRs need rebasing');
    expect(result.suggestedActions).toContain('1 PRs are ready to merge (no dependencies)');
  });
});