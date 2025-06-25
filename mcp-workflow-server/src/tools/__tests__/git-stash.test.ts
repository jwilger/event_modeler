import { describe, it, expect, beforeEach, vi } from 'vitest';
import { execSync } from 'child_process';
import { gitStash } from '../git-stash.js';

// Mock child_process
vi.mock('child_process');
const mockExecSync = vi.mocked(execSync);

describe('gitStash', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('list action', () => {
    it('should return empty array when no stashes exist', async () => {
      mockExecSync.mockReturnValue('');

      const result = await gitStash({ action: 'list' });

      expect(result.requestedData.stashes).toEqual([]);
      expect(result.automaticActions).toContain('No stashes found');
    });

    it('should parse and return stashes', async () => {
      mockExecSync.mockReturnValue(
        'stash@{0}|abc123|WIP on feature-branch: Initial commit|2024-01-20 10:00:00 +0000\n' +
        'stash@{1}|def456|On main: Fix bug|2024-01-19 15:30:00 +0000'
      );

      const result = await gitStash({ action: 'list' });

      expect(result.requestedData.stashes).toHaveLength(2);
      expect(result.requestedData.stashes![0]).toEqual({
        ref: 'stash@{0}',
        hash: 'abc123',
        branch: 'feature-branch',
        message: 'Initial commit',
        date: '2024-01-20 10:00:00 +0000',
      });
      expect(result.automaticActions).toContain('Found 2 stashes');
    });
  });

  describe('save action', () => {
    beforeEach(() => {
      // Mock getCurrentBranch
      mockExecSync.mockImplementation((command: string) => {
        if (command === 'git branch --show-current') {
          return 'feature/test-issue-123\n';
        }
        return '';
      });
    });

    it('should save changes with auto-generated message', async () => {
      mockExecSync.mockImplementation((command: string) => {
        if (command === 'git branch --show-current') {
          return 'feature/test-issue-123\n';
        }
        if (command.startsWith('git stash push')) {
          return 'Saved working directory and index state';
        }
        if (command === 'git stash list --format="%gd|%h|%gs|%ci"') {
          return 'stash@{0}|abc123|WIP on feature/test-issue-123: WIP: Issue #123|2024-01-20 10:00:00 +0000';
        }
        return '';
      });

      const result = await gitStash({ action: 'save' });

      expect(result.requestedData.stashSaved).toBe(true);
      expect(result.automaticActions).toContain('Changes stashed successfully');
      
      // Check that the command includes an auto-generated message
      const stashCommand = mockExecSync.mock.calls.find(
        call => typeof call[0] === 'string' && call[0].startsWith('git stash push')
      )?.[0];
      expect(stashCommand).toMatch(/WIP: Issue #123/);
    });

    it('should use custom message when provided', async () => {
      const customMessage = 'My custom stash message';
      
      await gitStash({ 
        action: 'save',
        message: customMessage 
      });

      const stashCommand = mockExecSync.mock.calls.find(
        call => typeof call[0] === 'string' && call[0].startsWith('git stash push')
      )?.[0];
      expect(stashCommand).toContain(`"${customMessage}"`);
    });

    it('should include untracked files when requested', async () => {
      await gitStash({ 
        action: 'save',
        includeUntracked: true 
      });

      const stashCommand = mockExecSync.mock.calls.find(
        call => typeof call[0] === 'string' && call[0].startsWith('git stash push')
      )?.[0];
      expect(stashCommand).toContain('--include-untracked');
    });

    it('should handle no changes to stash', async () => {
      mockExecSync.mockImplementation((command: string) => {
        if (command === 'git branch --show-current') {
          return 'main\n';
        }
        if (command.startsWith('git stash push')) {
          const error = new Error('No local changes to save');
          throw error;
        }
        return '';
      });

      const result = await gitStash({ action: 'save' });

      expect(result.requestedData.stashSaved).toBe(false);
      expect(result.issuesFound).toContain('No changes to stash');
    });
  });

  describe('pop action', () => {
    it('should pop the latest stash by default', async () => {
      mockExecSync.mockReturnValue('');

      const result = await gitStash({ action: 'pop' });

      expect(mockExecSync).toHaveBeenCalledWith('git stash pop stash@{0}', { encoding: 'utf8' });
      expect(result.requestedData.stashApplied).toBe(true);
      expect(result.automaticActions).toContain('Stash popped successfully');
    });

    it('should pop specific stash by index', async () => {
      await gitStash({ 
        action: 'pop',
        stashRef: 2 
      });

      expect(mockExecSync).toHaveBeenCalledWith('git stash pop stash@{2}', { encoding: 'utf8' });
    });

    it('should handle conflicts', async () => {
      mockExecSync.mockReturnValue('CONFLICT (content): Merge conflict in file.txt');

      const result = await gitStash({ action: 'pop' });

      expect(result.issuesFound).toContain('Conflicts detected while applying stash');
      expect(result.suggestedActions).toContain('Resolve conflicts and run "git add" on resolved files');
    });

    it('should handle invalid stash reference', async () => {
      mockExecSync.mockImplementation(() => {
        const error = new Error('stash@{5} is not a stash reference');
        throw error;
      });

      const result = await gitStash({ 
        action: 'pop',
        stashRef: 5 
      });

      expect(result.requestedData.stashApplied).toBe(false);
      expect(result.issuesFound[0]).toContain('Invalid stash reference');
    });
  });

  describe('apply action', () => {
    it('should apply stash without removing it', async () => {
      mockExecSync.mockReturnValue('');
      
      const result = await gitStash({ action: 'apply' });

      expect(mockExecSync).toHaveBeenCalledWith('git stash apply stash@{0}', { encoding: 'utf8' });
      expect(result.requestedData.stashApplied).toBe(true);
      expect(result.automaticActions).toContain('Stash applied successfully');
    });

    it('should apply with quiet flag', async () => {
      mockExecSync.mockReturnValue('');
      
      await gitStash({ 
        action: 'apply',
        quiet: true 
      });

      expect(mockExecSync).toHaveBeenCalledWith('git stash apply --quiet stash@{0}', { encoding: 'utf8' });
    });
  });

  describe('drop action', () => {
    it('should drop specific stash', async () => {
      mockExecSync.mockReturnValue('');
      
      const result = await gitStash({ 
        action: 'drop',
        stashRef: 'stash@{1}' 
      });

      expect(mockExecSync).toHaveBeenCalledWith('git stash drop stash@{1}', { encoding: 'utf8' });
      expect(result.requestedData.stashDropped).toBe(true);
    });

    it('should handle drop errors', async () => {
      mockExecSync.mockImplementation(() => {
        throw new Error('No stash entries found');
      });

      const result = await gitStash({ action: 'drop' });

      expect(result.requestedData.stashDropped).toBe(false);
      expect(result.issuesFound[0]).toContain('Failed to drop stash');
    });
  });

  describe('clear action', () => {
    it('should clear all stashes', async () => {
      mockExecSync.mockReturnValue('');
      
      const result = await gitStash({ action: 'clear' });

      expect(mockExecSync).toHaveBeenCalledWith('git stash clear', { encoding: 'utf8' });
      expect(result.requestedData.stashDropped).toBe(true);
      expect(result.automaticActions).toContain('All stashes cleared');
    });
  });

  describe('show action', () => {
    it('should show stash content', async () => {
      const stashContent = 'diff --git a/file.txt b/file.txt\n+new line';
      mockExecSync.mockReturnValue(stashContent);

      const result = await gitStash({ 
        action: 'show',
        stashRef: 1 
      });

      expect(mockExecSync).toHaveBeenCalledWith('git stash show -p stash@{1}', { encoding: 'utf8' });
      expect(result.requestedData.stashContent).toBe(stashContent);
    });
  });

  describe('error handling', () => {
    it('should handle unknown actions', async () => {
      const result = await gitStash({ action: 'invalid' as unknown as 'list' });

      expect(result.issuesFound[0]).toContain('Unknown action: invalid');
    });

    it('should handle general git errors', async () => {
      // For list action, errors in parseStashList are caught and return empty array
      // Let's test with an action that doesn't catch errors internally
      mockExecSync.mockImplementation(() => {
        throw new Error('Not a git repository');
      });

      const result = await gitStash({ action: 'save' });

      expect(result.issuesFound).toHaveLength(1);
      expect(result.issuesFound[0]).toContain('Git stash operation failed');
    });
  });
});