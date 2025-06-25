import { z } from "zod";
import { execSync } from "child_process";
import { Octokit } from "@octokit/rest";
import { getRepoInfo } from "../utils/github.js";
import type { WorkflowResponse } from "../types.js";

// GraphQL response types
interface GitHubUser {
  login: string;
  id: string;
  databaseId: number;
}

interface ReviewRequest {
  requestedReviewer: GitHubUser | null;
}

interface Review {
  author: GitHubUser | null;
  state: string;
}

interface ReviewComment {
  author: GitHubUser | null;
}

interface ReviewThread {
  comments: {
    nodes: ReviewComment[];
  };
}

interface PullRequestGraphQLResponse {
  repository: {
    pullRequest: {
      reviewRequests: {
        nodes: ReviewRequest[];
      };
      reviews: {
        nodes: Review[];
      };
      reviewThreads: {
        nodes: ReviewThread[];
      };
    };
  };
}

const RequestReviewInputSchema = z.object({
  prNumber: z.number().describe("Pull request number"),
  reviewers: z
    .array(z.string())
    .optional()
    .describe("Specific reviewers to request (optional - defaults to all previous reviewers)"),
  skipBots: z
    .boolean()
    .optional()
    .describe("Skip bot reviewers like Copilot (default: true)"),
});

type RequestReviewInput = z.infer<typeof RequestReviewInputSchema>;

// Known bot IDs that are auto-requested by repo rules
const COPILOT_BOT_ID = "BOT_kgDOCnlnWA";
const AUTO_REQUESTED_BOTS = [COPILOT_BOT_ID];

interface ReviewerInfo {
  login: string;
  id: string;
  isBot: boolean;
  nodeId: string;
}

