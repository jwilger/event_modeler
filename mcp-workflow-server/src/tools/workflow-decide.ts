import { execSync } from 'child_process';
import { Octokit } from '@octokit/rest';
import { WorkflowResponse } from '../types.js';
import { getProjectConfig } from '../config.js';

interface DecisionInput {
  decisionId: string;
  selectedChoice: string | number;
  reasoning?: string;
}

interface NextStepAction {
  action: 'epic_analysis' | 'work_on_issue' | 'assign_and_start';
  issueNumber?: number;
  title?: string;
  status?: string;
  suggestion?: string;
  epicNumber?: number;
  epicTitle?: string;
}

interface WorkflowDecideResponse extends WorkflowResponse {
  requestedData: {
    nextSteps: NextStepAction[];
    decision: {
      decisionId: string;
      selectedChoice: string | number;
      reasoning?: string;
    };
    context: Record<string, any>;
  };
}

async function getCurrentUser(): Promise<string> {
  try {
    const output = execSync('gh api user --jq .login', { encoding: 'utf8' });
    return output.trim();
  } catch (error) {
    throw new Error('Failed to get current GitHub user. Make sure gh CLI is authenticated.');
  }
}

export async function workflowDecide(input: DecisionInput): Promise<WorkflowDecideResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];

  try {
    // Check configuration first
    const { config, isComplete } = getProjectConfig();
    
    if (!isComplete) {
      throw new Error('Configuration is incomplete. Please run workflow_configure first.');
    }
    // Validate input
    if (!input.decisionId) {
      throw new Error('decisionId is required');
    }
    if (input.selectedChoice === undefined || input.selectedChoice === null) {
      throw new Error('selectedChoice is required');
    }

    automaticActions.push(`Processing decision for ID: ${input.decisionId}`);
    automaticActions.push(`Selected choice: ${input.selectedChoice}`);
    
    if (input.reasoning) {
      automaticActions.push(`Reasoning: ${input.reasoning}`);
    }

    // Extract epic number from decision ID (format: epic-{number}-next-issue-{timestamp})
    const epicMatch = input.decisionId.match(/epic-(\d+)-next-issue/);
    if (!epicMatch) {
      throw new Error('Invalid decision ID format');
    }
    const epicNumber = parseInt(epicMatch[1]);

    // Get GitHub token and set up Octokit
    const token = execSync('gh auth token', { encoding: 'utf8' }).trim();
    const octokit = new Octokit({ auth: token });

    // Get current user
    const currentUser = await getCurrentUser();
    automaticActions.push(`Current user: ${currentUser}`);

    // Get repository info from git remote
    const remoteUrl = execSync('git remote get-url origin', { encoding: 'utf8' }).trim();
    const repoMatch = remoteUrl.match(/github\.com[:/]([^/]+)\/([^/]+?)(?:\.git)?$/);
    if (!repoMatch) {
      throw new Error('Could not determine repository from git remote');
    }
    const owner = repoMatch[1];
    const repo = repoMatch[2];

    // Get the selected issue details
    const { data: issue } = await octokit.issues.get({
      owner,
      repo,
      issue_number: input.selectedChoice as number
    });

    // Check if issue is assigned to current user
    const isAssignedToMe = issue.assignees?.some(assignee => assignee.login === currentUser);
    
    if (!isAssignedToMe) {
      // Assign the issue to current user
      automaticActions.push(`Assigning issue #${issue.number} to ${currentUser}`);
      await octokit.issues.addAssignees({
        owner,
        repo,
        issue_number: issue.number,
        assignees: [currentUser]
      });
      suggestedActions.push(`Issue #${issue.number} has been assigned to you`);
    }

    // Update issue status to "In Progress" in the project
    try {
      automaticActions.push(`Updating issue #${issue.number} status to "In Progress"`);
      
      // First, get the project item ID for this issue
      const projectQuery = `
        query($owner: String!, $projectNumber: Int!) {
          user(login: $owner) {
            projectV2(number: $projectNumber) {
              items(first: 100) {
                nodes {
                  id
                  content {
                    ... on Issue {
                      number
                    }
                  }
                }
              }
            }
          }
        }
      `;
      
      const projectData = await octokit.graphql(projectQuery, {
        owner: owner,
        projectNumber: config.github.projectNumber!
      });
      
      const items = (projectData as any).user.projectV2.items.nodes;
      const projectItem = items.find((item: any) => 
        item.content && item.content.number === issue.number
      );
      
      if (projectItem) {
        // Update the status field
        const updateMutation = `
          mutation($projectId: ID!, $itemId: ID!, $fieldId: ID!, $value: ProjectV2FieldValue!) {
            updateProjectV2ItemFieldValue(input: {
              projectId: $projectId,
              itemId: $itemId,
              fieldId: $fieldId,
              value: $value
            }) {
              projectV2Item {
                id
              }
            }
          }
        `;
        
        await octokit.graphql(updateMutation, {
          projectId: config.github.projectId!,
          itemId: projectItem.id,
          fieldId: config.github.statusFieldId!,
          value: { singleSelectOptionId: config.github.statusOptions!.inProgress! }
        });
        
        suggestedActions.push(`Issue #${issue.number} status updated to "In Progress"`);
        automaticActions.push('Project status successfully updated');
      } else {
        automaticActions.push('Could not find issue in project - may need to be added to project first');
      }
    } catch (error) {
      automaticActions.push(`Could not update project status: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }

    // Get current git status
    const currentBranch = execSync('git branch --show-current', { encoding: 'utf8' }).trim();
    const gitStatus = execSync('git status --porcelain', { encoding: 'utf8' });
    const hasUncommittedChanges = gitStatus.length > 0;

    // Generate branch name
    const branchName = `feature/${issue.title.toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '')
      .substring(0, 50)}-${issue.number}`;

    // Check if we need to create a new branch
    let branchCreated = false;
    let branchSwitched = false;
    if (!hasUncommittedChanges && currentBranch !== branchName) {
      try {
        // Check if branch already exists locally
        execSync(`git rev-parse --verify ${branchName}`, { encoding: 'utf8' });
        // Branch exists, switch to it
        execSync(`git checkout ${branchName}`, { encoding: 'utf8' });
        automaticActions.push(`Switched to existing branch: ${branchName}`);
        branchSwitched = true;
      } catch {
        // Branch doesn't exist, create it
        try {
          execSync(`git checkout -b ${branchName}`, { encoding: 'utf8' });
          automaticActions.push(`Created and switched to new branch: ${branchName}`);
          branchCreated = true;
          branchSwitched = true;
        } catch (error) {
          automaticActions.push(`Could not create branch: ${error instanceof Error ? error.message : 'Unknown error'}`);
        }
      }
    }

    // Return the next steps
    return {
      requestedData: {
        nextSteps: [{
          action: 'assign_and_start',
          issueNumber: issue.number,
          title: issue.title,
          epicNumber: epicNumber,
          suggestion: `Issue #${issue.number} assigned and marked as "In Progress". ${
            hasUncommittedChanges ? 
            'Commit current changes before switching branches.' : 
            branchCreated ? 
            `Created and switched to new branch: ${branchName}` :
            branchSwitched ? 
            `Switched to existing branch: ${branchName}` :
            `Ready to work on branch: ${branchName}`
          }`
        }],
        decision: {
          decisionId: input.decisionId,
          selectedChoice: input.selectedChoice,
          reasoning: input.reasoning
        },
        context: {
          currentBranch,
          hasUncommittedChanges,
          isAssignedToMe: true, // Now it is
          issueStatus: 'In Progress'
        }
      },
      automaticActions,
      issuesFound,
      suggestedActions: [
        `Work on issue #${issue.number}: ${issue.title}`,
        hasUncommittedChanges ? 
          'Commit or stash current changes before creating a new branch' : 
          `Create branch: git checkout -b feature/issue-${issue.number}`
      ],
      allPRStatus: []
    };
  } catch (error) {
    issuesFound.push(`Error: ${error instanceof Error ? error.message : String(error)}`);
    
    return {
      requestedData: {
        nextSteps: [],
        decision: {
          decisionId: input.decisionId || 'unknown',
          selectedChoice: input.selectedChoice || 'none',
          reasoning: input.reasoning
        },
        context: {}
      },
      automaticActions,
      issuesFound,
      suggestedActions: ['Fix the error and try again'],
      allPRStatus: []
    };
  }
}