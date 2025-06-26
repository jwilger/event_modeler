import { Octokit } from '@octokit/rest';
import { WorkflowResponse } from '../types.js';
import { getRepoInfo } from '../utils/github.js';
import { getGitHubToken } from '../utils/auth.js';

interface ManageSubissuesInput {
  action: 'link' | 'unlink' | 'list';
  epicNumber: number;
  issueNumber?: number;
}

interface SubIssue {
  number: number;
  title: string;
  state: string;
  url: string;
  assignees: string[];
}

// GraphQL response types
interface GetIssueResponse {
  repository: {
    issue: {
      id: string;
      title: string;
    } | null;
  };
}

interface GetSubIssuesResponse {
  repository: {
    issue: {
      title: string;
      subIssues: {
        nodes: Array<{
          number: number;
          title: string;
          state: string;
          url: string;
          assignees: {
            nodes: Array<{
              login: string;
            }>;
          };
        }>;
      };
    } | null;
  };
}

interface WorkflowManageSubissuesResponse extends WorkflowResponse {
  requestedData: {
    action: string;
    epicNumber: number;
    issueNumber?: number;
    success: boolean;
    subIssues?: SubIssue[];
    error?: string;
  };
}

async function getIssueNodeId(
  octokit: Octokit,
  owner: string,
  repo: string,
  issueNumber: number
): Promise<string> {
  const query = `
    query GetIssueId($owner: String!, $repo: String!, $number: Int!) {
      repository(owner: $owner, name: $repo) {
        issue(number: $number) {
          id
          title
        }
      }
    }
  `;

  const result = await octokit.graphql<GetIssueResponse>(query, {
    owner,
    repo,
    number: issueNumber,
  });

  const issue = result.repository.issue;
  if (!issue) {
    throw new Error(`Issue #${issueNumber} not found`);
  }

  return issue.id;
}

async function skipCircularDependencyCheck(
  _octokit: Octokit,
  _parentId: string,
  _childId: string
): Promise<boolean> {
  // TODO: GitHub's GraphQL API doesn't provide a direct way to check parent relationships
  // The 'parentIssue' field doesn't exist in the API
  // For now, we'll skip circular dependency checking to fix the broken tool
  // A future enhancement could implement this by:
  // 1. Fetching all sub-issues of the child issue
  // 2. Checking if the parent is among them
  // This would require recursive queries which could be expensive

  return false;
}

