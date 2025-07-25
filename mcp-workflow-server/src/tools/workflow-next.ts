import { Octokit } from '@octokit/rest';
import { execSync } from 'child_process';
import { WorkflowResponse, NextStepAction } from '../types.js';
import { 
  getProjectConfig, 
  getMissingConfigFields, 
  createConfigRequest,
  getWorkflowState,
  updateWorkflowState,
  completeAction,
  getRequiredActions
} from '../config.js';
import {
  workflowMonitorReviews,
  requestCopilotReReview,
  type ReviewInfo,
  type PRReviewStatus,
} from './workflow-monitor-reviews.js';
import { getRepoInfo, getAllPRs, extractFailedChecks } from '../utils/github.js';
import { isBranchMerged } from '../utils/git.js';
import { getGitHubToken } from '../utils/auth.js';

// Constants for bot reviewer names
const COPILOT_BOT_REVIEWER = 'copilot-pull-request-reviewer[bot]';
const COPILOT_HUMAN_REVIEWER = 'copilot-pull-request-reviewer';

interface TodoItem {
  text: string;
  checked: boolean;
  index: number;
}

interface Choice {
  id: string | number;
  title: string;
  description?: string;
  metadata?: {
    state?: string;
    labels?: string[];
  };
}

interface DecisionContext {
  prompt: string;
  additionalInfo?: {
    currentBranch?: string;
    existingPR?: {
      number: number;
      title: string;
    } | null;
  };
}

// Enhanced NextStepAction that includes all workflow-next context fields
interface WorkflowNextStepAction extends NextStepAction {
  issueNumber?: number;
  title?: string;
  status?: string;
  todoItem?: string;
  todoIndex?: number;
  totalTodos?: number;
  completedTodos?: number;
  suggestion?: string;
  projectUrl?: string;
  reason?: string;
  epicNumber?: number;
  epicTitle?: string;
  subIssues?: Array<{
    number: number;
    title: string;
    status: string;
  }>;
  // Fields for LLM decision requests
  decisionType?: 'select_next_issue' | 'prioritize_work';
  decisionId?: string; // Unique ID to track this decision
  choices?: Choice[];
  decisionContext?: DecisionContext;
  // Fields for config requests
  missingConfig?: string[];
  configSuggestions?: string[];
  // Fields for PR feedback
  prNumber?: number;
  reviewStatus?: string;
  reviews?: ReviewInfo[];
  prUrl?: string;
  author?: string;
  // Fields for merge-ready PRs
  ciStatus?: 'success' | 'pending' | 'failure' | 'unknown';
  mergeable?: boolean;
  mergeableState?: string;
}

interface WorkflowContext {
  currentBranch?: string;
  hasUncommittedChanges?: boolean;
  myOpenPRs?: PRReviewStatus[];
  othersPRsToReview?: PRReviewStatus[];
  totalOpenPRs?: number;
  myPRsNeedingAttention?: PRReviewStatus[];
  otherOpenPRs?: number;
  existingPR?: {
    number: number;
    title: string;
  } | null;
  assignedIssues?: number;
  inProgressIssues?: number;
  totalTodos?: number;
  completedTodos?: number;
  hasPR?: boolean;
  branchMerged?: boolean;
  currentConfig?: ReturnType<typeof getProjectConfig>['config'];
}

interface WorkflowNextResponse extends WorkflowResponse {
  requestedData: {
    context: WorkflowContext;
  };
  nextSteps: WorkflowNextStepAction[];
}

function parseTodoItems(body: string): TodoItem[] {
  const lines = body.split('\n');
  const todos: TodoItem[] = [];
  let index = 0;

  for (const line of lines) {
    const checkedMatch = line.match(/^\s*-\s+\[x\]\s+(.+)$/i);
    const uncheckedMatch = line.match(/^\s*-\s+\[\s*\]\s+(.+)$/);

    if (checkedMatch || uncheckedMatch) {
      todos.push({
        text: (checkedMatch || uncheckedMatch)![1].trim(),
        checked: !!checkedMatch,
        index: index++,
      });
    }
  }

  return todos;
}

interface PRMergeReadiness {
  isMergeReady: boolean;
  ciStatus: 'success' | 'pending' | 'failure' | 'unknown';
  mergeable: boolean;
  mergeableState: string;
  hasApprovals: boolean;
  hasUnresolvedComments: boolean;
  blockingReasons: string[];
}

async function checkPRMergeReadiness(
  octokit: Octokit,
  owner: string,
  repo: string,
  prNumber: number,
  pr: PRReviewStatus
): Promise<PRMergeReadiness> {
  const blockingReasons: string[] = [];

  try {
    // Get detailed PR information including mergeable state
    const { data: prDetails } = await octokit.pulls.get({
      owner,
      repo,
      pull_number: prNumber,
    });

    // Check CI status from commit status
    let ciStatus: 'success' | 'pending' | 'failure' | 'unknown' = 'unknown';
    if (prDetails.head.sha) {
      try {
        const { data: combinedStatus } = await octokit.repos.getCombinedStatusForRef({
          owner,
          repo,
          ref: prDetails.head.sha,
        });

        if (combinedStatus.state === 'success') {
          ciStatus = 'success';
        } else if (combinedStatus.state === 'pending') {
          ciStatus = 'pending';
          blockingReasons.push('CI checks are still running');
        } else if (combinedStatus.state === 'failure' || combinedStatus.state === 'error') {
          ciStatus = 'failure';
          blockingReasons.push('CI checks are failing');
        }
      } catch {
        // If status checks fail, try check runs as fallback
        try {
          const { data: checkRuns } = await octokit.checks.listForRef({
            owner,
            repo,
            ref: prDetails.head.sha,
          });

          if (checkRuns.total_count > 0) {
            const hasFailures = checkRuns.check_runs.some(
              (run) =>
                run.status === 'completed' &&
                run.conclusion !== 'success' &&
                run.conclusion !== 'skipped'
            );
            const hasPending = checkRuns.check_runs.some((run) => run.status !== 'completed');

            if (hasFailures) {
              ciStatus = 'failure';
              blockingReasons.push('CI checks are failing');
            } else if (hasPending) {
              ciStatus = 'pending';
              blockingReasons.push('CI checks are still running');
            } else {
              ciStatus = 'success';
            }
          }
        } catch {
          // Unable to get CI status
        }
      }
    }

    // Check mergeable state
    const mergeable = prDetails.mergeable === true;
    const mergeableState = prDetails.mergeable_state || 'unknown';

    if (!mergeable) {
      if (mergeableState === 'conflicting') {
        blockingReasons.push('Has merge conflicts');
      } else if (mergeableState === 'blocked') {
        blockingReasons.push('Merge is blocked by branch protection rules');
      } else {
        blockingReasons.push('Not mergeable');
      }
    }

    // Check review status
    const hasApprovals = pr.reviewStatus === 'approved';
    if (!hasApprovals) {
      blockingReasons.push('Needs approval');
    }

    // Check for unresolved comments
    const hasUnresolvedComments = pr.commentSummary ? pr.commentSummary.unresolved > 0 : false;
    if (hasUnresolvedComments) {
      blockingReasons.push(`Has ${pr.commentSummary?.unresolved || 0} unresolved comments`);
    }

    const isMergeReady =
      hasApprovals &&
      !hasUnresolvedComments &&
      mergeable &&
      ciStatus === 'success' &&
      mergeableState === 'clean';

    return {
      isMergeReady,
      ciStatus,
      mergeable,
      mergeableState,
      hasApprovals,
      hasUnresolvedComments,
      blockingReasons,
    };
  } catch (error) {
    // Log the error for better observability
    console.error('Error during merge readiness check:', error);
    // If we can't determine merge readiness, assume it's not ready
    return {
      isMergeReady: false,
      ciStatus: 'unknown',
      mergeable: false,
      mergeableState: 'unknown',
      hasApprovals: pr.reviewStatus === 'approved',
      hasUnresolvedComments: pr.commentSummary ? pr.commentSummary.unresolved > 0 : false,
      blockingReasons: ['Unable to determine merge readiness'],
    };
  }
}

