import { WorkflowResponse, PRStatus, NextStepAction } from '../types.js';
import { getGitStatus, isCurrentBranchStale, isBranchMerged } from '../utils/git.js';
import { getAllPRs, extractFailedChecks } from '../utils/github.js';
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
        issuesFound.push(
          `Branch '${gitStatus.currentBranch}' may be stale (created before recent main merges)`
        );
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
          const currentCheckStatus =
            pr.checks.failed > 0 ? 'failed' : pr.checks.pending > 0 ? 'pending' : 'success';
          if (
            currentCheckStatus !== lastStatus.lastCheckRunStatus &&
            currentCheckStatus === 'failed'
          ) {
            issuesFound.push(`🆕 NEW: PR #${pr.number} CI checks started failing`);
          }
        }

        // Update stored status
        const reviewCount = pr.hasUnresolvedReviews ? 1 : 0;
        const checkStatus =
          pr.checks.failed > 0 ? 'failed' : pr.checks.pending > 0 ? 'pending' : 'success';
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
      issuesFound.push(`Current branch is ${gitStatus.aheadBehind.behind} commits behind main`);
      suggestedActions.push('Pull latest changes from main or rebase');
    }

    // Analyze PR status
    const failingPRs = allPRs.filter((pr) => pr.checks.failed > 0);
    if (failingPRs.length > 0) {
      issuesFound.push(`🔴 URGENT: ${failingPRs.length} PRs have failing CI checks`);
      failingPRs.forEach((pr) => {
        suggestedActions.push(`[URGENT] Fix CI failures in PR #${pr.number} (${pr.branch})`);
        
        // Add specific failing job details
        if (pr.checks.details) {
          const failedChecks = extractFailedChecks(pr.checks.details);
          failedChecks.forEach((check) => {
            issuesFound.push(`  ❌ ${check.name}: Failed`);
            issuesFound.push(`     ${check.summary}`);
          });
        }
      });
    }

    const prsNeedingRebase = allPRs.filter((pr) => pr.needsRebase);
    if (prsNeedingRebase.length > 0) {
      issuesFound.push(
        `🟡 HIGH: ${prsNeedingRebase.length} PRs need rebase after base branch merge`
      );
      prsNeedingRebase.forEach((pr) => {
        suggestedActions.push(`[HIGH] Rebase PR #${pr.number} (${pr.branch}) onto ${pr.baseRef}`);
      });
    }

    // Check for PRs with unresolved reviews
    const prsWithChangesRequested = allPRs.filter((pr) => pr.hasUnresolvedReviews);
    if (prsWithChangesRequested.length > 0) {
      issuesFound.push(
        `🟡 HIGH: ${prsWithChangesRequested.length} PRs have unresolved review comments or conversations`
      );
      prsWithChangesRequested.forEach((pr) => {
        suggestedActions.push(
          `[HIGH] Address review feedback or unresolved conversations in PR #${pr.number} (${pr.branch})`
        );
      });
    }

    // Generate contextual next steps based on current state
    const nextSteps: NextStepAction[] = [];

    // Handle urgent issues first
    if (failingPRs.length > 0) {
      failingPRs.forEach((pr) => {
        // Get failed check names for context
        const failedChecks = extractFailedChecks(pr.checks.details);
        const failedCheckNames = failedChecks.map((check) => check.name);
          
        nextSteps.push({
          action: 'fix_ci_failures',
          description: `Fix CI failures in PR #${pr.number}${failedCheckNames.length > 0 ? ` (${failedCheckNames.join(', ')})` : ''}`,
          tool: 'git_branch',
          parameters: { action: 'checkout', branch: pr.branch },
          priority: 'urgent',
          category: 'immediate',
          failedChecks: failedCheckNames,
          prNumber: pr.number,
        });
      });
    }

    // Handle PRs needing rebase
    if (prsNeedingRebase.length > 0) {
      prsNeedingRebase.forEach((pr) => {
        nextSteps.push({
          action: 'rebase_pr',
          description: `Rebase PR #${pr.number} onto ${pr.baseRef}`,
          tool: 'workflow_manage_pr',
          parameters: { action: 'rebase', prNumber: pr.number },
          priority: 'high',
          category: 'immediate',
        });
      });
    }

    // Handle PRs with unresolved reviews
    if (prsWithChangesRequested.length > 0) {
      prsWithChangesRequested.forEach((pr) => {
        nextSteps.push({
          action: 'address_pr_feedback',
          description: `Address review feedback in PR #${pr.number}`,
          tool: 'workflow_monitor_reviews',
          parameters: { prNumber: pr.number },
          priority: 'high',
          category: 'immediate',
        });
      });
    }

    // Handle git status issues
    if (!gitStatus.isClean) {
      nextSteps.push({
        action: 'commit_changes',
        description: 'Commit or stash uncommitted changes',
        tool: 'git_commit',
        parameters: { action: 'status' },
        priority: 'medium',
        category: 'immediate',
      });
    }

    // Provide workflow continuity guidance
    if (nextSteps.length === 0 || nextSteps.every((step) => step.priority !== 'urgent')) {
      if (gitStatus.currentBranch === 'main') {
        nextSteps.push({
          action: 'check_next_actions',
          description: 'Use workflow_next to determine what to work on',
          tool: 'workflow_next',
          priority: 'high',
          category: 'immediate',
        });
      } else if (allPRs.some((pr) => pr.branch === gitStatus.currentBranch)) {
        nextSteps.push({
          action: 'continue_development',
          description: 'Continue implementing features on current branch',
          priority: 'medium',
          category: 'next_logical',
        });
      } else {
        // Only suggest creating PR if branch isn't merged
        const branchMerged = await isBranchMerged(gitStatus.currentBranch);
        if (!branchMerged) {
          nextSteps.push({
            action: 'create_pr',
            description: 'Create a PR for your current branch',
            tool: 'workflow_create_pr',
            priority: 'high',
            category: 'next_logical',
          });
        }
      }
    }

    // Provide additional context for next actions
    if (gitStatus.currentBranch !== 'main' && gitStatus.aheadBehind.behind > 0) {
      nextSteps.push({
        action: 'sync_branch',
        description: 'Pull latest changes from main or rebase',
        tool: 'git_branch',
        parameters: { action: 'pull' },
        priority: 'medium',
        category: 'next_logical',
      });
    }

    // Provide next step guidance based on current state
    if (issuesFound.length === 0) {
      if (gitStatus.currentBranch === 'main') {
        suggestedActions.push('[NEXT] Create a new feature branch for your next task');
      } else if (allPRs.some((pr) => pr.branch === gitStatus.currentBranch)) {
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
      nextSteps,
      allPRStatus: allPRs,
    };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Unknown error';
    return {
      requestedData: null,
      automaticActions,
      issuesFound: [`Error getting workflow status: ${errorMessage}`],
      suggestedActions: ['Check git and GitHub configuration'],
      nextSteps: [
        {
          action: 'troubleshoot_config',
          description: 'Check git and GitHub CLI configuration',
          priority: 'high',
          category: 'immediate',
        },
      ],
      allPRStatus: [],
    };
  }
}
