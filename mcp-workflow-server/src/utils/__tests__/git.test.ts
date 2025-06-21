import { describe, it, expect, vi, beforeEach, MockedFunction } from 'vitest';
import { getGitStatus, isCurrentBranchStale } from '../git.js';

// Mock the entire simple-git module
vi.mock('simple-git', () => {
  const mockGit = {
    revparse: vi.fn(),
    status: vi.fn(),
    raw: vi.fn(),
    log: vi.fn(),
  };
  
  return {
    simpleGit: vi.fn(() => mockGit),
  };
});

// Import after mock is set up
import { simpleGit } from 'simple-git';

describe('Git Utilities', () => {
  let mockGit: {
    revparse: MockedFunction<any>;
    status: MockedFunction<any>;
    raw: MockedFunction<any>;
    log: MockedFunction<any>;
  };

  beforeEach(() => {
    vi.clearAllMocks();
    // Get the mocked git instance
    mockGit = (simpleGit as any)();
  });

  describe('getGitStatus', () => {
    it('should return clean status when no changes', async () => {
      mockGit.revparse.mockResolvedValue('feature/test\n');
      mockGit.status.mockResolvedValue({
        isClean: () => true,
        modified: [],
        deleted: [],
        created: [],
        renamed: [],
        not_added: [],
      });
      mockGit.raw.mockResolvedValueOnce('0\n').mockResolvedValueOnce('0\n');
      mockGit.log.mockResolvedValue({
        latest: {
          hash: 'abc123',
          message: 'Test commit',
          date: '2024-01-01',
        },
      });

      const status = await getGitStatus();

      expect(status.currentBranch).toBe('feature/test');
      expect(status.isClean).toBe(true);
      expect(status.uncommittedFiles).toHaveLength(0);
      expect(status.untrackedFiles).toHaveLength(0);
      expect(status.aheadBehind).toEqual({ ahead: 0, behind: 0 });
      expect(status.lastCommit.hash).toBe('abc123');
    });

    it('should detect uncommitted changes', async () => {
      mockGit.revparse.mockResolvedValue('main\n');
      mockGit.status.mockResolvedValue({
        isClean: () => false,
        modified: ['file1.ts', 'file2.ts'],
        deleted: ['file3.ts'],
        created: ['file4.ts'],
        renamed: [{ from: 'old.ts', to: 'new.ts' }],
        not_added: ['untracked.ts'],
      });
      mockGit.raw.mockResolvedValueOnce('2\n').mockResolvedValueOnce('3\n');
      mockGit.log.mockResolvedValue({
        latest: {
          hash: 'def456',
          message: 'Another commit',
          date: '2024-01-02',
        },
      });

      const status = await getGitStatus();

      expect(status.isClean).toBe(false);
      expect(status.uncommittedFiles).toHaveLength(5);
      expect(status.uncommittedFiles).toContain('file1.ts');
      expect(status.uncommittedFiles).toContain('new.ts');
      expect(status.untrackedFiles).toEqual(['untracked.ts']);
      expect(status.aheadBehind).toEqual({ ahead: 2, behind: 3 });
    });

    it('should handle errors gracefully', async () => {
      mockGit.revparse.mockRejectedValue(new Error('Git error'));

      await expect(getGitStatus()).rejects.toThrow('Failed to get git status');
    });
  });

  describe('isCurrentBranchStale', () => {
    it('should return false for main branch', async () => {
      mockGit.revparse.mockResolvedValue('main\n');

      const result = await isCurrentBranchStale();

      expect(result).toBe(false);
      expect(mockGit.raw).not.toHaveBeenCalled();
    });

    it('should detect stale branch with many commits on main', async () => {
      mockGit.revparse.mockResolvedValue('feature/old\n');
      mockGit.raw.mockResolvedValueOnce('abc123\n').mockResolvedValueOnce('15\n');

      const result = await isCurrentBranchStale();

      expect(result).toBe(true);
    });

    it('should return false for fresh branch', async () => {
      mockGit.revparse.mockResolvedValue('feature/new\n');
      mockGit.raw.mockResolvedValueOnce('def456\n').mockResolvedValueOnce('2\n');

      const result = await isCurrentBranchStale();

      expect(result).toBe(false);
    });

    it('should return false on error', async () => {
      mockGit.revparse.mockResolvedValue('feature/test\n');
      mockGit.raw.mockRejectedValue(new Error('Git error'));

      const result = await isCurrentBranchStale();

      expect(result).toBe(false);
    });
  });
});