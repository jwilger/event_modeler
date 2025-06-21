import { simpleGit, SimpleGit } from 'simple-git';
import { GitStatus } from '../types.js';

const git: SimpleGit = simpleGit();

export async function getGitStatus(): Promise<GitStatus> {
  try {
    // Get current branch
    const currentBranch = await git.revparse(['--abbrev-ref', 'HEAD']);

    // Get status
    const status = await git.status();

    // Get ahead/behind info relative to main
    let aheadBehind = { ahead: 0, behind: 0 };
    try {
      const ahead = await git.raw(['rev-list', '--count', 'origin/main..HEAD']);
      const behind = await git.raw(['rev-list', '--count', 'HEAD..origin/main']);
      aheadBehind = {
        ahead: parseInt(ahead.trim(), 10) || 0,
        behind: parseInt(behind.trim(), 10) || 0,
      };
    } catch {
      // If we can't get ahead/behind, just use defaults
    }

    // Get last commit info
    const log = await git.log({ maxCount: 1 });
    const lastCommit = log.latest
      ? {
          hash: log.latest.hash,
          message: log.latest.message,
          date: log.latest.date,
        }
      : {
          hash: '',
          message: 'No commits',
          date: new Date().toISOString(),
        };

    return {
      currentBranch: currentBranch.trim(),
      isClean: status.isClean(),
      uncommittedFiles: [
        ...status.modified,
        ...status.deleted,
        ...status.created,
        ...status.renamed.map((r) => r.to),
      ],
      untrackedFiles: status.not_added,
      aheadBehind,
      lastCommit,
    };
  } catch (error) {
    throw new Error(`Failed to get git status: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

export async function isCurrentBranchStale(): Promise<boolean> {
  try {
    // Get the creation date of current branch
    const currentBranch = await git.revparse(['--abbrev-ref', 'HEAD']);
    if (currentBranch.trim() === 'main') {
      return false; // main is never stale
    }

    // Get the merge base with main
    const mergeBase = await git.raw(['merge-base', 'HEAD', 'origin/main']);
    
    // Get commits on main since merge base
    const commitsSinceBranch = await git.raw([
      'rev-list',
      '--count',
      `${mergeBase.trim()}..origin/main`,
    ]);

    // If there are many commits on main since this branch was created, it might be stale
    return parseInt(commitsSinceBranch.trim(), 10) > 10;
  } catch {
    return false;
  }
}