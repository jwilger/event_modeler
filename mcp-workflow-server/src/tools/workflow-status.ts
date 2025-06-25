import { WorkflowResponse, PRStatus } from '../types.js';
import { getGitStatus, isCurrentBranchStale, isBranchMerged } from '../utils/git.js';
import { getAllPRs } from '../utils/github.js';
import { StateStore } from '../state/store.js';

const stateStore = new StateStore();

export async function workflowStatusTool(): Promise<WorkflowResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];
  
  // Update last status check time
  stateStore.updateLastStatusCheck();

  try {
    // Get git status
    const gitStatus = await getGitStatus();
    automaticActions.push('Checked git status and branch information');
    
    // Check if branch is stale or merged
    const isStale = await isCurrentBranchStale();
    
    if (gitStatus.currentBranch !== 'main') {
      const isMerged = await isBranchMerged(gitStatus.currentBranch);
      if (isMerged) {
        issuesFound.push(`Branch '${gitStatus.currentBranch}' has been merged to main`);
        suggestedActions.push('[NEXT] Switch to main: git checkout main && git pull origin main');
      } else if (isStale) {
        issuesFound.push(`Branch '${gitStatus.currentBranch}' may be stale (created before recent main merges)`);
        suggestedActions.push('Consider rebasing on latest main or creating a fresh branch');
      }
    }
    
    // Get all PRs
    let allPRs: PRStatus[] = [];
    try {
      allPRs = await getAllPRs();
      automaticActions.push(`Retrieved status for ${allPRs.length} open PRs`);
      
      // Check for new reviews or status changes
      for (const pr of allPRs) {
        const lastStatus = stateStore.getPRStatus(pr.number);
        
        if (lastStatus) {
          // Detect new reviews
          const currentReviewCount = pr.hasUnresolvedReviews ? 1 : 0; // Simplified for now
          if (currentReviewCount > lastStatus.lastReviewCount) {
            issuesFound.push(`🆕 NEW: PR #${pr.number} has new review comments`);
            suggestedActions.push(`[NEW] Check and address review feedback in PR #${pr.number}`);
          }
          
          // Detect CI status changes
          const currentCheckStatus = pr.checks.failed > 0 ? 'failed' : 
                                   pr.checks.pending > 0 ? 'pending' : 'success';
          if (currentCheckStatus !== lastStatus.lastCheckRunStatus && currentCheckStatus === 'failed') {
            issuesFound.push(`🆕 NEW: PR #${pr.number} CI checks started failing`);
          }
        }
        
        // Update stored status
        const reviewCount = pr.hasUnresolvedReviews ? 1 : 0;
        const checkStatus = pr.checks.failed > 0 ? 'failed' : 
                          pr.checks.pending > 0 ? 'pending' : 'success';
        stateStore.updatePRStatus(pr.number, reviewCount, checkStatus);
      }
    } catch {
      // If GitHub API fails, continue with empty PR list
      issuesFound.push('Unable to retrieve PR status from GitHub');
      suggestedActions.push('Ensure gh CLI is authenticated: gh auth status');
    }
    
    // Record current branch creation if not already tracked
    if (gitStatus.currentBranch !== 'main') {
      stateStore.recordBranchCreation(gitStatus.currentBranch);
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
      issuesFound.push(`🟡 HIGH: ${prsWithChangesRequested.length} PRs have unresolved review comments or conversations`);
      prsWithChangesRequested.forEach(pr => {
        suggestedActions.push(
          `[HIGH] Address review feedback or unresolved conversations in PR #${pr.number} (${pr.branch})`
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
        // Only suggest creating PR if branch isn't merged
        const branchMerged = await isBranchMerged(gitStatus.currentBranch);
        if (!branchMerged) {
          suggestedActions.push('[NEXT] Create a PR for your current branch');
        }
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