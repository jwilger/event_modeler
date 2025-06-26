import { execSync } from 'child_process';
import { Octokit } from '@octokit/rest';
import { WorkflowResponse } from '../types.js';
import { getRepoInfo } from '../utils/github.js';
import { getGitHubToken } from '../utils/auth.js';
import { updateWorkflowState } from '../config.js';

interface GitBranchInput {
  action: 'checkout' | 'create' | 'pull' | 'push' | 'list' | 'start-work';
  branch?: string;
  issueNumber?: number;
  force?: boolean;
}

interface BranchInfo {
  name: string;
  current: boolean;
  remote: boolean;
  ahead?: number;
  behind?: number;
}

interface GitBranchResponse extends WorkflowResponse {
  requestedData: {
    branches?: BranchInfo[];
    currentBranch?: string;
    previousBranch?: string;
    createdBranch?: string;
    pushedBranch?: string;
    issueDetails?: {
      number: number;
      title: string;
      url: string;
    };
  };
}

function hasUncommittedChanges(): boolean {
  try {
    const status = execSync('git status --porcelain', { encoding: 'utf8' });
    return status.trim().length > 0;
  } catch {
    return false;
  }
}

function getCurrentBranch(): string {
  try {
    return execSync('git branch --show-current', { encoding: 'utf8' }).trim();
  } catch {
    throw new Error('Failed to get current branch');
  }
}

function branchExists(branchName: string, remote: boolean = false): boolean {
  try {
    if (remote) {
      execSync(`git ls-remote --heads origin ${branchName}`, { encoding: 'utf8' });
    } else {
      execSync(`git rev-parse --verify ${branchName}`, { encoding: 'utf8' });
    }
    return true;
  } catch {
    return false;
  }
}

// Maximum length for the title portion of branch names
const MAX_BRANCH_TITLE_LENGTH = 50;

function createBranchNameFromIssue(issueTitle: string, issueNumber: number): string {
  // Convert title to branch-friendly format
  const cleanTitle = issueTitle
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, '') // Remove special characters
    .replace(/\s+/g, '-') // Replace spaces with hyphens
    .replace(/-+/g, '-') // Replace multiple hyphens with single
    .trim();

  // Truncate if too long (git has branch name limits)
  // Account for "feature/" prefix and "-{issueNumber}"
  const prefix = 'feature/';
  const suffix = `-${issueNumber}`;
  const maxTitleLength = MAX_BRANCH_TITLE_LENGTH - prefix.length - suffix.length;

  const truncatedTitle =
    cleanTitle.length > maxTitleLength
      ? cleanTitle.substring(0, maxTitleLength).replace(/-$/, '')
      : cleanTitle;

  return `${prefix}${truncatedTitle}${suffix}`;
}

