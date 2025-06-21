import { WorkflowResponse, PRStatus } from '../types.js';
import { getGitStatus, isCurrentBranchStale } from '../utils/git.js';
import { getAllPRs } from '../utils/github.js';

export async function workflowStatusTool(): Promise<WorkflowResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];

  try {
    // Get git status
    const gitStatus = await getGitStatus();
    automaticActions.push('Checked git status and branch information');
    
    // Check if branch is stale
    const isStale = await isCurrentBranchStale();
    if (isStale && gitStatus.currentBranch !== 'main') {
      issuesFound.push(`Branch '${gitStatus.currentBranch}' may be stale (created before recent main merges)`);
      suggestedActions.push('Consider rebasing on latest main or creating a fresh branch');
    }
    
    // Get all PRs
    let allPRs: PRStatus[] = [];
    try {
      allPRs = await getAllPRs();
      automaticActions.push(`Retrieved status for ${allPRs.length} open PRs`);
    } catch (error) {
      // If GitHub API fails, continue with empty PR list
      issuesFound.push('Unable to retrieve PR status from GitHub');
      suggestedActions.push('Ensure gh CLI is authenticated: gh auth status');
    }

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
      issuesFound.push(`🔴 URGENT: ${failingPRs.length} PRs have failing CI checks`);
      failingPRs.forEach(pr => {
        suggestedActions.push(`[URGENT] Fix CI failures in PR #${pr.number} (${pr.branch})`);
      });
    }

    const prsNeedingRebase = allPRs.filter(pr => pr.needsRebase);
    if (prsNeedingRebase.length > 0) {
      issuesFound.push(`🟡 HIGH: ${prsNeedingRebase.length} PRs need rebase after base branch merge`);
      prsNeedingRebase.forEach(pr => {
        suggestedActions.push(
          `[HIGH] Rebase PR #${pr.number} (${pr.branch}) onto ${pr.baseRef}`
        );
      });
    }
    
    // Check for PRs with unresolved reviews
    const prsWithChangesRequested = allPRs.filter(pr => pr.hasUnresolvedReviews);
    if (prsWithChangesRequested.length > 0) {
      issuesFound.push(`🟡 HIGH: ${prsWithChangesRequested.length} PRs have unresolved review comments`);
      prsWithChangesRequested.forEach(pr => {
        suggestedActions.push(
          `[HIGH] Address review feedback in PR #${pr.number} (${pr.branch})`
        );
      });
    }
    
    // Provide next step guidance based on current state
    if (issuesFound.length === 0) {
      if (gitStatus.currentBranch === 'main') {
        suggestedActions.push('[NEXT] Create a new feature branch for your next task');
      } else if (allPRs.some(pr => pr.branch === gitStatus.currentBranch)) {
        suggestedActions.push('[NEXT] Continue implementing features on current branch');
      } else {
        suggestedActions.push('[NEXT] Create a PR for your current branch');
      }
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