import { execSync } from 'child_process';
import { Octokit } from '@octokit/rest';
import { WorkflowResponse } from '../types.js';
import { getProjectConfig } from '../config.js';
import { getRepoInfo } from '../utils/github.js';

// GitHub bot IDs
const COPILOT_BOT_ID = 'BOT_kgDOCnlnWA';

interface MonitorReviewsInput {
  includeApproved?: boolean; // Include already approved PRs in response
  includeDrafts?: boolean;   // Include draft PRs in monitoring
}

export interface ReviewComment {
  file: string;
  line: number;
  comment: string;
  suggestion?: string;
}

export interface ReviewInfo {
  reviewer: string;
  type: 'approved' | 'changes_requested' | 'commented' | 'pending';
  submittedAt: string;
  comments: ReviewComment[];
}

interface PRReviewStatus {
  prNumber: number;
  title: string;
  author: string;
  isDraft: boolean;
  reviewStatus: 'approved' | 'changes_requested' | 'pending_review' | 'has_comments';
  reviews: ReviewInfo[];
  suggestedAction: string;
  url: string;
  lastUpdated: string;
}

interface WorkflowMonitorReviewsResponse extends WorkflowResponse {
  requestedData: {
    reviewsNeedingAttention: PRReviewStatus[];
    approvedPRs: number[];
    stalePRs: number[];
    summary: {
      totalOpenPRs: number;
      needingReview: number;
      changesRequested: number;
      approved: number;
    };
  };
}

function determineReviewStatus(reviews: ReviewInfo[]): 'approved' | 'changes_requested' | 'pending_review' | 'has_comments' {
  // Get the latest review from each reviewer
  const latestReviews = new Map<string, ReviewInfo>();
  
  for (const review of reviews) {
    const existing = latestReviews.get(review.reviewer);
    if (!existing || new Date(review.submittedAt) > new Date(existing.submittedAt)) {
      latestReviews.set(review.reviewer, review);
    }
  }

  // Check if any reviewer requested changes
  for (const review of latestReviews.values()) {
    if (review.type === 'changes_requested') {
      return 'changes_requested';
    }
  }

  // Check if we have any approvals (and no changes requested)
  const hasApprovals = Array.from(latestReviews.values()).some(r => r.type === 'approved');
  const hasComments = Array.from(latestReviews.values()).some(r => r.type === 'commented');

  if (hasApprovals && !hasComments) {
    return 'approved';
  } else if (hasComments) {
    return 'has_comments';
  }

  return 'pending_review';
}

function suggestAction(status: 'approved' | 'changes_requested' | 'pending_review' | 'has_comments', isDraft: boolean): string {
  if (isDraft) {
    return 'Mark PR as ready for review when complete';
  }

  switch (status) {
    case 'approved':
      return 'Ready to merge';
    case 'changes_requested':
      return 'Address review comments and push updates';
    case 'has_comments':
      return 'Review comments and respond if needed';
    case 'pending_review':
      return 'Waiting for review';
    default:
      return 'Monitor for updates';
  }
}

// Request re-review from Copilot after pushing changes
export async function requestCopilotReReview(prNumber: number): Promise<boolean> {
  try {
    const { owner, repo } = getRepoInfo();
    const token = execSync('gh auth token', { encoding: 'utf8' }).trim();
    const octokit = new Octokit({ auth: token });

    // First, get the PR node ID
    const prQuery = `
      query($owner: String!, $repo: String!, $prNumber: Int!) {
        repository(owner: $owner, name: $repo) {
          pullRequest(number: $prNumber) {
            id
          }
        }
      }
    `;

    const prResult = await octokit.graphql(prQuery, {
      owner,
      repo,
      prNumber
    });

    const prNodeId = (prResult as any).repository.pullRequest.id;

    // Request re-review from Copilot using the bot ID
    const requestReviewMutation = `
      mutation($pullRequestId: ID!, $botIds: [ID!]) {
        requestReviews(input: {
          pullRequestId: $pullRequestId,
          botIds: $botIds
        }) {
          pullRequest {
            id
            reviewRequests(first: 1) {
              nodes {
                requestedReviewer {
                  ... on Bot {
                    login
                  }
                }
              }
            }
          }
        }
      }
    `;

    await octokit.graphql(requestReviewMutation, {
      pullRequestId: prNodeId,
      botIds: [COPILOT_BOT_ID]
    });

    console.log('Requested Copilot re-review for PR #' + prNumber);
    return true;
  } catch (error) {
    console.error('Failed to request Copilot re-review:', error);
    return false;
  }
}

