import { Octokit } from '@octokit/rest';
import { execSync } from 'child_process';
import { WorkflowResponse } from '../types.js';

interface TodoItem {
  text: string;
  checked: boolean;
  index: number;
}

interface NextStepAction {
  action: 'work_on_todo' | 'todos_complete' | 'select_work';
  issueNumber?: number;
  title?: string;
  status?: string;
  todoItem?: string;
  todoIndex?: number;
  totalTodos?: number;
  completedTodos?: number;
  suggestion?: string;
  projectUrl?: string;
  reason?: string;
}

interface WorkflowNextResponse extends WorkflowResponse {
  requestedData: {
    nextSteps: NextStepAction[];
    context: Record<string, any>;
  };
}

function parseTodoItems(body: string): TodoItem[] {
  const lines = body.split('\n');
  const todos: TodoItem[] = [];
  let index = 0;

  for (const line of lines) {
    const checkedMatch = line.match(/^\s*-\s+\[x\]\s+(.+)$/i);
    const uncheckedMatch = line.match(/^\s*-\s+\[\s*\]\s+(.+)$/);
    
    if (checkedMatch || uncheckedMatch) {
      todos.push({
        text: (checkedMatch || uncheckedMatch)![1].trim(),
        checked: !!checkedMatch,
        index: index++
      });
    }
  }

  return todos;
}

async function getCurrentUser(): Promise<string> {
  try {
    const output = execSync('gh api user --jq .login', { encoding: 'utf8' });
    return output.trim();
  } catch (error) {
    throw new Error('Failed to get current GitHub user. Make sure gh CLI is authenticated.');
  }
}

export async function workflowNext(): Promise<WorkflowNextResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];

  try {
    // Get GitHub token from gh CLI
    const token = execSync('gh auth token', { encoding: 'utf8' }).trim();
    const octokit = new Octokit({ auth: token });

    // Get current user
    const currentUser = await getCurrentUser();
    automaticActions.push(`Identified current user: ${currentUser}`);

    // Get repository info
    const repoInfo = execSync('gh repo view --json owner,name', { encoding: 'utf8' });
    const { owner, name } = JSON.parse(repoInfo);
    automaticActions.push(`Working in repository: ${owner.login}/${name}`);

    // Query project for issues assigned to current user
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
                    title
                    body
                    state
                    assignees(first: 10) {
                      nodes {
                        login
                      }
                    }
                  }
                }
                fieldValues(first: 20) {
                  nodes {
                    ... on ProjectV2ItemFieldSingleSelectValue {
                      name
                      field {
                        ... on ProjectV2SingleSelectField {
                          name
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    `;

    const projectData = await octokit.graphql(projectQuery, {
      owner: owner.login,
      projectNumber: 9
    });

    // Filter to issues assigned to current user and in progress
    const items = (projectData as any).user.projectV2.items.nodes;
    const inProgressIssues = items.filter((item: any) => {
      if (!item.content || !item.content.assignees) return false;
      
      const isAssignedToUser = item.content.assignees.nodes.some(
        (assignee: any) => assignee.login === currentUser
      );
      
      const statusField = item.fieldValues.nodes.find(
        (field: any) => field.field && field.field.name === 'Status'
      );
      const isInProgress = statusField && statusField.name === 'In Progress';
      
      return isAssignedToUser && isInProgress;
    });

    automaticActions.push(`Found ${inProgressIssues.length} issues assigned to ${currentUser} in progress`);

    // Get current git status
    const gitStatus = execSync('git status --porcelain', { encoding: 'utf8' });
    const currentBranch = execSync('git branch --show-current', { encoding: 'utf8' }).trim();
    const hasUncommittedChanges = gitStatus.length > 0;

    if (inProgressIssues.length === 0) {
      // No in-progress work assigned
      return {
        requestedData: {
          nextSteps: [{
            action: 'select_work',
            projectUrl: `https://github.com/users/${owner.login}/projects/9`,
            reason: 'No issues in progress. Visit project board to select next item.'
          }],
          context: {
            assignedIssues: 0,
            inProgressIssues: 0
          }
        },
        automaticActions,
        issuesFound,
        suggestedActions: ['Visit the project board to select your next task'],
        allPRStatus: []
      };
    }

    // Process the first in-progress issue
    const issue = inProgressIssues[0].content;
    const todos = parseTodoItems(issue.body || '');
    const completedTodos = todos.filter(t => t.checked).length;
    const nextTodo = todos.find(t => !t.checked);

    if (!nextTodo) {
      // All todos complete
      return {
        requestedData: {
          nextSteps: [{
            action: 'todos_complete',
            issueNumber: issue.number,
            title: issue.title,
            status: 'In Progress',
            suggestion: 'All todos complete. Create PR if not exists, or close issue if PR merged.'
          }],
          context: {
            totalTodos: todos.length,
            completedTodos: todos.length,
            hasPR: false, // Could enhance to check for existing PR
            currentBranch,
            hasUncommittedChanges
          }
        },
        automaticActions,
        issuesFound,
        suggestedActions: ['Create a pull request for the completed work'],
        allPRStatus: []
      };
    }

    // Return next todo to work on
    return {
      requestedData: {
        nextSteps: [{
          action: 'work_on_todo',
          issueNumber: issue.number,
          title: issue.title,
          status: 'In Progress',
          todoItem: nextTodo.text,
          todoIndex: nextTodo.index,
          totalTodos: todos.length,
          completedTodos
        }],
        context: {
          currentBranch,
          hasUncommittedChanges
        }
      },
      automaticActions,
      issuesFound,
      suggestedActions: [`Work on: ${nextTodo.text}`],
      allPRStatus: []
    };

  } catch (error) {
    issuesFound.push(`Error: ${error instanceof Error ? error.message : String(error)}`);
    suggestedActions.push('Check that gh CLI is authenticated and has access to the repository');

    return {
      requestedData: {
        nextSteps: [],
        context: {}
      },
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus: []
    };
  }
}