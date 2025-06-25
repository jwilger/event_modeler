import { Octokit } from '@octokit/rest';
import { execSync } from 'child_process';
import { WorkflowResponse } from '../types.js';
import { getRepoInfo } from '../utils/github.js';
import { getProjectConfig } from '../config.js';

interface CreateIssueInput {
  title: string;
  body: string;
  epicNumber?: number;
  type?: 'bug' | 'feature' | 'enhancement' | 'documentation' | 'question';
  priority?: 'low' | 'medium' | 'high' | 'urgent';
  labels?: string[];
  assignees?: string[];
}

interface WorkflowCreateIssueResponse extends WorkflowResponse {
  requestedData: {
    issue?: {
      number: number;
      url: string;
      title: string;
      state: string;
    };
    linkedToEpic?: boolean;
    epicNumber?: number;
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

  const result = await octokit.graphql<{
    repository: {
      issue: {
        id: string;
        title: string;
      } | null;
    };
  }>(query, {
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

async function linkSubIssue(
  octokit: Octokit,
  epicNodeId: string,
  issueNodeId: string
): Promise<void> {
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
}

async function addToProject(
  octokit: Octokit,
  projectId: string,
  issueNodeId: string,
  statusFieldId: string,
  todoOptionId: string,
  _type?: string,
  _priority?: string
): Promise<string> {
  // Add issue to project
  const addToProjectMutation = `
    mutation($projectId: ID!, $contentId: ID!) {
      addProjectV2ItemById(input: {
        projectId: $projectId,
        contentId: $contentId
      }) {
        item {
          id
        }
      }
    }
  `;

  const addResult = await octokit.graphql<{
    addProjectV2ItemById: {
      item: {
        id: string;
      };
    };
  }>(addToProjectMutation, {
    projectId,
    contentId: issueNodeId,
  });

  const itemId = addResult.addProjectV2ItemById.item.id;

  // Set status to Todo
  const updateStatusMutation = `
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

  await octokit.graphql(updateStatusMutation, {
    projectId,
    itemId,
    fieldId: statusFieldId,
    value: { singleSelectOptionId: todoOptionId },
  });

  return itemId;
}

export async function workflowCreateIssue(
  input: CreateIssueInput
): Promise<WorkflowCreateIssueResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];

  try {
    const { title, body, epicNumber, type, priority, labels = [], assignees = [] } = input;

    // Validate required parameters
    if (!title || !body) {
      throw new Error('Missing required parameters: title and body are required');
    }

    // Get repository info
    const { owner, repo } = getRepoInfo();
    automaticActions.push(`Working in repository: ${owner}/${repo}`);

    // Set up GitHub API
    const token = execSync('gh auth token', { encoding: 'utf8' }).trim();
    const octokit = new Octokit({ auth: token });

    // Build labels array
    const finalLabels: string[] = [...labels];
    if (type) {
      finalLabels.push(type);
    }
    if (priority) {
      finalLabels.push(priority);
    }

    automaticActions.push(`Creating issue: "${title}"`);
    if (finalLabels.length > 0) {
      automaticActions.push(`Labels: ${finalLabels.join(', ')}`);
    }
    if (assignees.length > 0) {
      automaticActions.push(`Assignees: ${assignees.join(', ')}`);
    }

    // Create the issue
    const issue = await octokit.issues.create({
      owner,
      repo,
      title,
      body,
      labels: finalLabels,
      assignees,
    });

    automaticActions.push(`Created issue #${issue.data.number}: ${issue.data.title}`);
    suggestedActions.push(`View issue: ${issue.data.html_url}`);

    let linkedToEpic = false;

    // Link to epic if specified
    if (epicNumber) {
      try {
        automaticActions.push(`Linking issue #${issue.data.number} to epic #${epicNumber}`);

        const [epicNodeId, issueNodeId] = await Promise.all([
          getIssueNodeId(octokit, owner, repo, epicNumber),
          getIssueNodeId(octokit, owner, repo, issue.data.number),
        ]);

        await linkSubIssue(octokit, epicNodeId, issueNodeId);
        linkedToEpic = true;
        automaticActions.push(`Successfully linked issue #${issue.data.number} to epic #${epicNumber}`);
        suggestedActions.push(`View epic: https://github.com/${owner}/${repo}/issues/${epicNumber}`);
      } catch (linkError) {
        const linkErrorMessage = linkError instanceof Error ? linkError.message : String(linkError);
        issuesFound.push(`Could not link to epic #${epicNumber}: ${linkErrorMessage}`);
        automaticActions.push(`Warning: Failed to link to epic #${epicNumber}`);
      }
    }

    // Add to project board if configured
    try {
      const projectConfig = getProjectConfig();
      if (projectConfig.isComplete) {
        const issueNodeId = await getIssueNodeId(octokit, owner, repo, issue.data.number);
        
        await addToProject(
          octokit,
          projectConfig.config.github.projectId!,
          issueNodeId,
          projectConfig.config.github.statusFieldId!,
          projectConfig.config.github.statusOptions!.todo!,
          type,
          priority
        );

        automaticActions.push('Added issue to project board with "Todo" status');
      } else {
        automaticActions.push('Project configuration incomplete - skipping project board update');
      }
    } catch (projectError) {
      const projectErrorMessage = projectError instanceof Error ? projectError.message : String(projectError);
      issuesFound.push(`Could not add to project board: ${projectErrorMessage}`);
      automaticActions.push('Warning: Failed to add to project board');
    }

    return {
      requestedData: {
        issue: {
          number: issue.data.number,
          url: issue.data.html_url,
          title: issue.data.title,
          state: issue.data.state,
        },
        linkedToEpic,
        epicNumber,
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
        error: errorMessage,
      },
      automaticActions,
      issuesFound,
      suggestedActions: ['Check the error message and try again'],
      allPRStatus: [],
    };
  }
}