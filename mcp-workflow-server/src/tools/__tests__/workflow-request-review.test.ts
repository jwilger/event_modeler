import { describe, it, expect, vi, beforeEach } from "vitest";
import { workflowRequestReview } from "../workflow-request-review.js";
import { execSync } from "child_process";
import { Octokit } from "@octokit/rest";

vi.mock("child_process");
vi.mock("@octokit/rest");
vi.mock("../../utils/github.js", () => ({
  getRepoInfo: (): { owner: string; repo: string } => ({ owner: "test-owner", repo: "test-repo" }),
}));

interface MockOctokit {
  pulls: {
    get: ReturnType<typeof vi.fn>;
    requestReviewers: ReturnType<typeof vi.fn>;
  };
  graphql: ReturnType<typeof vi.fn>;
}

describe("workflowRequestReview", () => {
  let mockOctokit: MockOctokit;
  let mockExecSync: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.clearAllMocks();
    mockExecSync = vi.mocked(execSync);
    mockExecSync.mockReturnValue("test-token");

    mockOctokit = {
      pulls: {
        get: vi.fn(),
        requestReviewers: vi.fn(),
      },
      graphql: vi.fn(),
    };
    vi.mocked(Octokit).mockImplementation(() => mockOctokit as unknown as Octokit);
  });

  it("should request reviews from all previous reviewers by default", async (): Promise<void> => {
    // Mock PR data
    mockOctokit.pulls.get.mockResolvedValue({
      data: {
        number: 123,
        state: "open",
        node_id: "PR_123",
        html_url: "https://github.com/test-owner/test-repo/pull/123",
        user: { login: "author" },
      },
    });

    // Mock GraphQL response with reviewers
    mockOctokit.graphql.mockResolvedValueOnce({
      repository: {
        pullRequest: {
          reviewRequests: {
            nodes: [
              {
                requestedReviewer: {
                  login: "reviewer1",
                  id: "USER_1",
                  databaseId: 1,
                },
              },
            ],
          },
          reviews: {
            nodes: [
              {
                author: {
                  login: "reviewer2",
                  id: "USER_2",
                  databaseId: 2,
                },
                state: "COMMENTED",
              },
            ],
          },
          reviewThreads: {
            nodes: [
              {
                comments: {
                  nodes: [
                    {
                      author: {
                        login: "reviewer3",
                        id: "USER_3",
                        databaseId: 3,
                      },
                    },
                  ],
                },
              },
            ],
          },
        },
      },
    });

    // Mock successful review requests
    mockOctokit.pulls.requestReviewers.mockResolvedValue({});

    const result = await workflowRequestReview({
      prNumber: 123,
    });

    expect(result.requestedData).toEqual({
      prNumber: 123,
      reviewersRequested: ["reviewer1", "reviewer2", "reviewer3"],
      reviewersFailed: [],
      prUrl: "https://github.com/test-owner/test-repo/pull/123",
    });
    expect(result.automaticActions).toContain(
      "Found 3 previous reviewers on PR #123"
    );
    expect(mockOctokit.pulls.requestReviewers).toHaveBeenCalledTimes(3);
  });

  it("should request specific reviewers when provided", async (): Promise<void> => {
    mockOctokit.pulls.get.mockResolvedValue({
      data: {
        number: 123,
        state: "open",
        node_id: "PR_123",
        html_url: "https://github.com/test-owner/test-repo/pull/123",
        user: { login: "author" },
      },
    });

    mockOctokit.graphql.mockResolvedValueOnce({
      repository: {
        pullRequest: {
          reviewRequests: { nodes: [] },
          reviews: {
            nodes: [
              {
                author: {
                  login: "reviewer1",
                  id: "USER_1",
                  databaseId: 1,
                },
                state: "COMMENTED",
              },
              {
                author: {
                  login: "reviewer2",
                  id: "USER_2",
                  databaseId: 2,
                },
                state: "COMMENTED",
              },
            ],
          },
          reviewThreads: { nodes: [] },
        },
      },
    });

    mockOctokit.pulls.requestReviewers.mockResolvedValue({});

    const result = await workflowRequestReview({
      prNumber: 123,
      reviewers: ["reviewer1"],
    });

    expect(result.requestedData).toEqual({
      prNumber: 123,
      reviewersRequested: ["reviewer1"],
      reviewersFailed: [],
      prUrl: "https://github.com/test-owner/test-repo/pull/123",
    });
    expect(mockOctokit.pulls.requestReviewers).toHaveBeenCalledTimes(1);
    expect(mockOctokit.pulls.requestReviewers).toHaveBeenCalledWith({
      owner: "test-owner",
      repo: "test-repo",
      pull_number: 123,
      reviewers: ["reviewer1"],
    });
  });

  it("should skip bot reviewers when skipBots is true", async (): Promise<void> => {
    mockOctokit.pulls.get.mockResolvedValue({
      data: {
        number: 123,
        state: "open",
        node_id: "PR_123",
        html_url: "https://github.com/test-owner/test-repo/pull/123",
        user: { login: "author" },
      },
    });

    mockOctokit.graphql.mockResolvedValueOnce({
      repository: {
        pullRequest: {
          reviewRequests: { nodes: [] },
          reviews: {
            nodes: [
              {
                author: {
                  login: "human-reviewer",
                  id: "USER_1",
                  databaseId: 1,
                },
                state: "COMMENTED",
              },
              {
                author: {
                  login: "bot-reviewer",
                  id: "BOT_123",
                  databaseId: 123,
                },
                state: "COMMENTED",
              },
            ],
          },
          reviewThreads: { nodes: [] },
        },
      },
    });

    mockOctokit.pulls.requestReviewers.mockResolvedValue({});

    const result = await workflowRequestReview({
      prNumber: 123,
      skipBots: true,
    });

    expect(result.requestedData).toEqual({
      prNumber: 123,
      reviewersRequested: ["human-reviewer"],
      reviewersFailed: [],
      prUrl: "https://github.com/test-owner/test-repo/pull/123",
    });
    expect(result.automaticActions).toContain(
      "Skipping bot reviewers: bot-reviewer"
    );
    expect(mockOctokit.pulls.requestReviewers).toHaveBeenCalledTimes(1);
  });

  it("should handle bot reviewers with GraphQL mutation", async (): Promise<void> => {
    mockOctokit.pulls.get.mockResolvedValue({
      data: {
        number: 123,
        state: "open",
        node_id: "PR_123",
        html_url: "https://github.com/test-owner/test-repo/pull/123",
        user: { login: "author" },
      },
    });

    mockOctokit.graphql
      .mockResolvedValueOnce({
        // First call - get reviewers
        repository: {
          pullRequest: {
            reviewRequests: { nodes: [] },
            reviews: {
              nodes: [
                {
                  author: {
                    login: "dependabot[bot]",
                    id: "BOT_456",
                    databaseId: 456,
                  },
                  state: "COMMENTED",
                },
              ],
            },
            reviewThreads: { nodes: [] },
          },
        },
      })
      .mockResolvedValueOnce({
        // Second call - request review mutation
        requestReviews: {
          pullRequest: {
            id: "PR_123",
          },
        },
      });

    const result = await workflowRequestReview({
      prNumber: 123,
      skipBots: false,
    });

    expect(result.requestedData).toEqual({
      prNumber: 123,
      reviewersRequested: ["dependabot[bot]"],
      reviewersFailed: [],
      prUrl: "https://github.com/test-owner/test-repo/pull/123",
    });
    expect(mockOctokit.graphql).toHaveBeenCalledTimes(2);
    // Check that GraphQL mutation was called
    expect(mockOctokit.graphql).toHaveBeenCalledWith(
      expect.stringContaining("mutation"),
      expect.objectContaining({
        pullRequestId: "PR_123",
        userIds: ["BOT_456"],
      })
    );
  });

  it("should handle closed PR error", async (): Promise<void> => {
    mockOctokit.pulls.get.mockResolvedValue({
      data: {
        number: 123,
        state: "closed",
        node_id: "PR_123",
        html_url: "https://github.com/test-owner/test-repo/pull/123",
        user: { login: "author" },
      },
    });

    const result = await workflowRequestReview({
      prNumber: 123,
    });

    expect(result.requestedData).toBeNull();
    expect(result.issuesFound).toContain(
      "PR #123 is not open (state: closed)"
    );
  });

  it("should handle reviewer not found", async (): Promise<void> => {
    mockOctokit.pulls.get.mockResolvedValue({
      data: {
        number: 123,
        state: "open",
        node_id: "PR_123",
        html_url: "https://github.com/test-owner/test-repo/pull/123",
        user: { login: "author" },
      },
    });

    mockOctokit.graphql.mockResolvedValueOnce({
      repository: {
        pullRequest: {
          reviewRequests: { nodes: [] },
          reviews: {
            nodes: [
              {
                author: {
                  login: "existing-reviewer",
                  id: "USER_1",
                  databaseId: 1,
                },
                state: "COMMENTED",
              },
            ],
          },
          reviewThreads: { nodes: [] },
        },
      },
    });

    const result = await workflowRequestReview({
      prNumber: 123,
      reviewers: ["non-existent-reviewer"],
    });

    expect(result.issuesFound).toContain(
      "These reviewers were not found in PR history: non-existent-reviewer"
    );
  });

  it("should handle already requested error gracefully", async (): Promise<void> => {
    mockOctokit.pulls.get.mockResolvedValue({
      data: {
        number: 123,
        state: "open",
        node_id: "PR_123",
        html_url: "https://github.com/test-owner/test-repo/pull/123",
        user: { login: "author" },
      },
    });

    mockOctokit.graphql.mockResolvedValueOnce({
      repository: {
        pullRequest: {
          reviewRequests: { nodes: [] },
          reviews: {
            nodes: [
              {
                author: {
                  login: "reviewer1",
                  id: "USER_1",
                  databaseId: 1,
                },
                state: "COMMENTED",
              },
            ],
          },
          reviewThreads: { nodes: [] },
        },
      },
    });

    mockOctokit.pulls.requestReviewers.mockRejectedValue(
      new Error("Review has already been requested from reviewer1")
    );

    const result = await workflowRequestReview({
      prNumber: 123,
    });

    expect(result.requestedData).toEqual({
      prNumber: 123,
      reviewersRequested: ["reviewer1"],
      reviewersFailed: [],
      prUrl: "https://github.com/test-owner/test-repo/pull/123",
    });
    expect(result.automaticActions).toContain(
      "reviewer1 already has a pending review request"
    );
  });

  it("should filter out PR author from reviewers", async (): Promise<void> => {
    mockOctokit.pulls.get.mockResolvedValue({
      data: {
        number: 123,
        state: "open",
        node_id: "PR_123",
        html_url: "https://github.com/test-owner/test-repo/pull/123",
        user: { login: "pr-author" },
      },
    });

    mockOctokit.graphql.mockResolvedValueOnce({
      repository: {
        pullRequest: {
          reviewRequests: { nodes: [] },
          reviews: {
            nodes: [
              {
                author: {
                  login: "pr-author",
                  id: "USER_1",
                  databaseId: 1,
                },
                state: "COMMENTED",
              },
              {
                author: {
                  login: "other-reviewer",
                  id: "USER_2", 
                  databaseId: 2,
                },
                state: "COMMENTED",
              },
            ],
          },
          reviewThreads: { nodes: [] },
        },
      },
    });

    mockOctokit.pulls.requestReviewers.mockResolvedValue({});

    const result = await workflowRequestReview({
      prNumber: 123,
    });

    expect(result.requestedData?.reviewersRequested).toEqual([
      "other-reviewer",
    ]);
    expect(mockOctokit.pulls.requestReviewers).toHaveBeenCalledTimes(1);
    expect(mockOctokit.pulls.requestReviewers).toHaveBeenCalledWith({
      owner: "test-owner",
      repo: "test-repo",
      pull_number: 123,
      reviewers: ["other-reviewer"],
    });
  });
});