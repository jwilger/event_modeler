import { describe, it, expect, vi, beforeEach } from 'vitest';
import { workflowStatusTool } from '../workflow-status.js';
import { getGitStatus, isCurrentBranchStale } from '../../utils/git.js';
import { getAllPRs } from '../../utils/github.js';

vi.mock('../../utils/git.js');
vi.mock('../../utils/github.js');
vi.mock('../../state/store.js', () => {
  const mockStore = {
    updateLastStatusCheck: vi.fn(),
    updatePRStatus: vi.fn(),
    getPRStatus: vi.fn(),
    recordBranchCreation: vi.fn(),
    getBranchCreationDate: vi.fn(),
    clearPRStatus: vi.fn(),
  };
  
  return {
    StateStore: vi.fn(() => mockStore),
  };
});

describe('Workflow Status Tool', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should return clean status when everything is good', async () => {
    vi.mocked(getGitStatus).mockResolvedValue({
      currentBranch: 'feature/test',
      isClean: true,
      uncommittedFiles: [],
      untrackedFiles: [],
      aheadBehind: { ahead: 1, behind: 0 },
      lastCommit: {
        hash: 'abc123',
        message: 'Test commit',
        date: '2024-01-01',
      },
    });

    vi.mocked(isCurrentBranchStale).mockResolvedValue(false);
    vi.mocked(getAllPRs).mockResolvedValue([]);

    const result = await workflowStatusTool();

    expect(result.issuesFound).toHaveLength(0);
    expect(result.automaticActions).toContain('Checked git status and branch information');
    expect(result.suggestedActions).toContain('[NEXT] Create a PR for your current branch');
  });

  it('should detect uncommitted changes', async () => {
    vi.mocked(getGitStatus).mockResolvedValue({
      currentBranch: 'feature/test',
      isClean: false,
      uncommittedFiles: ['file1.ts', 'file2.ts'],
      untrackedFiles: ['new.ts'],
      aheadBehind: { ahead: 0, behind: 0 },
      lastCommit: {
        hash: 'abc123',
        message: 'Test commit',
        date: '2024-01-01',
      },
    });

    vi.mocked(isCurrentBranchStale).mockResolvedValue(false);
    vi.mocked(getAllPRs).mockResolvedValue([]);

    const result = await workflowStatusTool();

    expect(result.issuesFound).toContain('Working directory has uncommitted changes: 2 files');
    expect(result.issuesFound).toContain('1 untracked files found');
    expect(result.suggestedActions).toContain('Commit or stash changes before proceeding');
  });

  it('should detect failing CI and PRs needing rebase', async () => {
    vi.mocked(getGitStatus).mockResolvedValue({
      currentBranch: 'main',
      isClean: true,
      uncommittedFiles: [],
      untrackedFiles: [],
      aheadBehind: { ahead: 0, behind: 0 },
      lastCommit: {
        hash: 'abc123',
        message: 'Test commit',
        date: '2024-01-01',
      },
    });

    vi.mocked(isCurrentBranchStale).mockResolvedValue(false);
    vi.mocked(getAllPRs).mockResolvedValue([
      {
        number: 1,
        title: 'Test PR 1',
        branch: 'feature/test1',
        baseRef: 'main',
        state: 'open',
        isDraft: false,
        url: 'https://github.com/test/repo/pull/1',
        checks: { total: 3, passed: 1, failed: 2, pending: 0 },
        hasUnresolvedReviews: false,
        needsRebase: false,
        isMergeable: true,
      },
      {
        number: 2,
        title: 'Test PR 2',
        branch: 'feature/test2',
        baseRef: 'main',
        state: 'open',
        isDraft: false,
        url: 'https://github.com/test/repo/pull/2',
        checks: { total: 3, passed: 3, failed: 0, pending: 0 },
        hasUnresolvedReviews: true,
        needsRebase: true,
        isMergeable: false,
      },
    ]);

    const result = await workflowStatusTool();

    expect(result.issuesFound).toContain('🔴 URGENT: 1 PRs have failing CI checks');
    expect(result.issuesFound).toContain('🟡 HIGH: 1 PRs need rebase after base branch merge');
    expect(result.issuesFound).toContain('🟡 HIGH: 1 PRs have unresolved review comments or conversations');
    expect(result.suggestedActions).toContain('[URGENT] Fix CI failures in PR #1 (feature/test1)');
    expect(result.suggestedActions).toContain('[HIGH] Rebase PR #2 (feature/test2) onto main');
  });

  it('should handle GitHub API errors gracefully', async () => {
    vi.mocked(getGitStatus).mockResolvedValue({
      currentBranch: 'feature/test',
      isClean: true,
      uncommittedFiles: [],
      untrackedFiles: [],
      aheadBehind: { ahead: 0, behind: 0 },
      lastCommit: {
        hash: 'abc123',
        message: 'Test commit',
        date: '2024-01-01',
      },
    });

    vi.mocked(isCurrentBranchStale).mockResolvedValue(false);
    vi.mocked(getAllPRs).mockRejectedValue(new Error('GitHub API error'));

    const result = await workflowStatusTool();

    expect(result.issuesFound).toContain('Unable to retrieve PR status from GitHub');
    expect(result.suggestedActions).toContain('Ensure gh CLI is authenticated: gh auth status');
    expect(result.allPRStatus).toHaveLength(0);
  });

  it('should detect stale branches', async () => {
    vi.mocked(getGitStatus).mockResolvedValue({
      currentBranch: 'feature/old',
      isClean: true,
      uncommittedFiles: [],
      untrackedFiles: [],
      aheadBehind: { ahead: 5, behind: 20 },
      lastCommit: {
        hash: 'abc123',
        message: 'Old commit',
        date: '2024-01-01',
      },
    });

    vi.mocked(isCurrentBranchStale).mockResolvedValue(true);
    vi.mocked(getAllPRs).mockResolvedValue([]);

    const result = await workflowStatusTool();

    expect(result.issuesFound).toContain("Branch 'feature/old' may be stale (created before recent main merges)");
    expect(result.suggestedActions).toContain('Consider rebasing on latest main or creating a fresh branch');
  });
});