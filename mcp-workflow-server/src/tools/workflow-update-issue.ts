import { WorkflowResponse, PRStatus } from '../types.js';
import { getRepoInfo } from '../utils/github.js';
import { Octokit } from '@octokit/rest';
import { getGitHubToken } from '../utils/auth.js';
import { getProjectConfig } from '../config.js';

interface UpdateIssueFieldsParams {
  issueNumber: number;
  status?: 'todo' | 'in_progress' | 'done';
  type?: 'epic' | 'feature' | 'bug' | 'enhancement' | 'documentation' | 'question';
  priority?: 'low' | 'medium' | 'high' | 'urgent';
}

interface ProjectFieldOption {
  id: string;
  name: string;
}

interface ProjectField {
  id: string;
  name: string;
  options?: ProjectFieldOption[];
}

interface ProjectItem {
  id: string;
  content?: {
    number?: number;
  };
}

interface ProjectQueryResult {
  user?: {
    projectV2?: {
      id: string;
      fields?: {
        nodes?: ProjectField[];
      };
      items?: {
        nodes?: ProjectItem[];
      };
    };
  };
}


export async function workflowUpdateIssue(
  params: UpdateIssueFieldsParams
): Promise<WorkflowResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];
  const allPRStatus: PRStatus[] = [];

  try {
    const { issueNumber, status, type, priority } = params;
    
    if (!issueNumber) {
      throw new Error('Issue number is required');
    }

    // Get configuration
    const configResult = getProjectConfig();
    if (!configResult.isComplete || !configResult.config.github.projectId || !configResult.config.github.projectNumber) {
      throw new Error('Project configuration not found. Run workflow_configure first.');
    }
    const config = configResult.config.github;

    const token = getGitHubToken();
    const octokit = new Octokit({ auth: token });
    const { owner } = getRepoInfo();

    automaticActions.push(`Processing update for issue #${issueNumber}`);

    // First, get the project item ID for this issue
    const projectQuery = `
      query($owner: String!, $projectNumber: Int!) {
        user(login: $owner) {
          projectV2(number: $projectNumber) {
            id
            fields(first: 20) {
              nodes {
                ... on ProjectV2Field {
                  id
                  name
                }
                ... on ProjectV2SingleSelectField {
                  id
                  name
                  options {
                    id
                    name
                  }
                }
              }
            }
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

    const projectResult = await octokit.graphql<ProjectQueryResult>(projectQuery, {
      owner,
      projectNumber: config.projectNumber!,
    });

    // Find the project item for this issue
    const projectItems = projectResult.user?.projectV2?.items?.nodes || [];
    const projectItem = projectItems.find(
      (item) => item.content?.number === issueNumber
    );

    if (!projectItem) {
      throw new Error(`Issue #${issueNumber} not found in project`);
    }

    const projectItemId = projectItem.id;
    automaticActions.push(`Found project item ID: ${projectItemId}`);

    // Get field information
    const fields = projectResult.user?.projectV2?.fields?.nodes || [];
    const updatedFields: Record<string, string> = {};

    // Update status if provided
    if (status) {
      const statusField = fields.find((f) => f.name === 'Status');
      if (!statusField) {
        issuesFound.push('Status field not found in project');
      } else {
        const statusOption = statusField.options?.find(
          (opt) => opt.name.toLowerCase() === status.replace('_', ' ')
        );
        
        if (!statusOption) {
          issuesFound.push(`Invalid status value: ${status}`);
        } else {
          await updateProjectField(
            octokit,
            config.projectId!,
            projectItemId,
            statusField.id,
            { singleSelectOptionId: statusOption.id }
          );
          updatedFields.status = statusOption.name;
          automaticActions.push(`Updated status to "${statusOption.name}"`);
        }
      }
    }

    // Update type if provided
    if (type) {
      const typeField = fields.find((f) => f.name === 'Type');
      if (!typeField) {
        issuesFound.push('Type field not found in project');
      } else {
        const typeOption = typeField.options?.find(
          (opt) => opt.name.toLowerCase() === type.toLowerCase()
        );
        
        if (!typeOption) {
          issuesFound.push(`Invalid type value: ${type}`);
        } else {
          await updateProjectField(
            octokit,
            config.projectId!,
            projectItemId,
            typeField.id,
            { singleSelectOptionId: typeOption.id }
          );
          updatedFields.type = typeOption.name;
          automaticActions.push(`Updated type to "${typeOption.name}"`);
        }
      }
    }

    // Update priority if provided
    if (priority) {
      const priorityField = fields.find((f) => f.name === 'Priority');
      if (!priorityField) {
        issuesFound.push('Priority field not found in project');
      } else {
        const priorityOption = priorityField.options?.find(
          (opt) => opt.name.toLowerCase() === priority.toLowerCase()
        );
        
        if (!priorityOption) {
          issuesFound.push(`Invalid priority value: ${priority}`);
        } else {
          await updateProjectField(
            octokit,
            config.projectId!,
            projectItemId,
            priorityField.id,
            { singleSelectOptionId: priorityOption.id }
          );
          updatedFields.priority = priorityOption.name;
          automaticActions.push(`Updated priority to "${priorityOption.name}"`);
        }
      }
    }

    if (Object.keys(updatedFields).length === 0) {
      suggestedActions.push('No fields were updated. Specify status, type, or priority to update.');
    }

    return {
      requestedData: {
        issueNumber,
        updatedFields,
        projectItemId,
      },
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus,
    };
  } catch (error) {
    issuesFound.push(`Error: ${error instanceof Error ? error.message : String(error)}`);
    return {
      requestedData: {
        error: error instanceof Error ? error.message : String(error),
      },
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus,
    };
  }
}

async function updateProjectField(
  octokit: Octokit,
  projectId: string,
  itemId: string,
  fieldId: string,
  value: { singleSelectOptionId: string }
): Promise<void> {
  const mutation = `
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

  await octokit.graphql(mutation, {
    projectId,
    itemId,
    fieldId,
    value,
  });
}