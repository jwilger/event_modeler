import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { gitCommit } from '../git-commit.js';
import { execSync } from 'child_process';
import { promises as fs } from 'fs';

vi.mock('child_process');
vi.mock('fs', () => ({
  promises: {
    writeFile: vi.fn(),
    unlink: vi.fn(),
    access: vi.fn(),
  },
}));

describe('gitCommit', () => {
  const mockExecSync = vi.mocked(execSync);
  const mockWriteFile = vi.mocked(fs.writeFile);
  vi.mocked(fs.unlink);
  const mockAccess = vi.mocked(fs.access);

  beforeEach(() => {
    vi.clearAllMocks();
    // Default branch
    mockExecSync.mockImplementation((cmd) => {
      if (cmd === 'git branch --show-current') {
        return 'feature/test-feature-123';
      }
      return '';
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('status action', () => {
    it('should return file statuses', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git status --porcelain') {
          return 'M  src/file1.ts\n?? src/file2.ts\nA  src/file3.ts';
        }
        if (cmd === 'git branch --show-current') {
          return 'feature/test-123';
        }
        return '';
      });

      const result = await gitCommit({ action: 'status' });

      expect(result.requestedData.fileStatuses).toHaveLength(3);
      expect(result.requestedData.stagedFiles).toEqual(['src/file1.ts', 'src/file3.ts']);
      expect(result.requestedData.unstagedFiles).toEqual([]);
      expect(result.automaticActions).toContain('Found 3 files with changes');
    });

    it('should handle empty status', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git status --porcelain') {
          return '';
        }
        return 'main';
      });

      const result = await gitCommit({ action: 'status' });

      expect(result.requestedData.fileStatuses).toHaveLength(0);
      expect(result.automaticActions).toContain('Found 0 files with changes');
    });
  });

  describe('stage action', () => {
    it('should stage specific files', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git status --porcelain') {
          return cmd.includes('git add') ? 'A  src/file1.ts' : '?? src/file1.ts';
        }
        return '';
      });

      const result = await gitCommit({
        action: 'stage',
        files: ['src/file1.ts'],
      });

      expect(mockExecSync).toHaveBeenCalledWith('git add "src/file1.ts"', { encoding: 'utf8' });
      expect(result.automaticActions).toContain('Staged: src/file1.ts');
    });

    it('should stage all files when no files specified', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git status --porcelain') {
          return '?? src/file1.ts\n M src/file2.ts';
        }
        return '';
      });

      await gitCommit({ action: 'stage' });

      expect(mockExecSync).toHaveBeenCalledWith('git add "src/file1.ts"', { encoding: 'utf8' });
      expect(mockExecSync).toHaveBeenCalledWith('git add "src/file2.ts"', { encoding: 'utf8' });
    });

    it('should stage all tracked files with all flag', async () => {
      const result = await gitCommit({
        action: 'stage',
        all: true,
      });

      expect(mockExecSync).toHaveBeenCalledWith('git add -u', { encoding: 'utf8' });
      expect(result.automaticActions).toContain('Staged all tracked files with modifications');
    });

    it('should handle no files to stage', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git status --porcelain') {
          return '';
        }
        return '';
      });

      const result = await gitCommit({ action: 'stage' });

      expect(result.issuesFound).toContain('No files to stage');
    });
  });

  describe('unstage action', () => {
    it('should unstage specific files', async () => {
      const result = await gitCommit({
        action: 'unstage',
        files: ['src/file1.ts'],
      });

      expect(mockExecSync).toHaveBeenCalledWith('git reset HEAD "src/file1.ts"', { encoding: 'utf8' });
      expect(result.automaticActions).toContain('Unstaged: src/file1.ts');
    });

    it('should unstage all files when no files specified', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git status --porcelain') {
          return 'A  src/file1.ts\nM  src/file2.ts';
        }
        return '';
      });

      await gitCommit({ action: 'unstage' });

      expect(mockExecSync).toHaveBeenCalledWith('git reset HEAD "src/file1.ts"', { encoding: 'utf8' });
      expect(mockExecSync).toHaveBeenCalledWith('git reset HEAD "src/file2.ts"', { encoding: 'utf8' });
    });
  });

  describe('commit action', () => {
    it('should create commit with formatted message', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git status --porcelain') {
          return 'A  src/file1.ts';
        }
        if (cmd === 'git rev-parse HEAD') {
          return 'abc123def456';
        }
        if (cmd === 'git branch --show-current') {
          return 'feature/test-123';
        }
        if (cmd.includes('cargo fmt')) {
          return '';
        }
        if (cmd.includes('cargo clippy')) {
          return '';
        }
        return '';
      });

      const result = await gitCommit({
        action: 'commit',
        message: 'Add new feature',
      });

      expect(result.requestedData.commitHash).toBe('abc123def456');
      expect(result.requestedData.issueNumber).toBe(123);
      expect(mockWriteFile).toHaveBeenCalled();
      
      // Check commit message formatting
      const writeCall = mockWriteFile.mock.calls[0];
      const message = writeCall[1] as string;
      expect(message).toContain('Add new feature (#123)');
      expect(message).toContain('🤖 Generated with [Claude Code]');
      expect(message).toContain('Co-Authored-By: Claude <noreply@anthropic.com>');
    });

    it('should fail when no staged files', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git status --porcelain') {
          return '?? src/file1.ts';
        }
        return '';
      });

      const result = await gitCommit({
        action: 'commit',
        message: 'Test commit',
      });

      expect(result.issuesFound).toContain('No staged files to commit');
      expect(result.suggestedActions).toContain('Stage files first using stage action');
    });

    it('should fail pre-commit checks', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git status --porcelain') {
          return 'A  src/file.rs';
        }
        if (cmd === 'cargo fmt -- --check') {
          throw new Error('Formatting required');
        }
        return '';
      });

      const result = await gitCommit({
        action: 'commit',
        message: 'Test commit',
      });

      expect(result.issuesFound).toContain('Pre-commit checks failed');
      expect(result.automaticActions).toContain('cargo fmt: ✗ (run `cargo fmt` to fix)');
    });

    it('should handle TypeScript pre-commit checks', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git status --porcelain') {
          return 'A  src/file.ts';
        }
        if (cmd.includes('npm run lint')) {
          return '';
        }
        if (cmd.includes('npm run build')) {
          return '';
        }
        if (cmd === 'git rev-parse HEAD') {
          return 'abc123';
        }
        return '';
      });

      mockAccess.mockResolvedValue(undefined); // mcp-workflow-server exists

      const result = await gitCommit({
        action: 'commit',
        message: 'Test commit',
      });

      expect(result.automaticActions).toContain('npm run lint: ✓');
      expect(result.automaticActions).toContain('TypeScript build: ✓');
    });
  });

  describe('amend action', () => {
    it('should amend commit with new message', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git rev-parse HEAD') {
          return 'abc123def456';
        }
        if (cmd === 'git status --porcelain') {
          return '';
        }
        return '';
      });

      const result = await gitCommit({
        action: 'amend',
        message: 'Updated message',
      });

      expect(result.requestedData.commitHash).toBe('abc123def456');
      expect(result.automaticActions).toContain('Amended commit: abc123d');
    });

    it('should amend with staged files', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git status --porcelain') {
          return 'A  src/file.rs';
        }
        if (cmd === 'git rev-parse HEAD') {
          return 'abc123';
        }
        if (cmd.includes('cargo')) {
          return '';
        }
        return '';
      });

      const result = await gitCommit({
        action: 'amend',
      });

      expect(result.automaticActions).toContain('Adding 1 files to previous commit');
    });

    it('should fail when no commits to amend', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git rev-parse HEAD') {
          throw new Error('No commits');
        }
        return '';
      });

      const result = await gitCommit({ action: 'amend' });

      expect(result.issuesFound[0]).toContain('No commits to amend');
    });
  });

  describe('issue number extraction', () => {
    it('should extract issue number from branch name', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git branch --show-current') {
          return 'feature/add-new-feature-456';
        }
        if (cmd === 'git status --porcelain') {
          return 'A  src/file.ts';
        }
        if (cmd === 'git rev-parse HEAD') {
          return 'abc123';
        }
        return '';
      });

      const result = await gitCommit({
        action: 'commit',
        message: 'Test',
      });

      expect(result.requestedData.issueNumber).toBe(456);
    });

    it('should use provided issue number over detected', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git branch --show-current') {
          return 'feature/test-123';
        }
        if (cmd === 'git status --porcelain') {
          return 'A  src/file.ts';
        }
        if (cmd === 'git rev-parse HEAD') {
          return 'abc123';
        }
        return '';
      });

      const result = await gitCommit({
        action: 'commit',
        message: 'Test',
        issueNumber: 789,
      });

      expect(result.requestedData.issueNumber).toBe(789);
      const writeCall = mockWriteFile.mock.calls[0];
      const message = writeCall[1] as string;
      expect(message).toContain('#789');
    });
  });
});