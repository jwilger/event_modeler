import { WorkflowResponse } from '../types.js';
import { getGitStatus } from '../utils/git.js';
import { getAllPRs } from '../utils/github.js';

export async function workflowStatusTool(): Promise<WorkflowResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];

  try {
    // Get git status
    const gitStatus = await getGitStatus();
    
    // Get all PRs
    const allPRs = await getAllPRs();

    // Analyze git status for issues
    if (!gitStatus.isClean) {
      issuesFound.push(
        `Working directory has uncommitted changes: ${gitStatus.uncommittedFiles.length} files`
      );
      suggestedActions.push('Commit or stash changes before proceeding');
    }

    if (gitStatus.untrackedFiles.length > 0) {
      issuesFound.push(`${gitStatus.untrackedFiles.length} untracked files found`);
      suggestedActions.push('Review untracked files and add/ignore as needed');
    }

    // Check if current branch is behind main
    if (gitStatus.aheadBehind.behind > 0) {
      issuesFound.push(
        `Current branch is ${gitStatus.aheadBehind.behind} commits behind main`
      );
      suggestedActions.push('Pull latest changes from main or rebase');
    }

    // Analyze PR status
    const failingPRs = allPRs.filter(pr => pr.checks.failed > 0);
    if (failingPRs.length > 0) {
      issuesFound.push(`${failingPRs.length} PRs have failing CI checks`);
      failingPRs.forEach(pr => {
        suggestedActions.push(`Fix CI failures in PR #${pr.number} (${pr.branch})`);
      });
    }

    const prsNeedingRebase = allPRs.filter(pr => pr.needsRebase);
    if (prsNeedingRebase.length > 0) {
      issuesFound.push(`${prsNeedingRebase.length} PRs need rebase after base branch merge`);
      prsNeedingRebase.forEach(pr => {
        suggestedActions.push(
          `Rebase PR #${pr.number} (${pr.branch}) onto ${pr.baseRef}`
        );
      });
    }

    return {
      requestedData: {
        gitStatus,
        currentBranch: gitStatus.currentBranch,
        openPRCount: allPRs.length,
      },
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus: allPRs,
    };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Unknown error';
    return {
      requestedData: null,
      automaticActions,
      issuesFound: [`Error getting workflow status: ${errorMessage}`],
      suggestedActions: ['Check git and GitHub configuration'],
      allPRStatus: [],
    };
  }
}