async function getIssueDetails(issueNumber: number): Promise<{ title: string; url: string }> {
  try {
    const token = getGitHubToken();
    const octokit = new Octokit({ auth: token });
    const { owner, repo } = getRepoInfo();

    const { data: issue } = await octokit.issues.get({
      owner,
      repo,
      issue_number: issueNumber,
    });

    return {
      title: issue.title,
      url: issue.html_url,
    };
  } catch (error) {
    throw new Error(
      `Failed to get issue #${issueNumber}: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

export async function gitBranch(input: GitBranchInput): Promise<GitBranchResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];

  try {
    const currentBranch = getCurrentBranch();
    automaticActions.push(`Current branch: ${currentBranch}`);

    switch (input.action) {
      case 'checkout': {
        if (!input.branch) {
          throw new Error('Branch name is required for checkout');
        }

        // Check for uncommitted changes
        if (hasUncommittedChanges() && !input.force) {
          issuesFound.push('You have uncommitted changes');
          suggestedActions.push(
            'Commit your changes: use git_commit tool with action: "commit"',
            'Or stash them: use git_stash tool with action: "save"',
            'Or force checkout with force: true'
          );

          return {
            requestedData: {},
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: [],
          };
        }

        // Check if branch exists
        if (!branchExists(input.branch)) {
          // Try to fetch it from remote
          try {
            execSync(`git fetch origin ${input.branch}:${input.branch}`, { encoding: 'utf8' });
            automaticActions.push(`Fetched branch ${input.branch} from remote`);
          } catch {
            throw new Error(`Branch '${input.branch}' does not exist locally or remotely`);
          }
        }

        // Checkout the branch
        execSync(`git checkout ${input.branch}`, { encoding: 'utf8' });
        automaticActions.push(`Switched to branch '${input.branch}'`);

        // Pull latest changes if it tracks a remote branch
        try {
          const tracking = execSync(`git rev-parse --abbrev-ref ${input.branch}@{upstream}`, {
            encoding: 'utf8',
          }).trim();
          if (tracking) {
            execSync('git pull', { encoding: 'utf8' });
            automaticActions.push('Pulled latest changes from remote');
          }
        } catch {
          // Branch doesn't track a remote, that's ok
        }

        return {
          requestedData: {
            currentBranch: input.branch,
            previousBranch: currentBranch,
          },
          automaticActions,
          issuesFound,
          suggestedActions,
          allPRStatus: [],
        };
      }

      case 'create': {
        if (!input.branch && !input.issueNumber) {
          throw new Error('Either branch name or issue number is required for create');
        }

        let branchName = input.branch;
        let issueDetails;

        // If issue number provided, create branch name from issue
        if (input.issueNumber) {
          issueDetails = await getIssueDetails(input.issueNumber);
          branchName = createBranchNameFromIssue(issueDetails.title, input.issueNumber);
          automaticActions.push(
            `Creating branch for issue #${input.issueNumber}: ${issueDetails.title}`
          );
        }

        if (!branchName) {
          throw new Error('Failed to determine branch name');
        }

        // Check if branch already exists
        if (branchExists(branchName)) {
          issuesFound.push(`Branch '${branchName}' already exists`);
          suggestedActions.push(`Checkout existing branch: git checkout ${branchName}`);

          return {
            requestedData: {},
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: [],
          };
        }

        // Ensure we're on main/master and up to date
        const mainBranch = branchExists('main') ? 'main' : 'master';
        if (currentBranch !== mainBranch) {
          execSync(`git checkout ${mainBranch}`, { encoding: 'utf8' });
          automaticActions.push(`Switched to ${mainBranch}`);
        }

        execSync('git pull', { encoding: 'utf8' });
        automaticActions.push('Updated base branch');

        // Create and checkout new branch
        execSync(`git checkout -b ${branchName}`, { encoding: 'utf8' });
        automaticActions.push(`Created and switched to new branch: ${branchName}`);

        return {
          requestedData: {
            createdBranch: branchName,
            currentBranch: branchName,
            previousBranch: mainBranch,
            issueDetails: issueDetails
              ? {
                  number: input.issueNumber!,
                  title: issueDetails.title,
                  url: issueDetails.url,
                }
              : undefined,
          },
          automaticActions,
          issuesFound,
          suggestedActions: issueDetails
            ? [`Start working on issue #${input.issueNumber}`]
            : [`Branch '${branchName}' created and ready`],
          allPRStatus: [],
        };
      }

      case 'pull': {
        // Pull on current branch
        try {
          const result = execSync('git pull', { encoding: 'utf8' });
          automaticActions.push('Pulled latest changes');

          if (result.includes('Already up to date')) {
            automaticActions.push('Branch is already up to date');
          }
        } catch (error) {
          throw new Error(
            `Failed to pull: ${error instanceof Error ? error.message : String(error)}`
          );
        }

        return {
          requestedData: {
            currentBranch,
          },
          automaticActions,
          issuesFound,
          suggestedActions,
          allPRStatus: [],
        };
      }

      case 'push': {
        const branch = input.branch || currentBranch;

        try {
          // Check if branch has upstream
          try {
            execSync(`git rev-parse --abbrev-ref ${branch}@{upstream}`, { encoding: 'utf8' });
            // Has upstream, regular push
            execSync(`git push`, { encoding: 'utf8' });
          } catch {
            // No upstream, push with -u
            execSync(`git push -u origin ${branch}`, { encoding: 'utf8' });
            automaticActions.push(`Set upstream branch to origin/${branch}`);
          }

          automaticActions.push(`Pushed branch '${branch}' to remote`);
        } catch (error) {
          throw new Error(
            `Failed to push: ${error instanceof Error ? error.message : String(error)}`
          );
        }

        return {
          requestedData: {
            pushedBranch: branch,
            currentBranch,
          },
          automaticActions,
          issuesFound,
          suggestedActions: [`Branch '${branch}' pushed to remote`],
          allPRStatus: [],
        };
      }

      case 'list': {
        // Get all branches with their remote tracking info
        const branchOutput = execSync('git branch -vv', { encoding: 'utf8' });
        const branches: BranchInfo[] = [];

        for (const line of branchOutput.split('\n')) {
          if (!line.trim()) continue;

          const current = line.startsWith('*');
          const parts = line.substring(2).trim().split(/\s+/);
          const name = parts[0];

          // Check if branch has remote tracking
          const hasRemote = line.includes('[origin/');

          // Parse ahead/behind separately
          let ahead: number | undefined;
          let behind: number | undefined;

          const aheadMatch = line.match(/ahead (\d+)/);
          if (aheadMatch) {
            ahead = parseInt(aheadMatch[1]);
          }

          const behindMatch = line.match(/behind (\d+)/);
          if (behindMatch) {
            behind = parseInt(behindMatch[1]);
          }

          branches.push({
            name,
            current,
            remote: hasRemote,
            ahead,
            behind,
          });
        }

        automaticActions.push(`Found ${branches.length} branches`);

        return {
          requestedData: {
            branches,
            currentBranch,
          },
          automaticActions,
          issuesFound,
          suggestedActions,
          allPRStatus: [],
        };
      }

      case 'start-work': {
        if (!input.issueNumber) {
          throw new Error('Issue number is required for start-work action');
        }

        // This is a high-level action that combines multiple operations
        automaticActions.push(`Starting work on issue #${input.issueNumber}`);

        // First, check for uncommitted changes
        if (hasUncommittedChanges()) {
          issuesFound.push('You have uncommitted changes');
          suggestedActions.push(
            'Commit your changes before starting new work: use git_commit tool',
            'Or stash them: use git_stash tool with action: "save"'
          );

          return {
            requestedData: {},
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: [],
          };
        }

        // Get issue details
        const issueDetails = await getIssueDetails(input.issueNumber);
        const branchName = createBranchNameFromIssue(issueDetails.title, input.issueNumber);

        // Check if branch already exists
        if (branchExists(branchName)) {
          // Just checkout the existing branch
          execSync(`git checkout ${branchName}`, { encoding: 'utf8' });
          automaticActions.push(`Switched to existing branch: ${branchName}`);

          // Pull latest
          try {
            execSync('git pull', { encoding: 'utf8' });
            automaticActions.push('Pulled latest changes');
          } catch {
            // Might not have upstream yet
          }
        } else {
          // Create new branch from main
          const mainBranch = branchExists('main') ? 'main' : 'master';
          execSync(`git checkout ${mainBranch}`, { encoding: 'utf8' });
          execSync('git pull', { encoding: 'utf8' });
          execSync(`git checkout -b ${branchName}`, { encoding: 'utf8' });
          automaticActions.push(`Created new branch: ${branchName}`);
        }

        // Update workflow state to track current work
        updateWorkflowState({
          currentIssue: input.issueNumber,
          currentBranch: branchName,
          phase: 'implementation',
        });
        automaticActions.push(`Updated workflow state for issue #${input.issueNumber}`);

        return {
          requestedData: {
            currentBranch: branchName,
            createdBranch: branchExists(branchName) ? undefined : branchName,
            issueDetails: {
              number: input.issueNumber,
              title: issueDetails.title,
              url: issueDetails.url,
            },
          },
          automaticActions,
          issuesFound,
          suggestedActions: [`Ready to work on issue #${input.issueNumber}: ${issueDetails.title}`],
          nextSteps: [
            {
              action: 'mark_issue_in_progress',
              description: `Mark issue #${input.issueNumber} as "In Progress"`,
              tool: 'workflow_update_issue',
              parameters: {
                issueNumber: input.issueNumber,
                status: 'in_progress',
              },
              priority: 'high',
              category: 'immediate',
            },
            {
              action: 'start_development',
              description: 'Begin implementing the solution for this issue',
              priority: 'high',
              category: 'next_logical',
            },
            {
              action: 'commit_changes',
              description: 'Use git_commit when you have made meaningful progress',
              tool: 'git_commit',
              parameters: {
                action: 'commit',
                issueNumber: input.issueNumber,
              },
              priority: 'medium',
              category: 'next_logical',
            },
          ],
          allPRStatus: [],
        };
      }

      default:
        throw new Error(`Unknown action: ${input.action}`);
    }
  } catch (error) {
    issuesFound.push(`Error: ${error instanceof Error ? error.message : String(error)}`);

    return {
      requestedData: {},
      automaticActions,
      issuesFound,
      suggestedActions: ['Fix the error and try again'],
      nextSteps: [
        {
          action: 'troubleshoot_error',
          description: 'Resolve the git error and retry the operation',
          priority: 'high',
          category: 'immediate',
        },
      ],
      allPRStatus: [],
    };
  }
}
