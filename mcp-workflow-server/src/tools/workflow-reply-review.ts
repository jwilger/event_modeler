import { Octokit } from '@octokit/rest';
import { WorkflowResponse } from '../types.js';
import { getRepoInfo } from '../utils/github.js';
import { getGitHubToken } from '../utils/auth.js';

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
    const token = getGitHubToken();
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
    
    // Provide specific guidance for common errors
    if (errorMessage.includes('Parent comment not found') || errorMessage.includes('404')) {
      suggestedActions.push(
        'The comment ID appears to be invalid. To fix this:',
        '1. Use workflow_monitor_reviews to get the correct comment IDs',
        '2. Look for the "id" field in the "comments" array of each review',
        '3. Use that exact comment ID when calling workflow_reply_review',
        `4. The comment ID you tried (${input.commentId}) may be incorrect or the comment may have been deleted`
      );
      issuesFound.push('Comment ID not found - please verify using workflow_monitor_reviews first');
    } else if (errorMessage.includes('422')) {
      suggestedActions.push(
        'The comment cannot be replied to. Possible reasons:',
        '1. The comment is already a reply (not a top-level review comment)',
        '2. The PR or comment thread may be locked',
        '3. The comment may be on an outdated version of the code'
      );
    }

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