export async function workflowRequestReview(
  input: RequestReviewInput
): Promise<WorkflowResponse> {
  const { prNumber, reviewers: specificReviewers, skipBots = true } = input;
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];

  try {
    // Get GitHub token
    const token = execSync("gh auth token", { encoding: "utf8" }).trim();
    const octokit = new Octokit({ auth: token });
    const { owner, repo } = getRepoInfo();

    automaticActions.push(`Working in repository: ${owner}/${repo}`);

    // Get PR details including previous reviewers
    const { data: pr } = await octokit.pulls.get({
      owner,
      repo,
      pull_number: prNumber,
    });

    if (pr.state !== "open") {
      issuesFound.push(`PR #${prNumber} is not open (state: ${pr.state})`);
      return {
        requestedData: null,
        automaticActions,
        issuesFound,
        suggestedActions,
        allPRStatus: [],
      };
    }

    // Get review history using GraphQL to identify all reviewers
    const query = `
      query($owner: String!, $repo: String!, $number: Int!) {
        repository(owner: $owner, name: $repo) {
          pullRequest(number: $number) {
            reviewRequests(first: 100) {
              nodes {
                requestedReviewer {
                  ... on User {
                    login
                    id
                    databaseId
                  }
                  ... on Bot {
                    login
                    id
                    databaseId
                  }
                }
              }
            }
            reviews(first: 100) {
              nodes {
                author {
                  login
                  ... on User {
                    id
                    databaseId
                  }
                  ... on Bot {
                    id
                    databaseId
                  }
                }
                state
              }
            }
            reviewThreads(first: 100) {
              nodes {
                comments(first: 100) {
                  nodes {
                    author {
                      login
                      ... on User {
                        id
                        databaseId
                      }
                      ... on Bot {
                        id
                        databaseId
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

    const graphqlResponse = await octokit.graphql<PullRequestGraphQLResponse>(query, {
      owner,
      repo,
      number: prNumber,
    });

    const pullRequest = graphqlResponse.repository.pullRequest;
    const allReviewers = new Map<string, ReviewerInfo>();

    // Collect reviewers from review requests
    pullRequest.reviewRequests.nodes.forEach((node) => {
      if (node.requestedReviewer) {
        const reviewer = node.requestedReviewer;
        const isBot = reviewer.id.startsWith("BOT_");
        allReviewers.set(reviewer.login, {
          login: reviewer.login,
          id: reviewer.id,
          isBot,
          nodeId: reviewer.id,
        });
      }
    });

    // Collect reviewers from actual reviews
    pullRequest.reviews.nodes.forEach((review) => {
      if (review.author) {
        const isBot = review.author.id.startsWith("BOT_");
        allReviewers.set(review.author.login, {
          login: review.author.login,
          id: review.author.id,
          isBot,
          nodeId: review.author.id,
        });
      }
    });

    // Collect reviewers from review comments
    pullRequest.reviewThreads.nodes.forEach((thread) => {
      thread.comments.nodes.forEach((comment) => {
        if (comment.author) {
          const isBot = comment.author.id.startsWith("BOT_");
          allReviewers.set(comment.author.login, {
            login: comment.author.login,
            id: comment.author.id,
            isBot,
            nodeId: comment.author.id,
          });
        }
      });
    });

    // Remove PR author from reviewers
    allReviewers.delete(pr.user?.login || "");

    automaticActions.push(
      `Found ${allReviewers.size} previous reviewers on PR #${prNumber}`
    );

    // Filter reviewers based on input
    let reviewersToRequest: ReviewerInfo[] = [];

    if (specificReviewers && specificReviewers.length > 0) {
      // Use specific reviewers if provided
      reviewersToRequest = specificReviewers
        .map((login) => allReviewers.get(login))
        .filter((r): r is ReviewerInfo => r !== undefined);

      const notFound = specificReviewers.filter(
        (login) => !allReviewers.has(login)
      );
      if (notFound.length > 0) {
        issuesFound.push(
          `These reviewers were not found in PR history: ${notFound.join(", ")}`
        );
      }
    } else {
      // Use all previous reviewers
      reviewersToRequest = Array.from(allReviewers.values());
    }

    // Filter out bots if requested
    if (skipBots) {
      const botsToSkip = reviewersToRequest.filter((r) => r.isBot);
      reviewersToRequest = reviewersToRequest.filter((r) => !r.isBot);
      
      if (botsToSkip.length > 0) {
        automaticActions.push(
          `Skipping bot reviewers: ${botsToSkip.map((b) => b.login).join(", ")}`
        );
      }
    }

    // Filter out auto-requested bots (like Copilot)
    reviewersToRequest = reviewersToRequest.filter(
      (r) => !AUTO_REQUESTED_BOTS.includes(r.nodeId)
    );

    if (reviewersToRequest.length === 0) {
      issuesFound.push("No reviewers to request after filtering");
      return {
        requestedData: {
          prNumber,
          reviewersRequested: [],
        },
        automaticActions,
        issuesFound,
        suggestedActions,
        allPRStatus: [],
      };
    }

    // Request reviews
    const requestedReviewers: string[] = [];
    const failedRequests: string[] = [];

    for (const reviewer of reviewersToRequest) {
      try {
        if (reviewer.isBot) {
          // Use GraphQL mutation for bot reviewers
          const mutation = `
            mutation($pullRequestId: ID!, $userIds: [ID!]) {
              requestReviews(input: {
                pullRequestId: $pullRequestId,
                userIds: $userIds
              }) {
                pullRequest {
                  id
                }
              }
            }
          `;

          await octokit.graphql(mutation, {
            pullRequestId: pr.node_id,
            userIds: [reviewer.nodeId],
          });
        } else {
          // Use REST API for human reviewers
          await octokit.pulls.requestReviewers({
            owner,
            repo,
            pull_number: prNumber,
            reviewers: [reviewer.login],
          });
        }
        requestedReviewers.push(reviewer.login);
        automaticActions.push(`Requested review from ${reviewer.login}`);
      } catch (error) {
        // Check if already requested or other common errors
        const errorMessage = error instanceof Error ? error.message : 'unknown error';
        if (errorMessage.includes("Review has already been requested")) {
          automaticActions.push(
            `${reviewer.login} already has a pending review request`
          );
          requestedReviewers.push(reviewer.login);
        } else if (errorMessage.includes("not a collaborator")) {
          failedRequests.push(`${reviewer.login} (not a collaborator)`);
        } else {
          failedRequests.push(
            `${reviewer.login} (${errorMessage})`
          );
        }
      }
    }

    if (failedRequests.length > 0) {
      issuesFound.push(
        `Failed to request reviews from: ${failedRequests.join(", ")}`
      );
    }

    if (requestedReviewers.length > 0) {
      suggestedActions.push(
        `Successfully requested ${requestedReviewers.length} review(s)`
      );
    }

    return {
      requestedData: {
        prNumber,
        reviewersRequested: requestedReviewers,
        reviewersFailed: failedRequests,
        prUrl: pr.html_url,
      },
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus: [],
    };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
    issuesFound.push(`Error: ${errorMessage}`);
    return {
      requestedData: null,
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus: [],
    };
  }
}

export const workflowRequestReviewTool = {
  name: "mcp__workflow__workflow_request_review",
  description:
    "Request re-review from specific reviewers or all previous reviewers on a pull request",
  inputSchema: RequestReviewInputSchema,
};