async function getCurrentUser(): Promise<string> {
  try {
    const output = execSync('gh api user --jq .login', { encoding: 'utf8' });
    return output.trim();
  } catch {
    throw new Error('Failed to get current GitHub user. Make sure gh CLI is authenticated.');
  }
}

export async function workflowNext(): Promise<WorkflowNextResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];

  try {
    // Check configuration first
    const { config, isComplete } = getProjectConfig();

    if (!isComplete) {
      const missingFields = getMissingConfigFields(config);
      const configRequest = createConfigRequest(missingFields);

      return {
        requestedData: {
          context: {
            currentConfig: config,
          },
        },
        automaticActions: ['Configuration check failed - missing required fields'],
        issuesFound: [`Missing configuration: ${missingFields.join(', ')}`],
        suggestedActions: configRequest.suggestions,
        nextSteps: [
          {
            action: 'requires_config',
            description: 'Configure the workflow server with missing settings',
            priority: 'urgent',
            category: 'immediate',
            tool: 'workflow_configure',
            missingConfig: configRequest.missingFields,
            configSuggestions: configRequest.suggestions,
            suggestion:
              'Configuration is incomplete. Please run workflow_configure to set missing values.',
          },
        ],
        allPRStatus: [],
      };
    }
    
    // Priority 0: Check for failing CI builds across all PRs
    try {
      const allPRStatus = await getAllPRs();
      const prsWithFailingCI = allPRStatus.filter((pr) => pr.checks.failed > 0);
      
      if (prsWithFailingCI.length > 0) {
        automaticActions.push(`🔴 URGENT: Found ${prsWithFailingCI.length} PRs with failing CI checks`);
        
        // Get the first PR with failing CI to prioritize
        const mostUrgentPR = prsWithFailingCI[0];
        const failedChecks = extractFailedChecks(mostUrgentPR.checks.details);
        
        return {
          requestedData: {
            context: {
              // Add CI failure info to context
              totalOpenPRs: prsWithFailingCI.length,
            },
          },
          automaticActions,
          issuesFound: [
            `🔴 CI is failing on ${prsWithFailingCI.length} PR(s) - this must be fixed first!`,
            ...failedChecks.map(check => `  ❌ ${check.name}: ${check.summary}`)
          ],
          suggestedActions: [
            `Fix CI failures in PR #${mostUrgentPR.number} (${mostUrgentPR.branch})`,
            'CI failures block all other work and should be resolved immediately'
          ],
          nextSteps: [
            {
              action: 'fix_ci_failures',
              description: `Fix CI failures in PR #${mostUrgentPR.number}: ${failedChecks.map(c => c.name).join(', ')}`,
              priority: 'urgent',
              category: 'immediate',
              tool: 'git_branch',
              parameters: {
                action: 'checkout',
                branch: mostUrgentPR.branch,
              },
              prNumber: mostUrgentPR.number,
              title: mostUrgentPR.title,
              branch: mostUrgentPR.branch,
              failedChecks: failedChecks.map(c => c.name),
              suggestion: `CI is failing on PR #${mostUrgentPR.number}. This blocks all other work and must be fixed immediately.`,
            },
          ],
          allPRStatus: prsWithFailingCI,
        };
      }
    } catch (error) {
      // Log but don't fail if we can't check CI status
      automaticActions.push(`Unable to check CI status: ${error}`);
    }
    
    // Check for required actions and auto-enforce them first
    const workflowState = getWorkflowState();
    const autoActions = getRequiredActions('auto');
    
    if (autoActions.length > 0) {
      automaticActions.push(`Found ${autoActions.length} required auto-enforcement actions`);
      
      // Process auto-enforcement actions
      for (const action of autoActions) {
        try {
          switch (action.type) {
            case 'assign_issue_on_status_change':
              // This will be handled by workflow_update_issue
              break;
            case 'create_pr_when_commits_exist': {
              // Check if we're on a branch with commits but no PR
              const currentBranch = execSync('git branch --show-current', { encoding: 'utf8' }).trim();
              if (currentBranch !== 'main' && currentBranch !== 'master') {
                // This will be checked later in the normal flow
              }
              break;
            }
            default:
              automaticActions.push(`Unknown auto-action type: ${action.type}`);
          }
        } catch (error) {
          automaticActions.push(`Failed to auto-enforce ${action.type}: ${error}`);
        }
      }
    }
    
    // Get GitHub token from gh CLI
    const token = getGitHubToken();
    const octokit = new Octokit({ auth: token });

    // Get current user
    const currentUser = await getCurrentUser();
    automaticActions.push(`Identified current user: ${currentUser}`);

    // Get repository info from git remote
    const { owner: ownerName, repo: repoName } = getRepoInfo();
    const owner = { login: ownerName };
    const name = repoName;
    automaticActions.push(`Working in repository: ${owner.login}/${name}`);

    // Get current git status early as we need it for context
    const gitStatus = execSync('git status --porcelain', { encoding: 'utf8' });
    const currentBranch = execSync('git branch --show-current', { encoding: 'utf8' }).trim();
    const hasUncommittedChanges = gitStatus.length > 0;

    // Check if current branch has been merged
    if (currentBranch !== 'main') {
      const isMerged = await isBranchMerged(currentBranch);
      if (isMerged) {
        automaticActions.push(`Current branch '${currentBranch}' has been merged to main`);

        const suggestedActions = [];
        const issuesFound = [`Branch '${currentBranch}' has been merged - switch to main`];

        if (hasUncommittedChanges) {
          issuesFound.push(
            'You have uncommitted changes that need to be handled before switching branches'
          );
          suggestedActions.push(
            'Commit or stash your changes: git commit -am "Save work" OR git stash'
          );
        }

        suggestedActions.push(
          'Run: git checkout main && git pull origin main',
          'Then run workflow_next again to find next task'
        );

        return {
          requestedData: {
            context: {
              currentBranch,
              hasUncommittedChanges,
              branchMerged: true,
            },
          },
          automaticActions,
          issuesFound,
          suggestedActions,
          nextSteps: [
            {
              action: 'select_work',
              description: 'Switch to main branch and find next work',
              priority: 'high',
              category: 'immediate',
              tool: 'git_branch',
              parameters: {
                action: 'checkout',
                branch: 'main',
              },
              suggestion: `Your branch '${currentBranch}' has been merged. ${hasUncommittedChanges ? 'Commit or stash your changes, then s' : 'S'}witch to main and pull latest changes before starting new work.`,
              reason: 'Current branch has been merged to main',
            },
          ],
          allPRStatus: [],
        };
      }
    }

    // Parse issue number from branch name
    let branchIssueNumber: number | null = null;
    if (currentBranch !== 'main' && currentBranch !== 'master') {
      // Try to extract issue number from branch name
      // Common patterns: feature/some-feature-123, fix/issue-123, issue-123, etc.
      const issueNumberMatch = currentBranch.match(/-(\d+)(?:$|[^0-9])/);
      if (issueNumberMatch) {
        branchIssueNumber = parseInt(issueNumberMatch[1], 10);
      }
    }

    // First, check if there are any PRs needing attention
    const reviewStatus = await workflowMonitorReviews({
      includeApproved: true,
      includeDrafts: false,
    });
    const allOpenPRs = reviewStatus.requestedData.reviewsNeedingAttention;

    // Get all my open PRs (not just ones needing attention)
    const myOpenPRs = allOpenPRs.filter((pr) => pr.author === currentUser);

    // Categorize my PRs by what action is needed
    const myPRsNeedingAttention = myOpenPRs.filter(
      (pr) => pr.reviewStatus === 'changes_requested' || pr.reviewStatus === 'has_comments'
    );

    const myPRsWithoutReview = myOpenPRs.filter((pr) => pr.reviewStatus === 'pending_review');

    const myPRsApproved = myOpenPRs.filter((pr) => pr.reviewStatus === 'approved');

    const othersPRsToReview = allOpenPRs.filter((pr) => {
      if (pr.author === currentUser) return false;

      // Check if current user has already reviewed
      const myReview = pr.reviews.find((r) => r.reviewer === currentUser);

      if (!myReview) {
        // User hasn't reviewed yet
        return true;
      }

      // Check if there are new changes since user's review
      const myReviewDate = new Date(myReview.submittedAt);
      const prLastUpdated = new Date(pr.lastUpdated);

      // PR was updated after user's review
      return prLastUpdated > myReviewDate;
    });

    // Check if any of my PRs need Copilot re-review requested
    for (const pr of allOpenPRs.filter((p) => p.author === currentUser)) {
      // Get all Copilot reviews sorted by date
      const copilotReviews = pr.reviews
        .filter((r) => r.reviewer === COPILOT_BOT_REVIEWER || r.reviewer === COPILOT_HUMAN_REVIEWER)
        .sort((a, b) => new Date(b.submittedAt).getTime() - new Date(a.submittedAt).getTime());

      if (copilotReviews.length > 0) {
        const latestCopilotReview = copilotReviews[0];
        const reviewDate = new Date(latestCopilotReview.submittedAt);
        const prLastUpdated = new Date(pr.lastUpdated);

        // Check if PR was updated after Copilot's latest review
        if (prLastUpdated > reviewDate) {
          // Get commit count to see if there are actual new commits
          try {
            const { owner: ownerName, repo: repoName } = getRepoInfo();

            // Get commits since the review
            const commitsQuery = `
              query($owner: String!, $repo: String!, $prNumber: Int!, $since: GitTimestamp!) {
                repository(owner: $owner, name: $repo) {
                  pullRequest(number: $prNumber) {
                    commits(first: 1, since: $since) {
                      totalCount
                    }
                  }
                }
              }
            `;

            const result = await octokit.graphql(commitsQuery, {
              owner: ownerName,
              repo: repoName,
              prNumber: pr.prNumber,
              since: reviewDate.toISOString(),
            });

            interface CommitCountResult {
              repository: {
                pullRequest: {
                  commits: {
                    totalCount: number;
                  };
                };
              };
            }

            const newCommitCount = (result as CommitCountResult).repository.pullRequest.commits
              .totalCount;

            if (newCommitCount > 0) {
              // Only request re-review if there are actual new commits
              automaticActions.push(
                `PR #${pr.prNumber} has ${newCommitCount} new commits since Copilot's last review, requesting re-review`
              );
              const reReviewRequested = await requestCopilotReReview(pr.prNumber);
              if (reReviewRequested) {
                automaticActions.push(
                  `Successfully requested Copilot re-review for PR #${pr.prNumber}`
                );
              } else {
                issuesFound.push(`Failed to request Copilot re-review for PR #${pr.prNumber}`);
              }
            }
          } catch (error) {
            // If we can't check commits, skip re-review request
            console.error('Failed to check for new commits:', error);
          }
        }
      }
    }

    // Priority 1: My PRs with feedback that need addressing
    if (myPRsNeedingAttention.length > 0) {
      // Process PRs iteratively to find one with unresolved comments
      let prWithUnresolvedComments: PRReviewStatus | null = null;

      while (myPRsNeedingAttention.length > 0) {
        const pr = myPRsNeedingAttention[0];
        const summary = pr.commentSummary;
        const unresolvedComments = summary?.unresolved || 0;
        const totalComments = summary?.total || 0;

        if (unresolvedComments === 0 && totalComments > 0) {
          // All comments are resolved, skip this PR
          automaticActions.push(
            `PR #${pr.prNumber}: All ${totalComments} review comments have been addressed`
          );
          myPRsNeedingAttention.shift(); // Remove this PR from the list
        } else if (unresolvedComments > 0) {
          // Found a PR with unresolved comments
          prWithUnresolvedComments = pr;
          break;
        } else {
          // No comments at all, remove from list
          myPRsNeedingAttention.shift();
        }
      }

      if (prWithUnresolvedComments) {
        const pr = prWithUnresolvedComments;
        const summary = pr.commentSummary;
        const unresolvedComments = summary?.unresolved || 0;
        const resolvedComments = summary?.resolved || 0;
        const totalComments = summary?.total || 0;

        const commentStatus =
          totalComments > 0
            ? ` (${unresolvedComments} unresolved, ${resolvedComments} resolved)`
            : '';

        automaticActions.push(
          `Found PR #${pr.prNumber} authored by you with ${unresolvedComments} unresolved review comments`
        );

        if (resolvedComments > 0 && summary?.resolutionDetails) {
          automaticActions.push(`${resolvedComments} comments have been resolved:`);
          summary.resolutionDetails.forEach((detail) => {
            automaticActions.push(`  - ${detail}`);
          });
        }

        return {
          requestedData: {
            context: {
              currentBranch,
              hasUncommittedChanges,
              myOpenPRs,
              othersPRsToReview,
              totalOpenPRs: allOpenPRs.length,
            },
          },
          automaticActions,
          issuesFound,
          suggestedActions: [
            `Address ${unresolvedComments} unresolved ${pr.reviewStatus === 'changes_requested' ? 'requested changes' : 'review comments'} on PR #${pr.prNumber}`,
          ],
          nextSteps: [
            {
              action: 'address_pr_feedback',
              description: `Address ${unresolvedComments} unresolved review comments on PR #${pr.prNumber}`,
              priority: 'high',
              category: 'immediate',
              tool: 'workflow_monitor_reviews',
              parameters: {
                prNumber: pr.prNumber,
              },
              prNumber: pr.prNumber,
              title: pr.title,
              reviewStatus: pr.reviewStatus,
              reviews: pr.reviews,
              suggestion: `Address ${unresolvedComments} unresolved review comments on PR #${pr.prNumber}${commentStatus}`,
              prUrl: pr.url,
            },
          ],
          allPRStatus: [],
        };
      }
    }

    // Priority 2: My PRs without any reviews yet
    if (myPRsWithoutReview.length > 0) {
      const pr = myPRsWithoutReview[0];
      const hasCopilotReview = pr.reviews.some(
        (r) => r.reviewer === COPILOT_BOT_REVIEWER || r.reviewer === COPILOT_HUMAN_REVIEWER
      );

      automaticActions.push(`Found PR #${pr.prNumber} authored by you awaiting review`);

      return {
        requestedData: {
          context: {
            currentBranch,
            hasUncommittedChanges,
            myOpenPRs,
            othersPRsToReview,
            totalOpenPRs: allOpenPRs.length,
          },
        },
        nextSteps: [
          {
            action: 'work_on_todo', // Using existing action type that suggests waiting
            description: hasCopilotReview
              ? `PR #${pr.prNumber} is awaiting human review. Consider reviewing the PR yourself or wait for team review.`
              : `PR #${pr.prNumber} is awaiting review. Wait for Copilot and/or team review before proceeding.`,
            priority: 'high',
            category: 'immediate',
            prNumber: pr.prNumber,
            title: pr.title,
            reviewStatus: pr.reviewStatus,
            suggestion: hasCopilotReview
              ? `PR #${pr.prNumber} is awaiting human review. Consider reviewing the PR yourself or wait for team review.`
              : `PR #${pr.prNumber} is awaiting review. Wait for Copilot and/or team review before proceeding.`,
            prUrl: pr.url,
          },
        ],
        automaticActions,
        issuesFound: [`You have ${myPRsWithoutReview.length} PR(s) awaiting review`],
        suggestedActions: [
          `Wait for review on PR #${pr.prNumber}`,
          "Meanwhile, you could review other team members' PRs",
        ],
        allPRStatus: [],
      };
    }

    // Priority 3: Check for truly merge-ready PRs
    if (myPRsApproved.length > 0) {
      // Check merge readiness, short-circuiting when we find a ready PR
      let mergeReadyPR = null;
      const notYetReadyPRs = [];

      for (const pr of myPRsApproved) {
        const readiness = await checkPRMergeReadiness(octokit, owner.login, name, pr.prNumber, pr);
        if (readiness.isMergeReady) {
          mergeReadyPR = { pr, readiness };
          break;
        } else {
          notYetReadyPRs.push({ pr, readiness });
        }
      }

      // If we have a truly merge-ready PR, prioritize it
      if (mergeReadyPR) {
        const { pr, readiness } = mergeReadyPR;
        automaticActions.push(
          `Found PR #${pr.prNumber} authored by you that is fully ready to merge (approved, CI passing, no conflicts)`
        );

        return {
          requestedData: {
            context: {
              currentBranch,
              hasUncommittedChanges,
              myOpenPRs,
              othersPRsToReview,
              totalOpenPRs: allOpenPRs.length,
            },
          },
          nextSteps: [
            {
              action: 'merge_pr',
              description: `PR #${pr.prNumber} is fully ready to merge! All checks passed, approved, and no conflicts. Review and merge when ready.`,
              priority: 'urgent',
              category: 'immediate',
              prNumber: pr.prNumber,
              title: pr.title,
              reviewStatus: pr.reviewStatus,
              suggestion: `PR #${pr.prNumber} is fully ready to merge! All checks passed, approved, and no conflicts. Review and merge when ready.`,
              prUrl: pr.url,
              ciStatus: readiness.ciStatus,
              mergeable: readiness.mergeable,
              mergeableState: readiness.mergeableState,
            },
          ],
          automaticActions,
          issuesFound: [],
          suggestedActions: [`Merge PR #${pr.prNumber}: ${pr.url}`],
          allPRStatus: [],
        };
      }

      // If we have approved PRs that aren't fully ready, show what's blocking them
      if (notYetReadyPRs.length > 0) {
        const { pr, readiness } = notYetReadyPRs[0];
        const blockingReasons = readiness.blockingReasons.join(', ');

        automaticActions.push(
          `Found PR #${pr.prNumber} authored by you that is approved but not ready to merge: ${blockingReasons}`
        );

        return {
          requestedData: {
            context: {
              currentBranch,
              hasUncommittedChanges,
              myOpenPRs,
              othersPRsToReview,
              totalOpenPRs: allOpenPRs.length,
            },
          },
          nextSteps: [
            {
              action: 'work_on_todo',
              description: `PR #${pr.prNumber} is approved but cannot be merged yet: ${blockingReasons}`,
              priority: 'high',
              category: 'immediate',
              prNumber: pr.prNumber,
              title: pr.title,
              reviewStatus: pr.reviewStatus,
              suggestion: `PR #${pr.prNumber} is approved but cannot be merged yet: ${blockingReasons}`,
              prUrl: pr.url,
              ciStatus: readiness.ciStatus,
              mergeable: readiness.mergeable,
              mergeableState: readiness.mergeableState,
            },
          ],
          automaticActions,
          issuesFound: readiness.blockingReasons,
          suggestedActions: [`Address issues with PR #${pr.prNumber}: ${blockingReasons}`],
          allPRStatus: [],
        };
      }
    }

    // Priority 4: Others' PRs needing my review
    if (othersPRsToReview.length > 0) {
      const pr = othersPRsToReview[0];
      const hasReviewedBefore = pr.reviews.some((r) => r.reviewer === currentUser);

      automaticActions.push(`Found PR #${pr.prNumber} by ${pr.author} needing review`);

      return {
        requestedData: {
          context: {
            currentBranch,
            hasUncommittedChanges,
            myPRsNeedingAttention,
            othersPRsToReview,
          },
        },
        nextSteps: [
          {
            action: 'review_pr',
            description: hasReviewedBefore
              ? `Check new updates on PR #${pr.prNumber} by ${pr.author} since your last review`
              : `Review PR #${pr.prNumber} by ${pr.author}`,
            priority: 'medium',
            category: 'next_logical',
            prNumber: pr.prNumber,
            title: pr.title,
            author: pr.author,
            reviewStatus: pr.reviewStatus,
            suggestion: hasReviewedBefore
              ? `Check new updates on PR #${pr.prNumber} by ${pr.author} since your last review`
              : `Review PR #${pr.prNumber} by ${pr.author}`,
            prUrl: pr.url,
          },
        ],
        automaticActions,
        issuesFound,
        suggestedActions: [
          hasReviewedBefore
            ? `Check new updates on PR #${pr.prNumber}`
            : `Review PR #${pr.prNumber} by ${pr.author}`,
        ],
        allPRStatus: [],
      };
    }

    // If we reach here but there are still open PRs, block new work
    if (allOpenPRs.length > 0) {
      automaticActions.push(`Found ${allOpenPRs.length} total open PRs in the repository`);

      // Provide a summary of the PR status
      const prSummary = [];
      if (myOpenPRs.length > 0) {
        prSummary.push(`${myOpenPRs.length} of your PRs`);
      }
      const otherPRCount = allOpenPRs.length - myOpenPRs.length;
      if (otherPRCount > 0) {
        prSummary.push(`${otherPRCount} PRs by other team members`);
      }

      return {
        requestedData: {
          context: {
            currentBranch,
            hasUncommittedChanges,
            totalOpenPRs: allOpenPRs.length,
            myOpenPRs: [] as PRReviewStatus[],
            otherOpenPRs: otherPRCount,
          },
        },
        nextSteps: [
          {
            action: 'select_work',
            description: `Cannot suggest new work while there are ${allOpenPRs.length} open PRs (${prSummary.join(' and ')}). All PRs should be reviewed and merged before starting new work.`,
            priority: 'high',
            category: 'immediate',
            suggestion: `Cannot suggest new work while there are ${allOpenPRs.length} open PRs (${prSummary.join(' and ')}). All PRs should be reviewed and merged before starting new work.`,
            reason: 'Open PRs exist that need attention',
          },
        ],
        automaticActions,
        issuesFound: [`${allOpenPRs.length} open PRs blocking new work`],
        suggestedActions: [
          'Review the open PRs in your repository',
          'Help get PRs reviewed and merged before starting new work',
        ],
        allPRStatus: [],
      };
    }

    // Query project for issues assigned to current user
    const projectQuery = `
      query($owner: String!, $projectNumber: Int!) {
        user(login: $owner) {
          projectV2(number: $projectNumber) {
            items(first: 100) {
              nodes {
                id
                content {
                  ... on Issue {
                    number
                    title
                    body
                    state
                    labels(first: 10) {
                      nodes {
                        name
                      }
                    }
                    assignees(first: 10) {
                      nodes {
                        login
                      }
                    }
                  }
                }
                fieldValues(first: 20) {
                  nodes {
                    ... on ProjectV2ItemFieldSingleSelectValue {
                      name
                      field {
                        ... on ProjectV2SingleSelectField {
                          name
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    `;

    const projectData = await octokit.graphql(projectQuery, {
      owner: owner.login,
      projectNumber: config.github.projectNumber!,
    });

    interface ProjectItem {
      id: string;
      content?: {
        number: number;
        title: string;
        body?: string;
        state: string;
        labels?: {
          nodes: Array<{ name: string }>;
        };
        assignees?: {
          nodes: Array<{ login: string }>;
        };
      };
      fieldValues: {
        nodes: Array<{
          name?: string;
          field?: {
            name: string;
          };
        }>;
      };
    }

    interface ProjectData {
      user: {
        projectV2: {
          items: {
            nodes: ProjectItem[];
          };
        };
      };
    }

    // Filter to issues assigned to current user and in progress
    const items = (projectData as ProjectData).user.projectV2.items.nodes;

    // Separate regular issues and epics
    const allInProgressIssues = items.filter((item) => {
      if (!item.content || !item.content.assignees) return false;

      const isAssignedToUser = item.content.assignees.nodes.some(
        (assignee) => assignee.login === currentUser
      );

      const statusField = item.fieldValues.nodes.find(
        (field) => field.field && field.field.name === 'Status'
      );
      const isInProgress = statusField && statusField.name === 'In Progress';

      return isAssignedToUser && isInProgress;
    });

    const inProgressIssues = allInProgressIssues.filter((item) => {
      const isEpic =
        item.content?.labels && item.content.labels.nodes.some((label) => label.name === 'epic');
      return !isEpic;
    });

    const inProgressEpics = allInProgressIssues.filter((item) => {
      const isEpic =
        item.content?.labels && item.content.labels.nodes.some((label) => label.name === 'epic');
      return isEpic;
    });

    automaticActions.push(
      `Found ${inProgressIssues.length} issues assigned to ${currentUser} in progress`
    );

    // Check for existing PR on current branch using GraphQL
    let existingPR = null;
    try {
      const prQuery = `
        query($owner: String!, $repo: String!, $headRef: String!) {
          repository(owner: $owner, name: $repo) {
            pullRequests(headRefName: $headRef, states: [OPEN], first: 1) {
              nodes {
                number
                title
                state
              }
            }
          }
        }
      `;

      const prResult = await octokit.graphql(prQuery, {
        owner: owner.login,
        repo: name,
        headRef: currentBranch,
      });

      interface PRQueryResult {
        repository: {
          pullRequests: {
            nodes: Array<{
              number: number;
              title: string;
              state: string;
            }>;
          };
        };
      }

      const prs = (prResult as PRQueryResult).repository.pullRequests.nodes;
      if (prs.length > 0) {
        existingPR = {
          number: prs[0].number,
          title: prs[0].title,
        };
      }
    } catch {
      // No PR found or error checking
    }

    // Check if the branch corresponds to an in-progress issue without a PR
    if (branchIssueNumber && !existingPR) {
      // Check if this issue is in the in-progress list
      const branchIssue = allInProgressIssues.find(
        (item) => item.content?.number === branchIssueNumber
      );

      if (branchIssue) {
        automaticActions.push(
          `Detected issue #${branchIssueNumber} from branch name: ${currentBranch}`
        );
        automaticActions.push(
          `Current branch corresponds to in-progress issue #${branchIssueNumber} but no PR exists`
        );

        // Check if there are commits on this branch
        let hasCommits = false;
        try {
          // Detect the default branch
          let defaultBranch = 'origin/master'; // Fallback to origin/master
          try {
            const branchRef = execSync(`git symbolic-ref refs/remotes/origin/HEAD`, {
              encoding: 'utf8',
            }).trim();
            defaultBranch = branchRef.replace('refs/remotes/', '');
          } catch {
            // If detection fails, fallback to origin/master
          }
          const commitCount = execSync(`git rev-list --count ${defaultBranch}..HEAD`, {
            encoding: 'utf8',
          }).trim();
          hasCommits = parseInt(commitCount, 10) > 0;
        } catch {
          // If the command fails, assume we have commits (safer assumption)
          hasCommits = true;
        }

        if (hasCommits) {
          // Check enforcement policy for PR creation
          const enforcement = workflowState.enforcementPolicies.create_pr_when_commits_exist;
          
          if (enforcement === 'auto') {
            // Auto-create the PR
            automaticActions.push(`Auto-creating PR for issue #${branchIssueNumber} (enforcement: auto)`);
            
            try {
              // Import and call workflow_create_pr
              const { workflowCreatePR } = await import('./workflow-create-pr.js');
              const prResult = await workflowCreatePR({});
              
              automaticActions.push(`Successfully created PR #${prResult.requestedData.pr?.number}`);
              completeAction('create_pr_when_commits_exist');
              
              // Update workflow state
              updateWorkflowState({
                phase: 'pr_created',
                currentIssue: branchIssueNumber,
                currentBranch,
              });
              
              return {
                requestedData: {
                  context: {
                    currentBranch,
                    hasUncommittedChanges,
                    existingPR: prResult.requestedData.pr ? {
                      number: prResult.requestedData.pr.number,
                      title: prResult.requestedData.pr.title
                    } : null,
                  },
                },
                automaticActions,
                issuesFound: [],
                suggestedActions: [`Monitor PR #${prResult.requestedData.pr?.number} for reviews`],
                nextSteps: [
                  {
                    action: 'monitor_pr_reviews',
                    description: `Monitor PR #${prResult.requestedData.pr?.number} for review feedback`,
                    priority: 'high',
                    category: 'immediate',
                    tool: 'workflow_monitor_reviews',
                    prNumber: prResult.requestedData.pr?.number,
                    suggestion: `PR created automatically. Monitor for reviews.`,
                  },
                ],
                allPRStatus: [],
              };
            } catch (error) {
              automaticActions.push(`Failed to auto-create PR: ${error}`);
              // Fall through to manual suggestion
            }
          }
          
          // Manual suggestion (if auto failed or enforcement is not auto)
          const nextSteps: WorkflowNextStepAction[] = [
            {
              action: 'todos_complete',
              description: `Create a PR for issue #${branchIssueNumber} - work is complete`,
              priority: 'high',
              category: 'immediate',
              tool: 'workflow_create_pr',
              issueNumber: branchIssueNumber,
              title: branchIssue.content!.title,
              status: 'In Progress',
              suggestion: `You have commits for issue #${branchIssueNumber} on branch '${currentBranch}'. Create a PR before moving to the next issue.`,
            },
          ];

          return {
            requestedData: {
              context: {
                currentBranch,
                hasUncommittedChanges,
                existingPR: null,
              },
            },
            automaticActions,
            issuesFound: [`Branch '${currentBranch}' has commits but no PR`],
            suggestedActions: [`Create a PR for issue #${branchIssueNumber}`],
            nextSteps,
            allPRStatus: [],
          };
        }
      }
    }

    if (inProgressIssues.length === 0) {
      // No regular issues in progress, check for epics
      if (inProgressEpics.length > 0) {
        // Analyze the first epic
        const epic = inProgressEpics[0].content;
        if (!epic) {
          throw new Error('Epic content is undefined');
        }

        // Get all sub-issues linked to this epic
        const epicQuery = `
          query($owner: String!, $repo: String!, $epicNumber: Int!) {
            repository(owner: $owner, name: $repo) {
              issue(number: $epicNumber) {
                title
                body
                subIssues(first: 100) {
                  nodes {
                    number
                    title
                    state
                    labels(first: 10) {
                      nodes {
                        name
                      }
                    }
                  }
                }
              }
            }
          }
        `;

        try {
          const epicData = await octokit.graphql(epicQuery, {
            owner: owner.login,
            repo: name,
            epicNumber: epic.number,
          });

          interface SubIssue {
            number: number;
            title: string;
            state: string;
            labels?: {
              nodes: Array<{ name: string }>;
            };
          }

          interface EpicData {
            repository: {
              issue: {
                title: string;
                body?: string;
                subIssues?: {
                  nodes: SubIssue[];
                };
              };
            };
          }

          const epicIssue = (epicData as EpicData).repository.issue;
          const subIssues = epicIssue.subIssues?.nodes || [];
          const openSubIssues = subIssues.filter((issue) => issue.state === 'OPEN');

          if (openSubIssues.length === 0) {
            // Epic has no open sub-issues
            return {
              requestedData: {
                context: {
                  currentBranch,
                  hasUncommittedChanges,
                  existingPR: existingPR
                    ? { number: existingPR.number, title: existingPR.title }
                    : null,
                },
              },
              automaticActions,
              issuesFound,
              suggestedActions: [`Mark epic #${epic.number} as complete`],
              nextSteps: [
                {
                  action: 'complete_epic',
                  description: 'Mark epic as complete - all sub-issues are done',
                  priority: 'high',
                  category: 'immediate',
                  tool: 'workflow_update_issue',
                  parameters: {
                    issueNumber: epic.number,
                    status: 'done',
                  },
                  epicNumber: epic.number,
                  epicTitle: epic.title,
                  suggestion:
                    'All sub-issues for this epic are complete. Consider marking the epic as done.',
                },
              ],
              allPRStatus: [],
            };
          }

          // Epic has open sub-issues - request LLM decision if multiple options
          if (openSubIssues.length > 1) {
            // Multiple sub-issues available, request LLM decision
            const decisionId = `epic-${epic.number}-next-issue-${Date.now()}`;

            return {
              requestedData: {
                context: {
                  currentBranch,
                  hasUncommittedChanges,
                  existingPR: existingPR
                    ? { number: existingPR.number, title: existingPR.title }
                    : null,
                },
              },
              automaticActions,
              issuesFound,
              suggestedActions: ['Awaiting decision on which sub-issue to work on next'],
              nextSteps: [
                {
                  action: 'requires_llm_decision',
                  description: `Choose which sub-issue of epic "${epic.title}" to work on next`,
                  priority: 'high',
                  category: 'immediate',
                  tool: 'workflow_decide',
                  parameters: {
                    decisionId,
                  },
                  decisionType: 'select_next_issue',
                  decisionId,
                  epicNumber: epic.number,
                  epicTitle: epic.title,
                  choices: openSubIssues.map((issue) => ({
                    id: issue.number,
                    title: issue.title,
                    description: `Issue #${issue.number}`,
                    metadata: {
                      state: issue.state,
                      labels: issue.labels?.nodes?.map((l) => l.name) || [],
                    },
                  })),
                  decisionContext: {
                    prompt: `Which sub-issue of the epic "${epic.title}" should be worked on next? Consider dependencies, logical ordering, and which issues might be foundation work that enables others.`,
                    additionalInfo: {
                      currentBranch,
                      existingPR,
                    },
                  },
                },
              ],
              allPRStatus: [],
            };
          }

          // Only one sub-issue, suggest it directly
          const nextIssue = openSubIssues[0];

          return {
            requestedData: {
              context: {
                currentBranch,
                hasUncommittedChanges,
                existingPR: existingPR
                  ? { number: existingPR.number, title: existingPR.title }
                  : null,
              },
            },
            automaticActions,
            issuesFound,
            suggestedActions: [
              `Start work on issue #${nextIssue.number} from epic #${epic.number}`,
            ],
            nextSteps: [
              {
                action: 'epic_analysis',
                description: `Start work on sub-issue #${nextIssue.number}: ${nextIssue.title}`,
                priority: 'high',
                category: 'immediate',
                tool: 'workflow_update_issue',
                parameters: {
                  issueNumber: nextIssue.number,
                  status: 'in_progress',
                },
                epicNumber: epic.number,
                epicTitle: epic.title,
                suggestion: `Work on sub-issue #${nextIssue.number}: ${nextIssue.title}`,
                subIssues: openSubIssues.map((issue) => ({
                  number: issue.number,
                  title: issue.title,
                  status: issue.state,
                })),
              },
            ],
            allPRStatus: [],
          };
        } catch {
          // Fallback to searching for issues that mention the epic using GraphQL
          automaticActions.push('Primary query failed, using search fallback');

          const searchQuery = `
            query($query: String!) {
              search(query: $query, type: ISSUE, first: 100) {
                nodes {
                  ... on Issue {
                    number
                    title
                    state
                    repository {
                      name
                      owner {
                        login
                      }
                    }
                  }
                }
              }
            }
          `;

          const searchResult = await octokit.graphql(searchQuery, {
            query: `repo:${owner.login}/${name} is:issue is:open "#${epic.number}" in:body`,
          });

          interface SearchIssue {
            number: number;
            title: string;
            state: string;
            repository?: {
              name: string;
              owner: {
                login: string;
              };
            };
          }

          interface SearchResult {
            search: {
              nodes: SearchIssue[];
            };
          }

          const relatedIssues = ((searchResult as SearchResult).search.nodes || [])
            .filter(
              (issue) =>
                issue.repository?.owner?.login === owner.login && issue.repository?.name === name
            )
            .map((issue) => ({
              number: issue.number,
              title: issue.title,
              state: issue.state,
            }));

          if (relatedIssues.length === 0) {
            return {
              requestedData: {
                context: {
                  currentBranch,
                  hasUncommittedChanges,
                  existingPR: existingPR
                    ? { number: existingPR.number, title: existingPR.title }
                    : null,
                },
              },
              nextSteps: [
                {
                  action: 'complete_epic',
                  description: 'No open issues found for this epic. Consider marking it as done.',
                  priority: 'medium',
                  category: 'optional',
                  epicNumber: epic.number,
                  epicTitle: epic.title,
                  suggestion: 'No open issues found for this epic. Consider marking it as done.',
                },
              ],
              automaticActions,
              issuesFound,
              suggestedActions: [`Mark epic #${epic.number} as complete`],
              allPRStatus: [],
            };
          }

          const nextIssue = relatedIssues[0];
          return {
            requestedData: {
              context: {
                currentBranch,
                hasUncommittedChanges,
                existingPR: existingPR
                  ? { number: existingPR.number, title: existingPR.title }
                  : null,
              },
            },
            nextSteps: [
              {
                action: 'epic_analysis',
                description: `Work on sub-issue #${nextIssue.number}: ${nextIssue.title}`,
                priority: 'high',
                category: 'next_logical',
                epicNumber: epic.number,
                epicTitle: epic.title,
                suggestion: `Work on sub-issue #${nextIssue.number}: ${nextIssue.title}`,
                subIssues: relatedIssues.map((issue) => ({
                  number: issue.number,
                  title: issue.title,
                  status: issue.state,
                })),
              },
            ],
            automaticActions,
            issuesFound,
            suggestedActions: [
              `Start work on issue #${nextIssue.number} from epic #${epic.number}`,
            ],
            allPRStatus: [],
          };
        }
      }

      // No issues or epics in progress - search for available work
      automaticActions.push('No issues in progress - searching for available work');
      
      // Find unassigned issues with "Todo" status
      const availableIssues = items.filter((item) => {
        if (!item.content) return false;

        // Skip if it's an epic (epics should not be assigned directly)
        const isEpic =
          item.content?.labels && item.content.labels.nodes.some((label) => label.name === 'epic');
        if (isEpic) return false;

        // Check if unassigned (no assignees)
        const isUnassigned = !item.content.assignees || 
          item.content.assignees.nodes.length === 0;

        // Check if status is "Todo"
        const statusField = item.fieldValues.nodes.find(
          (field) => field.field && field.field.name === 'Status'
        );
        const isTodo = statusField && statusField.name === 'Todo';

        // Check if the issue is open
        const isOpen = item.content.state === 'OPEN';

        return isUnassigned && isTodo && isOpen;
      });

      if (availableIssues.length === 0) {
        // No available issues found
        return {
          requestedData: {
            context: {
              assignedIssues: 0,
              inProgressIssues: 0,
            },
          },
          automaticActions,
          issuesFound,
          suggestedActions: ['Visit the project board to select your next task'],
          nextSteps: [
            {
              action: 'select_work',
              description: 'Visit project board to select next task',
              priority: 'high',
              category: 'immediate',
              projectUrl: `https://github.com/users/${owner.login}/projects/9`,
              reason: 'No available issues found. Visit project board to create or assign work.',
            },
          ],
          allPRStatus: [],
        };
      }

      // Present available issues for selection
      if (availableIssues.length === 1) {
        // Only one available issue - suggest it directly
        const availableIssue = availableIssues[0].content!;
        automaticActions.push(`Found 1 available issue: #${availableIssue.number}`);

        return {
          requestedData: {
            context: {
              assignedIssues: 0,
              inProgressIssues: 0,
            },
          },
          automaticActions,
          issuesFound,
          suggestedActions: [`Start work on issue #${availableIssue.number}: ${availableIssue.title}`],
          nextSteps: [
            {
              action: 'start_new_work',
              description: `Start work on issue #${availableIssue.number}: ${availableIssue.title}`,
              priority: 'high',
              category: 'immediate',
              tool: 'git_branch',
              parameters: {
                action: 'start-work',
                issueNumber: availableIssue.number,
              },
              issueNumber: availableIssue.number,
              title: availableIssue.title,
              status: 'Todo',
              suggestion: `Start work on issue #${availableIssue.number}: ${availableIssue.title}`,
            },
          ],
          allPRStatus: [],
        };
      }

      // Multiple available issues - request LLM decision
      const decisionId = `select-available-work-${Date.now()}`;
      automaticActions.push(`Found ${availableIssues.length} available issues - requesting decision`);

      return {
        requestedData: {
          context: {
            assignedIssues: 0,
            inProgressIssues: 0,
          },
        },
        automaticActions,
        issuesFound,
        suggestedActions: ['Awaiting decision on which issue to work on next'],
        nextSteps: [
          {
            action: 'requires_llm_decision',
            description: `Choose which of ${availableIssues.length} available issues to work on next`,
            priority: 'high',
            category: 'immediate',
            tool: 'workflow_decide',
            parameters: {
              decisionId,
            },
            decisionType: 'select_next_issue',
            decisionId,
            choices: availableIssues.map((item) => ({
              id: item.content!.number,
              title: item.content!.title,
              description: `Issue #${item.content!.number}`,
              metadata: {
                state: item.content!.state,
                labels: item.content!.labels?.nodes?.map((l) => l.name) || [],
              },
            })),
            decisionContext: {
              prompt: `Which issue should be worked on next? Consider priorities, dependencies, and logical work ordering.`,
              additionalInfo: {
                currentBranch,
                existingPR: null,
              },
            },
          },
        ],
        allPRStatus: [],
      };
    }

    // Process the first in-progress issue
    const issue = inProgressIssues[0].content;
    if (!issue) {
      throw new Error('Issue content is undefined');
    }
    const todos = parseTodoItems(issue.body || '');
    const completedTodos = todos.filter((t) => t.checked).length;
    const nextTodo = todos.find((t) => !t.checked);

    if (!nextTodo) {
      // All todos complete
      const suggestion = existingPR
        ? `All todos complete. PR #${existingPR.number} exists - check if ready to merge.`
        : 'All todos complete. Create PR for the completed work.';

      return {
        requestedData: {
          context: {
            totalTodos: todos.length,
            completedTodos: todos.length,
            hasPR: !!existingPR,
            existingPR: existingPR ? { number: existingPR.number, title: existingPR.title } : null,
            currentBranch,
            hasUncommittedChanges,
          },
        },
        automaticActions,
        issuesFound,
        suggestedActions: existingPR
          ? [`Check PR #${existingPR.number} for review status`]
          : ['Create a pull request for the completed work'],
        nextSteps: [
          {
            action: 'todos_complete',
            description: existingPR
              ? 'Check if PR is ready to merge'
              : 'Create PR for completed work',
            priority: 'high',
            category: 'immediate',
            tool: existingPR ? 'workflow_monitor_reviews' : 'workflow_create_pr',
            parameters: existingPR ? { prNumber: existingPR.number } : {},
            issueNumber: issue.number,
            title: issue.title,
            status: 'In Progress',
            suggestion,
          },
        ],
        allPRStatus: [],
      };
    }

    // Return next todo to work on
    return {
      requestedData: {
        context: {
          currentBranch,
          hasUncommittedChanges,
          existingPR: existingPR ? { number: existingPR.number, title: existingPR.title } : null,
        },
      },
      automaticActions,
      issuesFound,
      suggestedActions: [`Work on: ${nextTodo.text}`],
      nextSteps: [
        {
          action: 'work_on_todo',
          description: `Work on todo: ${nextTodo.text}`,
          priority: 'high',
          category: 'immediate',
          issueNumber: issue.number,
          title: issue.title,
          status: 'In Progress',
          todoItem: nextTodo.text,
          todoIndex: nextTodo.index,
          totalTodos: todos.length,
          completedTodos,
        },
      ],
      allPRStatus: [],
    };
  } catch (error) {
    issuesFound.push(`Error: ${error instanceof Error ? error.message : String(error)}`);
    suggestedActions.push('Check that gh CLI is authenticated and has access to the repository');

    return {
      requestedData: {
        context: {},
      },
      automaticActions,
      issuesFound,
      suggestedActions,
      nextSteps: [],
      allPRStatus: [],
    };
  }
}
