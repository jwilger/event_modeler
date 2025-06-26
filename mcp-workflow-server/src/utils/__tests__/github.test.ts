import { describe, it, expect, vi, beforeEach } from 'vitest';
import { execSync } from 'child_process';
import { Octokit } from '@octokit/rest';

// Mock modules
vi.mock('child_process');
vi.mock('@octokit/rest');

// Mock auth
vi.mock('../auth.js', () => ({
  getGitHubToken: vi.fn(() => 'mock-token'),
}));

// Import will be done per test after mocking

describe('GitHub Utilities', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Repository URL parsing', () => {
    it('should handle repository names with dots', async () => {
      // Mock git remote
      vi.mocked(execSync).mockImplementation((cmd) => {
        if (cmd === 'git config --get remote.origin.url') {
          return 'git@github.com:user/repo.with.dots.git\n';
        }
        return '';
      });

      // Mock Octokit
      const mockOctokit = {
        pulls: {
          list: vi.fn().mockResolvedValue({ data: [] }),
        },
      };
      vi.mocked(Octokit).mockImplementation(() => mockOctokit as unknown as Octokit);

      // This should not throw
      const { getAllPRs } = await import('../github.js');
      expect(async () => await getAllPRs()).not.toThrow();
    });

    it('should handle HTTPS URLs', async () => {
      vi.mocked(execSync).mockImplementation((cmd) => {
        if (cmd === 'git config --get remote.origin.url') {
          return 'https://github.com/owner/repository.git\n';
        }
        return '';
      });

      const mockOctokit = {
        pulls: {
          list: vi.fn().mockResolvedValue({ data: [] }),
        },
      };
      vi.mocked(Octokit).mockImplementation(() => mockOctokit as unknown as Octokit);

      const { getAllPRs } = await import('../github.js');
      expect(async () => await getAllPRs()).not.toThrow();
    });

    it('should handle SSH URLs without .git suffix', async () => {
      vi.mocked(execSync).mockImplementation((cmd) => {
        if (cmd === 'gh auth token') {
          return 'mock-token';
        }
        if (cmd === 'git config --get remote.origin.url') {
          return 'git@github.com:owner/repository\n';
        }
        return '';
      });

      const mockOctokit = {
        pulls: {
          list: vi.fn().mockResolvedValue({ data: [] }),
        },
      };
      vi.mocked(Octokit).mockImplementation(() => mockOctokit as unknown as Octokit);

      const { getAllPRs } = await import('../github.js');
      expect(async () => await getAllPRs()).not.toThrow();
    });

    it('should throw for non-GitHub URLs', async () => {
      vi.mocked(execSync).mockImplementation((cmd: string) => {
        if (cmd === 'gh auth token') {
          return 'mock-token' as unknown as Buffer;
        }
        if (cmd === 'git config --get remote.origin.url') {
          return 'git@gitlab.com:owner/repository.git\n' as unknown as Buffer;
        }
        throw new Error('Unexpected command');
      });

      const { getAllPRs } = await import('../github.js');
      await expect(getAllPRs()).rejects.toThrow('Failed to get repository info');
    });
  });

  describe('PR retrieval', () => {
    it('should handle empty PR list', async () => {
      vi.mocked(execSync).mockImplementation((cmd) => {
        if (cmd === 'gh auth token') {
          return 'mock-token';
        }
        if (cmd === 'git config --get remote.origin.url') {
          return 'git@github.com:owner/repository.git\n';
        }
        return '';
      });

      const mockOctokit = {
        pulls: {
          list: vi.fn().mockResolvedValue({ data: [] }),
        },
      };
      vi.mocked(Octokit).mockImplementation(() => mockOctokit as unknown as Octokit);

      const { getAllPRs } = await import('../github.js');
      const result = await getAllPRs();
      expect(result).toEqual([]);
    });

    it('should fetch detailed check run information', async () => {
      // Reset modules to ensure fresh import
      vi.resetModules();
      
      vi.mocked(execSync).mockImplementation((cmd) => {
        if (cmd === 'gh auth token') {
          return 'mock-token' as unknown as Buffer;
        }
        if (cmd === 'git config --get remote.origin.url') {
          return 'git@github.com:owner/repository.git\n' as unknown as Buffer;
        }
        return '' as unknown as Buffer;
      });

      const mockPR = {
        number: 123,
        title: 'Test PR',
        head: { ref: 'feature/test', sha: 'abc123' },
        base: { ref: 'main' },
        state: 'open',
        draft: false,
        html_url: 'https://github.com/owner/repository/pull/123',
      };

      const mockCheckRuns = {
        total_count: 3,
        check_runs: [
          {
            name: 'CI / Build',
            status: 'completed',
            conclusion: 'failure',
            html_url: 'https://github.com/owner/repository/actions/runs/456',
            output: {
              title: 'Build failed',
              summary: 'Compilation error in main.ts\nCannot find module',
            },
          },
          {
            name: 'CI / Test',
            status: 'completed',
            conclusion: 'success',
            html_url: 'https://github.com/owner/repository/actions/runs/457',
            output: null,
          },
          {
            name: 'CI / Lint',
            status: 'in_progress',
            conclusion: null,
            html_url: 'https://github.com/owner/repository/actions/runs/458',
            output: null,
          },
        ],
      };

      const mockOctokit = {
        pulls: {
          list: vi.fn().mockResolvedValue({ data: [mockPR] }),
          get: vi.fn().mockResolvedValue({ 
            data: { 
              mergeable: true, 
              mergeable_state: 'clean' 
            } 
          }),
          listReviews: vi.fn().mockResolvedValue({ data: [] }),
        },
        checks: {
          listForRef: vi.fn().mockResolvedValue({ data: mockCheckRuns }),
        },
        graphql: vi.fn().mockResolvedValue({
          repository: {
            pullRequest: {
              reviewThreads: {
                nodes: [],
              },
            },
          },
        }),
        paginate: vi.fn().mockImplementation(async (method: unknown) => {
          if (method === mockOctokit.checks.listForRef) {
            return mockCheckRuns.check_runs;
          }
          return [];
        }),
      };
      vi.mocked(Octokit).mockImplementation(() => mockOctokit as unknown as Octokit);

      const { getAllPRs } = await import('../github.js');
      const result = await getAllPRs();
      
      expect(result).toHaveLength(1);
      expect(result[0]).toMatchObject({
        number: 123,
        title: 'Test PR',
        branch: 'feature/test',
        checks: {
          total: 3,
          passed: 1,
          failed: 1,
          pending: 1,
          details: expect.arrayContaining([
            {
              name: 'CI / Build',
              status: 'completed',
              conclusion: 'failure',
              url: 'https://github.com/owner/repository/actions/runs/456',
              output: {
                title: 'Build failed',
                summary: 'Compilation error in main.ts\nCannot find module',
              },
            },
            {
              name: 'CI / Test',
              status: 'completed',
              conclusion: 'success',
              url: 'https://github.com/owner/repository/actions/runs/457',
              output: undefined,
            },
            {
              name: 'CI / Lint',
              status: 'in_progress',
              conclusion: null,
              url: 'https://github.com/owner/repository/actions/runs/458',
              output: undefined,
            },
          ]),
        },
      });
    });
  });
});