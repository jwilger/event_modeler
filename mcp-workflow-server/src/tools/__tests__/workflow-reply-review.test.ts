import { describe, it, expect, beforeEach, vi } from 'vitest';
import { workflowReplyReview } from '../workflow-reply-review.js';
import { execSync } from 'child_process';
import { Octokit } from '@octokit/rest';
import * as github from '../../utils/github.js';

// Mock modules
vi.mock('child_process');
vi.mock('@octokit/rest');
vi.mock('../../utils/github.js');

// Mock auth
vi.mock('../../utils/auth.js', () => ({
  getGitHubToken: vi.fn(() => 'test-token'),
}));

const mockExecSync = vi.mocked(execSync);
const mockOctokit = vi.mocked(Octokit);
const mockGetRepoInfo = vi.mocked(github.getRepoInfo);

describe('workflowReplyReview', () => {
  let mockOctokitInstance: {
    pulls: {
      createReplyForReviewComment: ReturnType<typeof vi.fn>;
    };
  };

  beforeEach(() => {
    vi.clearAllMocks();

    // Setup default mocks
    mockOctokitInstance = {
      pulls: {
        createReplyForReviewComment: vi.fn()
      }
    };

    mockOctokit.mockImplementation(() => mockOctokitInstance as unknown as Octokit);

    mockGetRepoInfo.mockReturnValue({
      owner: 'testowner',
      repo: 'testrepo'
    });

    mockExecSync.mockImplementation(() => {
      return '';
    });
  });

  it('should successfully reply to a review comment', async () => {
    const mockResponse = {
      data: {
        id: 123456,
        html_url: 'https://github.com/testowner/testrepo/pull/100#discussion_r123456',
        body: 'Fixed in commit abc123'
      }
    };

    mockOctokitInstance.pulls.createReplyForReviewComment.mockResolvedValue(mockResponse);

    const result = await workflowReplyReview({
      prNumber: 100,
      commentId: 2164153964,
      body: 'Fixed in commit abc123'
    });

    expect(result.requestedData.replied).toBe(true);
    expect(result.requestedData.prNumber).toBe(100);
    expect(result.requestedData.commentId).toBe(2164153964);
    expect(result.requestedData.replyUrl).toBe('https://github.com/testowner/testrepo/pull/100#discussion_r123456');
    expect(result.automaticActions).toContain('Successfully replied to review comment 2164153964 on PR #100');

    expect(mockOctokitInstance.pulls.createReplyForReviewComment).toHaveBeenCalledWith({
      owner: 'testowner',
      repo: 'testrepo',
      pull_number: 100,
      comment_id: 2164153964,
      body: 'Fixed in commit abc123'
    });
  });

  it('should handle missing parameters', async () => {
    const result = await workflowReplyReview({
      prNumber: 0,
      commentId: 0,
      body: ''
    });

    expect(result.requestedData.replied).toBe(false);
    expect(result.requestedData.error).toContain('Missing required parameters');
    expect(result.issuesFound).toHaveLength(1);
  });

  it('should handle API errors gracefully', async () => {
    mockOctokitInstance.pulls.createReplyForReviewComment.mockRejectedValue(
      new Error('Not Found')
    );

    const result = await workflowReplyReview({
      prNumber: 100,
      commentId: 999999,
      body: 'This comment does not exist'
    });

    expect(result.requestedData.replied).toBe(false);
    expect(result.requestedData.error).toBe('Not Found');
    expect(result.issuesFound).toContain('Failed to reply to review comment: Not Found');
  });

  it('should handle network errors', async () => {
    mockOctokitInstance.pulls.createReplyForReviewComment.mockRejectedValue(
      new Error('Network error')
    );

    const result = await workflowReplyReview({
      prNumber: 100,
      commentId: 2164153964,
      body: 'Reply attempt'
    });

    expect(result.requestedData.replied).toBe(false);
    expect(result.requestedData.error).toBe('Network error');
  });
});