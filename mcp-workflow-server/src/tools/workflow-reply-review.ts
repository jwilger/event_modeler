import { Octokit } from '@octokit/rest';
import { execSync } from 'child_process';
import { WorkflowResponse } from '../types.js';
import { getRepoInfo } from '../utils/github.js';

interface ReplyReviewInput {
  prNumber: number;
  commentId: number;
  body: string;
}

interface WorkflowReplyReviewResponse extends WorkflowResponse {
  requestedData: {
    replied: boolean;
    prNumber: number;
    commentId: number;
    replyUrl?: string;
    error?: string;
  };
}

export async function workflowReplyReview(
  input: ReplyReviewInput
): Promise<WorkflowReplyReviewResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];

  try {
    const { prNumber, commentId, body } = input;

    if (!prNumber || !commentId || !body) {
      throw new Error('Missing required parameters: prNumber, commentId, and body are required');
    }

    // Get repository info
    const { owner, repo } = getRepoInfo();
    automaticActions.push(`Working in repository: ${owner}/${repo}`);

    // Set up GitHub API
    const token = execSync('gh auth token', { encoding: 'utf8' }).trim();
    const octokit = new Octokit({ auth: token });

    // Reply to the review comment
    const response = await octokit.pulls.createReplyForReviewComment({
      owner,
      repo,
      pull_number: prNumber,
      comment_id: commentId,
      body,
    });

    automaticActions.push(`Successfully replied to review comment ${commentId} on PR #${prNumber}`);

    return {
      requestedData: {
        replied: true,
        prNumber,
        commentId,
        replyUrl: response.data.html_url,
      },
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus: [],
    };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Unknown error';
    issuesFound.push(`Failed to reply to review comment: ${errorMessage}`);

    return {
      requestedData: {
        replied: false,
        prNumber: input.prNumber || 0,
        commentId: input.commentId || 0,
        error: errorMessage,
      },
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus: [],
    };
  }
}

// Tool definition for MCP registration
export const workflowReplyReviewTool = {
  name: 'workflow_reply_review',
  description: 'Reply to a specific PR review comment to mark it as addressed',
  inputSchema: {
    type: 'object',
    properties: {
      prNumber: {
        type: 'number',
        description: 'Pull request number',
      },
      commentId: {
        type: 'number',
        description: 'Review comment ID to reply to',
      },
      body: {
        type: 'string',
        description: 'Reply message (e.g., "Fixed in commit abc123")',
      },
    },
    required: ['prNumber', 'commentId', 'body'],
  },
};