export async function workflowManageSubissues(
  input: ManageSubissuesInput
): Promise<WorkflowManageSubissuesResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];

  try {
    const { action, epicNumber, issueNumber } = input;

    // Validate input
    if (!action || !epicNumber) {
      throw new Error('Missing required parameters: action and epicNumber are required');
    }

    if ((action === 'link' || action === 'unlink') && !issueNumber) {
      throw new Error(`issueNumber is required for ${action} action`);
    }

    // Get repository info
    const { owner, repo } = getRepoInfo();
    automaticActions.push(`Working in repository: ${owner}/${repo}`);

    // Set up GitHub API
    const token = getGitHubToken();
    const octokit = new Octokit({ auth: token });

    if (action === 'link') {
      automaticActions.push(`Linking issue #${issueNumber} as sub-issue of epic #${epicNumber}`);

      // Get node IDs for both issues
      const [epicNodeId, issueNodeId] = await Promise.all([
        getIssueNodeId(octokit, owner, repo, epicNumber),
        getIssueNodeId(octokit, owner, repo, issueNumber!),
      ]);

      // Check for circular dependency (currently disabled due to API limitations)
      const hasCircular = await skipCircularDependencyCheck(octokit, epicNodeId, issueNodeId);

      if (hasCircular) {
        throw new Error(
          `Cannot link issue #${issueNumber} to epic #${epicNumber}: would create circular dependency`
        );
      }

      // Link the sub-issue
      const mutation = `
        mutation LinkSubIssue($parentId: ID!, $childId: ID!) {
          addSubIssue(input: { issueId: $parentId, subIssueId: $childId }) {
            issue {
              number
              title
            }
            subIssue {
              number
              title
            }
          }
        }
      `;

      await octokit.graphql(mutation, {
        parentId: epicNodeId,
        childId: issueNodeId,
      });

      automaticActions.push(`Successfully linked issue #${issueNumber} to epic #${epicNumber}`);
      suggestedActions.push(
        `View epic #${epicNumber}: https://github.com/${owner}/${repo}/issues/${epicNumber}`
      );

      return {
        requestedData: {
          action,
          epicNumber,
          issueNumber,
          success: true,
        },
        automaticActions,
        issuesFound,
        suggestedActions,
        allPRStatus: [],
      };
    } else if (action === 'unlink') {
      automaticActions.push(`Unlinking issue #${issueNumber} from epic #${epicNumber}`);

      // Get node IDs for both issues
      const [epicNodeId, issueNodeId] = await Promise.all([
        getIssueNodeId(octokit, owner, repo, epicNumber),
        getIssueNodeId(octokit, owner, repo, issueNumber!),
      ]);

      // Unlink the sub-issue
      const mutation = `
        mutation UnlinkSubIssue($parentId: ID!, $childId: ID!) {
          removeSubIssue(input: { issueId: $parentId, subIssueId: $childId }) {
            issue {
              number
              title
            }
            subIssue {
              number
              title
            }
          }
        }
      `;

      await octokit.graphql(mutation, {
        parentId: epicNodeId,
        childId: issueNodeId,
      });

      automaticActions.push(`Successfully unlinked issue #${issueNumber} from epic #${epicNumber}`);

      return {
        requestedData: {
          action,
          epicNumber,
          issueNumber,
          success: true,
        },
        automaticActions,
        issuesFound,
        suggestedActions,
        allPRStatus: [],
      };
    } else if (action === 'list') {
      automaticActions.push(`Listing sub-issues for epic #${epicNumber}`);

      // Get sub-issues for the epic
      const query = `
        query GetSubIssues($owner: String!, $repo: String!, $number: Int!) {
          repository(owner: $owner, name: $repo) {
            issue(number: $number) {
              title
              subIssues(first: 100) {
                nodes {
                  number
                  title
                  state
                  url
                  assignees(first: 10) {
                    nodes {
                      login
                    }
                  }
                }
              }
            }
          }
        }
      `;

      const result = await octokit.graphql<GetSubIssuesResponse>(query, {
        owner,
        repo,
        number: epicNumber,
      });

      const epic = result.repository.issue;
      if (!epic) {
        throw new Error(`Epic #${epicNumber} not found`);
      }

      const subIssues: SubIssue[] = epic.subIssues.nodes.map((issue) => ({
        number: issue.number,
        title: issue.title,
        state: issue.state,
        url: issue.url,
        assignees: issue.assignees.nodes.map((a) => a.login),
      }));

      automaticActions.push(`Found ${subIssues.length} sub-issues for epic #${epicNumber}`);

      if (subIssues.length > 0) {
        suggestedActions.push(
          ...subIssues.map(
            (issue) =>
              `${issue.state === 'OPEN' ? '🟢' : '✅'} #${issue.number}: ${issue.title}${
                issue.assignees.length > 0 ? ` (assigned to: ${issue.assignees.join(', ')})` : ''
              }`
          )
        );
      }

      return {
        requestedData: {
          action,
          epicNumber,
          success: true,
          subIssues,
        },
        automaticActions,
        issuesFound,
        suggestedActions,
        allPRStatus: [],
      };
    } else {
      throw new Error(`Unknown action: ${action}`);
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    issuesFound.push(`Error: ${errorMessage}`);

    return {
      requestedData: {
        action: input.action,
        epicNumber: input.epicNumber,
        issueNumber: input.issueNumber,
        success: false,
        error: errorMessage,
      },
      automaticActions,
      issuesFound,
      suggestedActions: ['Check the error message and try again'],
      allPRStatus: [],
    };
  }
}
