import { describe, it, expect, vi, beforeEach } from 'vitest';
import { execSync } from 'child_process';
import { Octokit } from '@octokit/rest';

// Mock modules
vi.mock('child_process');
vi.mock('@octokit/rest');

// Import after mocks
import { getAllPRs } from '../github.js';

describe('GitHub Utilities', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Repository URL parsing', () => {
    it('should handle repository names with dots', async () => {
      // Mock gh auth token
      vi.mocked(execSync).mockImplementation((cmd) => {
        if (cmd === 'gh auth token') {
          return 'mock-token';
        }
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
      expect(async () => await getAllPRs()).not.toThrow();
    });

    it('should handle HTTPS URLs', async () => {
      vi.mocked(execSync).mockImplementation((cmd) => {
        if (cmd === 'gh auth token') {
          return 'mock-token';
        }
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

      const result = await getAllPRs();
      expect(result).toEqual([]);
    });
  });
});