export async function workflowMonitorReviews(input: MonitorReviewsInput = {}): Promise<WorkflowMonitorReviewsResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];

  try {
    // Check configuration
    const { isComplete } = getProjectConfig();
    
    if (!isComplete) {
      throw new Error('Configuration is incomplete. Please run workflow_configure first.');
    }

    // Get repository info
    const { owner, repo } = getRepoInfo();

    // Set up GitHub API
    const token = execSync('gh auth token', { encoding: 'utf8' }).trim();
    const octokit = new Octokit({ auth: token });

    // Get all open PRs
    const prs = await octokit.pulls.list({
      owner,
      repo,
      state: 'open',
      per_page: 100
    });

    automaticActions.push(`Found ${prs.data.length} open PRs`);

    const reviewsNeedingAttention: PRReviewStatus[] = [];
    const approvedPRs: number[] = [];
    const stalePRs: number[] = [];

    // Process each PR
    for (const pr of prs.data) {
      if (!input.includeDrafts && pr.draft) {
        continue;
      }

      // Get reviews for this PR
      const reviews = await octokit.pulls.listReviews({
        owner,
        repo,
        pull_number: pr.number
      });

      // Get review comments
      const reviewComments = await octokit.pulls.listReviewComments({
        owner,
        repo,
        pull_number: pr.number
      });

      // Process reviews
      const reviewInfos: ReviewInfo[] = [];
      
      for (const review of reviews.data) {
        if (review.state === 'PENDING') continue;

        const reviewInfo: ReviewInfo = {
          reviewer: review.user?.login || 'unknown',
          type: review.state === 'APPROVED' ? 'approved' :
                review.state === 'CHANGES_REQUESTED' ? 'changes_requested' :
                'commented',
          submittedAt: review.submitted_at || '',
          comments: []
        };

        // Find comments associated with this review
        for (const comment of reviewComments.data) {
          if (comment.pull_request_review_id === review.id) {
            reviewInfo.comments.push({
              file: comment.path,
              line: comment.line || comment.original_line || 0,
              comment: comment.body,
              suggestion: undefined // GitHub API doesn't provide direct suggestion text
            });
          }
        }

        reviewInfos.push(reviewInfo);
      }

      // Determine overall status
      const reviewStatus = determineReviewStatus(reviewInfos);
      const prStatus: PRReviewStatus = {
        prNumber: pr.number,
        title: pr.title,
        author: pr.user?.login || 'unknown',
        isDraft: pr.draft || false,
        reviewStatus,
        reviews: reviewInfos,
        suggestedAction: suggestAction(reviewStatus, pr.draft || false),
        url: pr.html_url,
        lastUpdated: pr.updated_at
      };

      // Categorize PR
      if (reviewStatus === 'approved') {
        approvedPRs.push(pr.number);
        if (input.includeApproved) {
          reviewsNeedingAttention.push(prStatus);
        }
      } else if (reviewStatus === 'changes_requested' || reviewStatus === 'has_comments') {
        reviewsNeedingAttention.push(prStatus);
      } else if (reviewStatus === 'pending_review') {
        // Check if PR is stale (no activity for 7 days)
        const lastUpdated = new Date(pr.updated_at);
        const daysSinceUpdate = (Date.now() - lastUpdated.getTime()) / (1000 * 60 * 60 * 24);
        
        if (daysSinceUpdate > 7) {
          stalePRs.push(pr.number);
          prStatus.suggestedAction = 'PR is stale - consider pinging reviewers';
        }
        // Always include pending_review PRs so workflow_next can decide what to do
        reviewsNeedingAttention.push(prStatus);
      }
    }

    // Sort by priority: changes_requested first, then has_comments, then stale
    reviewsNeedingAttention.sort((a, b) => {
      const priority = { 'changes_requested': 0, 'has_comments': 1, 'pending_review': 2, 'approved': 3 };
      return (priority[a.reviewStatus] || 999) - (priority[b.reviewStatus] || 999);
    });

    // Generate summary
    const summary = {
      totalOpenPRs: prs.data.length,
      needingReview: reviewsNeedingAttention.filter(pr => pr.reviewStatus === 'pending_review').length,
      changesRequested: reviewsNeedingAttention.filter(pr => pr.reviewStatus === 'changes_requested').length,
      approved: approvedPRs.length
    };

    // Add suggested actions
    if (reviewsNeedingAttention.filter(pr => pr.reviewStatus === 'changes_requested').length > 0) {
      suggestedActions.push('Address requested changes in PRs');
    }
    if (reviewsNeedingAttention.filter(pr => pr.reviewStatus === 'has_comments').length > 0) {
      suggestedActions.push('Review and respond to PR comments');
    }
    if (stalePRs.length > 0) {
      suggestedActions.push('Follow up on stale PRs');
    }
    if (approvedPRs.length > 0) {
      suggestedActions.push('Merge approved PRs');
    }

    return {
      requestedData: {
        reviewsNeedingAttention,
        approvedPRs,
        stalePRs,
        summary
      },
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus: []
    };

  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    issuesFound.push(`Error: ${errorMessage}`);
    
    return {
      requestedData: {
        reviewsNeedingAttention: [],
        approvedPRs: [],
        stalePRs: [],
        summary: {
          totalOpenPRs: 0,
          needingReview: 0,
          changesRequested: 0,
          approved: 0
        }
      },
      automaticActions,
      issuesFound,
      suggestedActions: ['Fix the error and try again'],
      allPRStatus: []
    };
  }
}