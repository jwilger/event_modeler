import { describe, it, expect, beforeEach, vi } from 'vitest';
import { execSync } from 'child_process';
import { Octokit } from '@octokit/rest';
import { gitBranch } from '../git-branch.js';

// Mock modules
vi.mock('child_process', () => ({
  execSync: vi.fn(),
}));

vi.mock('@octokit/rest');

vi.mock('../../utils/github.js', () => ({
  getRepoInfo: vi.fn(() => ({ owner: 'testowner', repo: 'testrepo' })),
}));

vi.mock('../../utils/auth.js', () => ({
  getGitHubToken: vi.fn(() => 'mock-token'),
}));

describe('gitBranch', () => {
  let mockExecSync: ReturnType<typeof vi.fn>;
  let mockOctokit: { issues: { get: ReturnType<typeof vi.fn> } };

  beforeEach(() => {
    vi.clearAllMocks();
    mockExecSync = vi.mocked(execSync);
    
    mockOctokit = {
      issues: {
        get: vi.fn(),
      },
    };
    
    vi.mocked(Octokit).mockImplementation(() => mockOctokit as unknown as Octokit);
    
    // Default mocks
    mockExecSync.mockImplementation((cmd: string) => {
      if (cmd === 'git branch --show-current') return 'main\n';
      if (cmd === 'git status --porcelain') return '';
      return '';
    });
  });

  describe('checkout action', () => {
    it('should checkout an existing branch', async () => {
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'main\n';
        if (cmd === 'git status --porcelain') return '';
        if (cmd === 'git rev-parse --verify feature/test') return 'abc123\n';
        if (cmd === 'git checkout feature/test') return '';
        if (cmd === 'git rev-parse --abbrev-ref feature/test@{upstream}') return 'origin/feature/test\n';
        if (cmd === 'git pull') return 'Already up to date.\n';
        return '';
      });

      const result = await gitBranch({ action: 'checkout', branch: 'feature/test' });

      expect(result.requestedData.currentBranch).toBe('feature/test');
      expect(result.requestedData.previousBranch).toBe('main');
      expect(result.automaticActions).toContain("Switched to branch 'feature/test'");
      expect(result.automaticActions).toContain('Pulled latest changes from remote');
    });

    it('should prevent checkout with uncommitted changes', async () => {
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'main\n';
        if (cmd === 'git status --porcelain') return 'M file.txt\n';
        return '';
      });

      const result = await gitBranch({ action: 'checkout', branch: 'feature/test' });

      expect(result.issuesFound).toContain('You have uncommitted changes');
      expect(result.suggestedActions).toContain('Or stash them: use git_stash tool with action: "save"');
      expect(result.requestedData.currentBranch).toBeUndefined();
    });

    it('should force checkout when force flag is set', async () => {
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'main\n';
        if (cmd === 'git status --porcelain') return 'M file.txt\n';
        if (cmd === 'git rev-parse --verify feature/test') return 'abc123\n';
        if (cmd === 'git checkout feature/test') return '';
        return '';
      });

      const result = await gitBranch({ action: 'checkout', branch: 'feature/test', force: true });

      expect(result.requestedData.currentBranch).toBe('feature/test');
      expect(result.issuesFound).toHaveLength(0);
    });

    it('should fetch branch from remote if not local', async () => {
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'main\n';
        if (cmd === 'git status --porcelain') return '';
        if (cmd === 'git rev-parse --verify feature/remote') throw new Error('Not found');
        if (cmd === 'git fetch origin feature/remote:feature/remote') return '';
        if (cmd === 'git checkout feature/remote') return '';
        return '';
      });

      const result = await gitBranch({ action: 'checkout', branch: 'feature/remote' });

      expect(result.automaticActions).toContain('Fetched branch feature/remote from remote');
      expect(result.requestedData.currentBranch).toBe('feature/remote');
    });
  });

  describe('create action', () => {
    it('should create branch from issue number', async () => {
      mockOctokit.issues.get.mockResolvedValue({
        data: {
          title: 'Add new feature: User authentication',
          html_url: 'https://github.com/owner/repo/issues/123',
        },
      });

      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'feature/old\n';
        if (cmd === 'git status --porcelain') return '';
        if (cmd === 'gh auth token') return 'mock-token\n';
        if (cmd === 'git rev-parse --verify feature/add-new-feature-user-authentication-123') {
          throw new Error('Not found');
        }
        if (cmd === 'git rev-parse --verify main') return 'abc123\n';
        if (cmd === 'git checkout main') return '';
        if (cmd === 'git pull') return '';
        if (cmd === 'git checkout -b feature/add-new-feature-user-authentication-123') return '';
        return '';
      });

      const result = await gitBranch({ action: 'create', issueNumber: 123 });

      expect(result.requestedData.createdBranch).toBe('feature/add-new-feature-user-authentication-123');
      expect(result.requestedData.issueDetails).toEqual({
        number: 123,
        title: 'Add new feature: User authentication',
        url: 'https://github.com/owner/repo/issues/123',
      });
      expect(result.automaticActions).toContain('Updated base branch');
    });

    it('should handle existing branch', async () => {
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'main\n';
        if (cmd === 'git status --porcelain') return '';
        if (cmd === 'git rev-parse --verify my-branch') return 'abc123\n';
        return '';
      });

      const result = await gitBranch({ action: 'create', branch: 'my-branch' });

      expect(result.issuesFound).toContain("Branch 'my-branch' already exists");
      expect(result.suggestedActions).toContain('Checkout existing branch: git checkout my-branch');
    });

    it('should truncate long branch names', async () => {
      mockOctokit.issues.get.mockResolvedValue({
        data: {
          title: 'This is a very long issue title that should be truncated to fit within git branch name limits for safety',
          html_url: 'https://github.com/owner/repo/issues/456',
        },
      });

      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'main\n';
        if (cmd === 'git status --porcelain') return '';
        if (cmd === 'gh auth token') return 'mock-token\n';
        if (cmd.includes('git rev-parse --verify feature/')) throw new Error('Not found');
        if (cmd === 'git rev-parse --verify main') return 'abc123\n';
        if (cmd === 'git checkout main') return '';
        if (cmd === 'git pull') return '';
        if (cmd.startsWith('git checkout -b feature/')) return '';
        return '';
      });

      const result = await gitBranch({ action: 'create', issueNumber: 456 });

      expect(result.requestedData.createdBranch).toMatch(/^feature\/.{1,50}-456$/);
      expect(result.requestedData.createdBranch!.length).toBeLessThanOrEqual(60);
    });
  });

  describe('pull action', () => {
    it('should pull latest changes', async () => {
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'feature/test\n';
        if (cmd === 'git pull') return 'Updating abc123..def456\n';
        return '';
      });

      const result = await gitBranch({ action: 'pull' });

      expect(result.automaticActions).toContain('Pulled latest changes');
      expect(result.requestedData.currentBranch).toBe('feature/test');
    });

    it('should handle already up to date', async () => {
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'main\n';
        if (cmd === 'git pull') return 'Already up to date.\n';
        return '';
      });

      const result = await gitBranch({ action: 'pull' });

      expect(result.automaticActions).toContain('Branch is already up to date');
    });
  });

  describe('push action', () => {
    it('should push with existing upstream', async () => {
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'feature/test\n';
        if (cmd === 'git rev-parse --abbrev-ref feature/test@{upstream}') return 'origin/feature/test\n';
        if (cmd === 'git push') return '';
        return '';
      });

      const result = await gitBranch({ action: 'push' });

      expect(result.requestedData.pushedBranch).toBe('feature/test');
      expect(result.automaticActions).toContain("Pushed branch 'feature/test' to remote");
    });

    it('should set upstream when missing', async () => {
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'feature/new\n';
        if (cmd === 'git rev-parse --abbrev-ref feature/new@{upstream}') throw new Error('No upstream');
        if (cmd === 'git push -u origin feature/new') return '';
        return '';
      });

      const result = await gitBranch({ action: 'push' });

      expect(result.automaticActions).toContain('Set upstream branch to origin/feature/new');
      expect(result.requestedData.pushedBranch).toBe('feature/new');
    });
  });

  describe('list action', () => {
    it('should list all branches with status', async () => {
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'develop\n';
        if (cmd === 'git branch -vv') {
          return `  main          abc123 [origin/main] Initial commit
* develop       def456 [origin/develop: ahead 2, behind 1] Work in progress
  feature/test  ghi789 Local branch only`;
        }
        return '';
      });

      const result = await gitBranch({ action: 'list' });

      expect(result.requestedData.branches).toHaveLength(3);
      expect(result.requestedData.branches![0]).toEqual({
        name: 'main',
        current: false,
        remote: true,
        ahead: undefined,
        behind: undefined,
      });
      expect(result.requestedData.branches![1]).toEqual({
        name: 'develop',
        current: true,
        remote: true,
        ahead: 2,
        behind: 1,
      });
      expect(result.requestedData.branches![2]).toEqual({
        name: 'feature/test',
        current: false,
        remote: false,
        ahead: undefined,
        behind: undefined,
      });
    });
  });

  describe('start-work action', () => {
    it('should create new branch for issue', async () => {
      mockOctokit.issues.get.mockResolvedValue({
        data: {
          title: 'Fix bug in login',
          html_url: 'https://github.com/owner/repo/issues/789',
        },
      });

      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'feature/old\n';
        if (cmd === 'git status --porcelain') return '';
        if (cmd === 'gh auth token') return 'mock-token\n';
        if (cmd === 'git rev-parse --verify feature/fix-bug-in-login-789') throw new Error('Not found');
        if (cmd === 'git rev-parse --verify main') return 'abc123\n';
        if (cmd === 'git checkout main') return '';
        if (cmd === 'git pull') return '';
        if (cmd === 'git checkout -b feature/fix-bug-in-login-789') return '';
        return '';
      });

      const result = await gitBranch({ action: 'start-work', issueNumber: 789 });

      expect(result.requestedData.currentBranch).toBe('feature/fix-bug-in-login-789');
      expect(result.requestedData.createdBranch).toBe('feature/fix-bug-in-login-789');
      expect(result.automaticActions).toContain('Created new branch: feature/fix-bug-in-login-789');
    });

    it('should checkout existing branch for issue', async () => {
      mockOctokit.issues.get.mockResolvedValue({
        data: {
          title: 'Fix bug in login',
          html_url: 'https://github.com/owner/repo/issues/789',
        },
      });

      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'main\n';
        if (cmd === 'git status --porcelain') return '';
        if (cmd === 'gh auth token') return 'mock-token\n';
        if (cmd === 'git rev-parse --verify feature/fix-bug-in-login-789') return 'abc123\n';
        if (cmd === 'git checkout feature/fix-bug-in-login-789') return '';
        if (cmd === 'git pull') return '';
        return '';
      });

      const result = await gitBranch({ action: 'start-work', issueNumber: 789 });

      expect(result.requestedData.currentBranch).toBe('feature/fix-bug-in-login-789');
      expect(result.requestedData.createdBranch).toBeUndefined();
      expect(result.automaticActions).toContain('Switched to existing branch: feature/fix-bug-in-login-789');
    });

    it('should block with uncommitted changes', async () => {
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'git branch --show-current') return 'main\n';
        if (cmd === 'git status --porcelain') return 'M file.txt\n';
        return '';
      });

      const result = await gitBranch({ action: 'start-work', issueNumber: 123 });

      expect(result.issuesFound).toContain('You have uncommitted changes');
      expect(result.suggestedActions).toContain('Commit your changes before starting new work: use git_commit tool');
    });
  });

  describe('error handling', () => {
    it('should handle missing branch parameter', async () => {
      const result = await gitBranch({ action: 'checkout' });

      expect(result.issuesFound).toContain('Error: Branch name is required for checkout');
    });

    it('should handle missing issue number for start-work', async () => {
      const result = await gitBranch({ action: 'start-work' });

      expect(result.issuesFound).toContain('Error: Issue number is required for start-work action');
    });

    it('should handle API errors', async () => {
      mockOctokit.issues.get.mockRejectedValue(new Error('API rate limit exceeded'));

      const result = await gitBranch({ action: 'create', issueNumber: 123 });

      expect(result.issuesFound[0]).toContain('Failed to get issue #123');
      expect(result.issuesFound[0]).toContain('API rate limit exceeded');
    });
  });
});