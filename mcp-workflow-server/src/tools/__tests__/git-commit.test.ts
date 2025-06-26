import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { gitCommit } from '../git-commit.js';
import { execSync } from 'child_process';
import { promises as fs } from 'fs';

vi.mock('child_process');
vi.mock('fs', () => ({
  promises: {
    writeFile: vi.fn(),
    unlink: vi.fn(),
  },
}));

describe('gitCommit', () => {
  const mockExecSync = vi.mocked(execSync);
  const mockWriteFile = vi.mocked(fs.writeFile);
  const mockUnlink = vi.mocked(fs.unlink);

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
        if (cmd.includes('git commit')) {
          const error = new Error('Command failed') as Error & { stdout: string; stderr: string };
          error.stdout = '';
          error.stderr = 'cargo fmt.........Failed\n- hook id: cargo-fmt\n- exit code: 1\n\nFormatting required';
          throw error;
        }
        return '';
      });

      const result = await gitCommit({
        action: 'commit',
        message: 'Test commit',
      });

      expect(result.issuesFound).toContain('Pre-commit checks failed');
      expect(result.suggestedActions).toContain('Run `cargo fmt` to fix formatting issues');
    });

    it('should handle successful commit with pre-commit hooks', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git status --porcelain') {
          return 'A  src/file.ts';
        }
        if (cmd.includes('git commit')) {
          return 'MCP Server lint..........................................................Passed\nMCP Server build.........................................................Passed\n[feature-branch abc123] Test commit\n 1 file changed, 10 insertions(+)';
        }
        if (cmd === 'git rev-parse HEAD') {
          return 'abc123';
        }
        return '';
      });

      mockWriteFile.mockResolvedValue(undefined);
      mockUnlink.mockResolvedValue(undefined);

      const result = await gitCommit({
        action: 'commit',
        message: 'Test commit',
      });

      expect(result.requestedData.commitHash).toBe('abc123');
      expect(result.automaticActions).toContain('MCP Server lint..........................................................Passed');
      expect(result.automaticActions).toContain('MCP Server build.........................................................Passed');
    });

    it('should parse detailed TypeScript errors from pre-commit', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git status --porcelain') {
          return 'A  src/file.ts';
        }
        if (cmd.includes('git commit')) {
          const error = new Error('Command failed') as Error & { stdout: string; stderr: string };
          error.stdout = '';
          error.stderr = `MCP Server build.........................................................Failed
- hook id: mcp-server-build
- exit code: 1

src/tools/example.ts(42,5): error TS2339: Property 'foo' does not exist on type 'Bar'.
src/tools/example.ts(78,12): error TS2304: Cannot find name 'undefined_var'.`;
          throw error;
        }
        return '';
      });

      const result = await gitCommit({
        action: 'commit',
        message: 'Test commit',
      });

      expect(result.issuesFound).toContain('Pre-commit checks failed: TypeScript');
      expect(result.automaticActions).toContain('Pre-commit hook failures:');
      expect(result.automaticActions).toContain('TypeScript check failed with 2 errors:');
      expect(result.automaticActions).toContain("  src/tools/example.ts:42:5 [error] - Property 'foo' does not exist on type 'Bar'. (TS2339)");
      expect(result.automaticActions).toContain("  src/tools/example.ts:78:12 [error] - Cannot find name 'undefined_var'. (TS2304)");
      expect(result.suggestedActions).toContain('Fix TypeScript errors in the affected files');
      expect(result.suggestedActions).toContain('Run `npm run build` to see full error details');
    });

    it('should parse ESLint errors from pre-commit', async () => {
      mockExecSync.mockImplementation((cmd) => {
        if (cmd === 'git status --porcelain') {
          return 'A  src/file.ts';
        }
        if (cmd.includes('git commit')) {
          const error = new Error('Command failed') as Error & { stdout: string; stderr: string };
          error.stdout = '';
          error.stderr = `MCP Server lint..........................................................Failed
- hook id: mcp-server-lint
- exit code: 1

/home/user/project/src/tools/example.ts
  10:5  error  'foo' is defined but never used  @typescript-eslint/no-unused-vars
  25:10  warning  Missing return type on function  @typescript-eslint/explicit-function-return-type`;
          throw error;
        }
        return '';
      });

      const result = await gitCommit({
        action: 'commit',
        message: 'Test commit',
      });

      expect(result.issuesFound).toContain('Pre-commit checks failed: ESLint');
      expect(result.automaticActions).toContain('ESLint found 2 issues:');
      expect(result.automaticActions).toContain("  /home/user/project/src/tools/example.ts:10:5 [error] - 'foo' is defined but never used (@typescript-eslint/no-unused-vars)");
      expect(result.suggestedActions).toContain('Fix ESLint issues in the affected files');
      expect(result.suggestedActions).toContain('Run `npm run lint -- --fix` to auto-fix some issues');
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