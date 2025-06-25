import { execSync } from 'child_process';
import { Octokit } from '@octokit/rest';
import { WorkflowResponse } from '../types.js';
import { getProjectConfig } from '../config.js';
import { getRepoInfo } from '../utils/github.js';

// GitHub bot IDs
const COPILOT_BOT_ID = 'BOT_kgDOCnlnWA';

// Resolution patterns to look for in PR author replies
const RESOLUTION_PATTERNS = [
  /fixed in (?:commit )?([a-f0-9]{7,})/i,
  /addressed in ([a-f0-9]{7,})/i,
  /resolved in ([a-f0-9]{7,})/i,
  /done in ([a-f0-9]{7,})/i,
  /fixed$/i,
  /addressed$/i,
  /resolved$/i,
  /done$/i,
  /\bfixed\b/i,
  /\baddressed\b/i,
  /\bresolved\b/i,
  /\bdone\b/i,
  /updated the code/i,
  /made the change/i,
  /applied the suggestion/i,
  /good catch/i,
  /thanks,? (?:i've |i have )?(?:fixed|addressed|updated)/i,
];

function isResolutionReply(body: string): boolean {
  return RESOLUTION_PATTERNS.some((pattern) => pattern.test(body));
}

interface MonitorReviewsInput {
  includeApproved?: boolean; // Include already approved PRs in response
  includeDrafts?: boolean; // Include draft PRs in monitoring
}

export interface ReviewComment {
  id: number;
  file: string;
  line: number;
  comment: string;
  suggestion?: string;
  hasReplies?: boolean;
  threadId?: string;
  isResolved?: boolean;
  resolutionReason?: string;
}

export interface ReviewInfo {
  reviewer: string;
  type: 'approved' | 'changes_requested' | 'commented' | 'pending';
  submittedAt: string;
  comments: ReviewComment[];
}

export interface PRReviewStatus {
  prNumber: number;
  title: string;
  author: string;
  isDraft: boolean;
  reviewStatus: 'approved' | 'changes_requested' | 'pending_review' | 'has_comments';
  reviews: ReviewInfo[];
  suggestedAction: string;
  url: string;
  lastUpdated: string;
  commentSummary?: {
    total: number;
    resolved: number;
    unresolved: number;
    resolutionDetails?: string[];
  };
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

function determineReviewStatus(
  reviews: ReviewInfo[]
): 'approved' | 'changes_requested' | 'pending_review' | 'has_comments' {
  // Get the latest review from each reviewer
  const latestReviews = new Map<string, ReviewInfo>();

  for (const review of reviews) {
    const existing = latestReviews.get(review.reviewer);
    if (!existing || new Date(review.submittedAt) > new Date(existing.submittedAt)) {
      latestReviews.set(review.reviewer, review);
    }
  }

  // Check if any reviewer requested changes with unresolved comments
  for (const review of latestReviews.values()) {
    if (review.type === 'changes_requested') {
      // Check if there are any unresolved comments
      const unresolvedComments = review.comments.filter((c) => !c.isResolved);
      if (unresolvedComments.length > 0) {
        return 'changes_requested';
      }
    }
  }

  // Check if we have any approvals (and no changes requested)
  const hasApprovals = Array.from(latestReviews.values()).some((r) => r.type === 'approved');

  // Check if there are any unresolved comments
  const hasUnresolvedComments = Array.from(latestReviews.values()).some(
    (r) => r.type === 'commented' && r.comments.some((c) => !c.isResolved)
  );

  if (hasApprovals && !hasUnresolvedComments) {
    return 'approved';
  } else if (hasUnresolvedComments) {
    return 'has_comments';
  }

  return 'pending_review';
}

function suggestAction(
  status: 'approved' | 'changes_requested' | 'pending_review' | 'has_comments',
  isDraft: boolean
): string {
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
      prNumber,
    });

    const prNodeId = (prResult as { repository: { pullRequest: { id: string } } }).repository
      .pullRequest.id;

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
      botIds: [COPILOT_BOT_ID],
    });

    return true;
  } catch (error) {
    console.error('Failed to request Copilot re-review:', error);
    return false;
  }
}

