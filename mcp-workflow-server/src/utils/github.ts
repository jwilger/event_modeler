import { Octokit } from '@octokit/rest';
import { execSync } from 'child_process';
import { PRStatus, CheckRunDetail } from '../types.js';
import { getGitHubToken } from './auth.js';

// Get repository info from git remote
export function getRepoInfo(): { owner: string; repo: string } {
  try {
    const remoteUrl = execSync('git config --get remote.origin.url', { encoding: 'utf-8' }).trim();

    // Parse GitHub URL (supports both HTTPS and SSH)
    const match = remoteUrl.match(/github\.com[:/]([^/]+)\/([^/]+?)(?:\.git)?$/);
    if (!match) {
      throw new Error('Not a GitHub repository');
    }

    return {
      owner: match[1],
      repo: match[2],
    };
  } catch {
    throw new Error('Failed to get repository info from git remote');
  }
}

let octokit: Octokit | null = null;

function getOctokit(): Octokit {
  if (!octokit) {
    const token = getGitHubToken();
    octokit = new Octokit({ auth: token });
  }
  return octokit;
}

export async function getAllPRs(): Promise<PRStatus[]> {
  try {
    const { owner, repo } = getRepoInfo();
    const octokit = getOctokit();

    // Get all open PRs
    const { data: pulls } = await octokit.pulls.list({
      owner,
      repo,
      state: 'open',
      per_page: 100,
    });

    // Get detailed status for each PR
    const prStatuses = await Promise.all(
      pulls.map(async (pr) => {
        // Get check runs
        const checks = { total: 0, passed: 0, failed: 0, pending: 0, details: [] as CheckRunDetail[] };
        try {
          const { data: checkRuns } = await octokit.checks.listForRef({
            owner,
            repo,
            ref: pr.head.sha,
            per_page: 100,
          });

          checks.total = checkRuns.total_count;
          checkRuns.check_runs.forEach((run) => {
            // Collect details for all check runs
            const detail: CheckRunDetail = {
              name: run.name,
              status: run.status as 'queued' | 'in_progress' | 'completed',
              conclusion: run.conclusion,
              url: run.html_url || undefined,
              output: run.output ? {
                title: run.output.title || undefined,
                summary: run.output.summary || undefined,
              } : undefined,
            };
            checks.details.push(detail);

            // Update counts
            if (run.status === 'completed') {
              if (run.conclusion === 'success') checks.passed++;
              else checks.failed++;
            } else {
              checks.pending++;
            }
          });
        } catch (error) {
          console.error(`Failed to retrieve check runs for PR with SHA ${pr.head.sha}:`, error);
          // If we can't get checks, leave them at 0
        }

        // Get reviews
        let hasUnresolvedReviews = false;
        try {
          const { data: reviews } = await octokit.pulls.listReviews({
            owner,
            repo,
            pull_number: pr.number,
          });

          hasUnresolvedReviews = reviews.some((review) => review.state === 'CHANGES_REQUESTED');
        } catch {
          // If we can't get reviews, assume false
        }

        // Also check for unresolved review threads (e.g., Copilot comments)
        try {
          const reviewThreadsQuery = `
            query($owner: String!, $repo: String!, $number: Int!) {
              repository(owner: $owner, name: $repo) {
                pullRequest(number: $number) {
                  reviewThreads(first: 100) {
                    nodes {
                      isResolved
                    }
                  }
                }
              }
            }
          `;

          const reviewThreadsData = await octokit.graphql(reviewThreadsQuery, {
            owner,
            repo,
            number: pr.number,
          });

          interface ReviewThreadsResult {
            repository: {
              pullRequest: {
                reviewThreads: {
                  nodes: Array<{ isResolved: boolean }>;
                };
              };
            };
          }

          const threads = (reviewThreadsData as ReviewThreadsResult).repository.pullRequest
            .reviewThreads.nodes;
          const hasUnresolvedThreads = threads.some((thread) => !thread.isResolved);

          // Set hasUnresolvedReviews to true if there are unresolved threads
          if (hasUnresolvedThreads) {
            hasUnresolvedReviews = true;
          }
        } catch {
          // If we can't get review threads, continue with existing value
        }

        // Check if PR needs rebase - we'll need to get detailed PR info for this
        let needsRebase = false;
        let isMergeable = false;

        try {
          const { data: prDetail } = await octokit.pulls.get({
            owner,
            repo,
            pull_number: pr.number,
          });

          needsRebase =
            prDetail.mergeable_state === 'behind' || prDetail.mergeable_state === 'dirty';
          isMergeable = prDetail.mergeable === true;
        } catch {
          // If we can't get PR details, use defaults
        }

        return {
          number: pr.number,
          title: pr.title,
          branch: pr.head.ref,
          baseRef: pr.base.ref,
          state: pr.state as 'open' | 'closed',
          isDraft: pr.draft || false,
          url: pr.html_url,
          checks,
          hasUnresolvedReviews,
          needsRebase,
          isMergeable,
        } satisfies PRStatus;
      })
    );

    return prStatuses;
  } catch (error) {
    throw new Error(
      `Failed to get PRs: ${error instanceof Error ? error.message : 'Unknown error'}`
    );
  }
}