export async function workflowMonitorReviews(
  input: MonitorReviewsInput = {}
): Promise<WorkflowMonitorReviewsResponse> {
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
      per_page: 100,
    });

    automaticActions.push(`Found ${prs.data.length} open PRs`);

    const reviewsNeedingAttention: PRReviewStatus[] = [];
    const approvedPRs: number[] = [];
    const stalePRs: number[] = [];

    // Fetch reviews and comments for all PRs in parallel
    const reviewPromises = prs.data
      .map((pr) => {
        if (!input.includeDrafts && pr.draft) {
          return null; // Skip drafts if not included
        }
        return Promise.all([
          octokit.pulls.listReviews({
            owner,
            repo,
            pull_number: pr.number,
          }),
          // Use GraphQL to get review threads with reply information
          octokit.graphql(
            `
          query($owner: String!, $repo: String!, $prNumber: Int!) {
            repository(owner: $owner, name: $repo) {
              pullRequest(number: $prNumber) {
                reviewThreads(first: 100) {
                  nodes {
                    id
                    isResolved
                    isOutdated
                    path
                    line
                    comments(first: 100) {
                      nodes {
                        id
                        databaseId
                        author {
                          login
                        }
                        body
                        createdAt
                        pullRequestReview {
                          id
                          databaseId
                          state
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        `,
            {
              owner,
              repo,
              prNumber: pr.number,
            }
          ),
        ]).then(([reviews, reviewThreads]) => ({ pr, reviews, reviewThreads }));
      })
      .filter(Boolean); // Remove null values

    const reviewResults = await Promise.all(reviewPromises);

    // Process each PR's review data
    for (const result of reviewResults) {
      if (!result) continue;

      const { pr, reviews, reviewThreads } = result;

      // Process reviews
      const reviewInfos: ReviewInfo[] = [];

      // Type the GraphQL response
      interface ReviewThreadsResponse {
        repository: {
          pullRequest: {
            reviewThreads: {
              nodes: Array<{
                id: string;
                isResolved: boolean;
                isOutdated: boolean;
                path: string;
                line: number;
                comments: {
                  nodes: Array<{
                    id: string;
                    databaseId: number;
                    author: { login: string };
                    body: string;
                    createdAt: string;
                    pullRequestReview?: {
                      id: string;
                      databaseId: number;
                      state: string;
                    };
                  }>;
                };
              }>;
            };
          };
        };
      }

      const threads = (reviewThreads as ReviewThreadsResponse).repository.pullRequest.reviewThreads
        .nodes;

      // Build a map of review comments grouped by review ID
      const reviewCommentsMap = new Map<number, ReviewComment[]>();

      for (const thread of threads) {
        // Skip resolved or outdated threads
        if (thread.isResolved || thread.isOutdated) continue;

        // Check if thread has replies (more than one comment)
        const hasReplies = thread.comments.nodes.length > 1;

        // Get the first comment (the original review comment)
        const firstComment = thread.comments.nodes[0];
        if (firstComment && firstComment.pullRequestReview) {
          const reviewId = firstComment.pullRequestReview.databaseId;

          // Check if PR author has replied and if it indicates resolution
          let isResolved = false;
          let resolutionReason: string | undefined;

          if (hasReplies) {
            // Look through all replies to see if author responded
            const prAuthor = pr.user?.login;
            const authorReplies = thread.comments.nodes
              .slice(1) // Skip the first comment (the review comment)
              .filter((comment) => comment.author.login === prAuthor);

            if (authorReplies.length > 0) {
              // Check if any author reply indicates resolution
              for (const reply of authorReplies) {
                if (isResolutionReply(reply.body)) {
                  isResolved = true;
                  resolutionReason = `Author replied: "${reply.body.substring(0, 100)}${reply.body.length > 100 ? '...' : ''}"`;
                  break;
                }
              }

              // If author replied but didn't explicitly indicate resolution,
              // check if there are no further reviewer comments after the last author reply
              if (!isResolved && authorReplies.length > 0) {
                const lastAuthorReply = authorReplies[authorReplies.length - 1];
                const lastAuthorReplyDate = new Date(lastAuthorReply.createdAt);

                // Check if reviewer commented again after author's reply
                const reviewerCommentsAfterReply = thread.comments.nodes
                  .slice(1)
                  .filter(
                    (comment) =>
                      comment.author.login !== prAuthor &&
                      new Date(comment.createdAt) > lastAuthorReplyDate
                  );

                if (reviewerCommentsAfterReply.length === 0) {
                  // No further reviewer comments, consider it potentially resolved
                  isResolved = true;
                  resolutionReason = 'Author replied (no further reviewer comments)';
                }
              }
            }
          }

          const comment: ReviewComment = {
            id: firstComment.databaseId,
            file: thread.path,
            line: thread.line,
            comment: firstComment.body,
            hasReplies,
            threadId: thread.id,
            isResolved,
            resolutionReason,
          };

          if (!reviewCommentsMap.has(reviewId)) {
            reviewCommentsMap.set(reviewId, []);
          }
          reviewCommentsMap.get(reviewId)!.push(comment);
        }
      }

      for (const review of reviews.data) {
        if (review.state === 'PENDING') continue;

        const reviewInfo: ReviewInfo = {
          reviewer: review.user?.login || 'unknown',
          type:
            review.state === 'APPROVED'
              ? 'approved'
              : review.state === 'CHANGES_REQUESTED'
                ? 'changes_requested'
                : 'commented',
          submittedAt: review.submitted_at || '',
          comments: [],
        };

        // Add comments from this review that are in unresolved threads
        const reviewComments = reviewCommentsMap.get(review.id) || [];
        reviewInfo.comments.push(...reviewComments);

        reviewInfos.push(reviewInfo);
      }

      // Calculate comment summary
      let totalComments = 0;
      let resolvedComments = 0;
      let unresolvedComments = 0;
      const resolutionDetails: string[] = [];

      for (const review of reviewInfos) {
        for (const comment of review.comments) {
          totalComments++;
          if (comment.isResolved) {
            resolvedComments++;
            if (comment.resolutionReason) {
              resolutionDetails.push(
                `${comment.file}:${comment.line} - ${comment.resolutionReason}`
              );
            }
          } else {
            unresolvedComments++;
          }
        }
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
        lastUpdated: pr.updated_at,
        commentSummary:
          totalComments > 0
            ? {
                total: totalComments,
                resolved: resolvedComments,
                unresolved: unresolvedComments,
                resolutionDetails: resolutionDetails.length > 0 ? resolutionDetails : undefined,
              }
            : undefined,
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
      const priority = { changes_requested: 0, has_comments: 1, pending_review: 2, approved: 3 };
      return (priority[a.reviewStatus] || 999) - (priority[b.reviewStatus] || 999);
    });

    // Generate summary
    const summary = {
      totalOpenPRs: prs.data.length,
      needingReview: reviewsNeedingAttention.filter((pr) => pr.reviewStatus === 'pending_review')
        .length,
      changesRequested: reviewsNeedingAttention.filter(
        (pr) => pr.reviewStatus === 'changes_requested'
      ).length,
      approved: approvedPRs.length,
    };

    // Add suggested actions
    if (
      reviewsNeedingAttention.filter((pr) => pr.reviewStatus === 'changes_requested').length > 0
    ) {
      suggestedActions.push('Address requested changes in PRs');
    }
    if (reviewsNeedingAttention.filter((pr) => pr.reviewStatus === 'has_comments').length > 0) {
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
        summary,
      },
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus: [],
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
          approved: 0,
        },
      },
      automaticActions,
      issuesFound,
      suggestedActions: ['Fix the error and try again'],
      allPRStatus: [],
    };
  }
}
