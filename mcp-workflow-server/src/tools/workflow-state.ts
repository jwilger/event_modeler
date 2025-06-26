import { WorkflowResponse } from '../types.js';
import { 
  getWorkflowState, 
  resetWorkflowState, 
  WorkflowState 
} from '../config.js';

interface WorkflowStateInput {
  action: 'get' | 'reset' | 'validate';
}

interface WorkflowStateResponse extends WorkflowResponse {
  requestedData: {
    workflowState?: WorkflowState;
    validation?: {
      isValid: boolean;
      issues: string[];
      recommendations: string[];
    };
    error?: string;
  };
}

export async function workflowState(
  input: WorkflowStateInput
): Promise<WorkflowStateResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];

  try {
    const { action } = input;

    switch (action) {
      case 'get': {
        const state = getWorkflowState();
        
        return {
          requestedData: {
            workflowState: state,
          },
          automaticActions: ['Retrieved current workflow state'],
          issuesFound,
          suggestedActions,
          nextSteps: [],
          allPRStatus: [],
        };
      }

      case 'reset': {
        resetWorkflowState();
        automaticActions.push('Reset workflow state to defaults');
        
        return {
          requestedData: {
            workflowState: getWorkflowState(),
          },
          automaticActions,
          issuesFound,
          suggestedActions: ['Workflow state has been reset. Use workflow_next to determine next actions.'],
          nextSteps: [
            {
              action: 'check_next_work',
              description: 'Use workflow_next to determine what to work on',
              tool: 'workflow_next',
              priority: 'high',
              category: 'immediate',
            },
          ],
          allPRStatus: [],
        };
      }

      case 'validate': {
        const state = getWorkflowState();
        const validation = validateWorkflowState(state);
        
        if (validation.issues.length > 0) {
          issuesFound.push(...validation.issues);
        }
        
        if (validation.recommendations.length > 0) {
          suggestedActions.push(...validation.recommendations);
        }

        return {
          requestedData: {
            workflowState: state,
            validation,
          },
          automaticActions: ['Validated workflow state'],
          issuesFound,
          suggestedActions,
          nextSteps: validation.isValid 
            ? [
                {
                  action: 'continue_workflow',
                  description: 'Workflow state is valid, continue with current work',
                  tool: 'workflow_next',
                  priority: 'medium',
                  category: 'next_logical',
                },
              ]
            : [
                {
                  action: 'fix_workflow_state',
                  description: 'Fix workflow state issues before continuing',
                  tool: 'workflow_state',
                  parameters: { action: 'reset' },
                  priority: 'high',
                  category: 'immediate',
                },
              ],
          allPRStatus: [],
        };
      }

      default:
        throw new Error(`Unknown action: ${action}`);
    }
  } catch (error) {
    issuesFound.push(`Error: ${error instanceof Error ? error.message : String(error)}`);
    
    return {
      requestedData: {
        error: error instanceof Error ? error.message : String(error),
      },
      automaticActions,
      issuesFound,
      suggestedActions: ['Check workflow state configuration'],
      nextSteps: [],
      allPRStatus: [],
    };
  }
}

function validateWorkflowState(state: WorkflowState): {
  isValid: boolean;
  issues: string[];
  recommendations: string[];
} {
  const issues: string[] = [];
  const recommendations: string[] = [];

  // Check for inconsistencies
  if (state.currentIssue && !state.currentBranch) {
    issues.push('Current issue is set but no current branch specified');
    recommendations.push('Use git_branch with start-work action to sync branch with issue');
  }

  if (state.currentBranch && !state.currentIssue) {
    issues.push('Current branch is set but no current issue specified');
    recommendations.push('Extract issue number from branch name or reset workflow state');
  }

  // Check for stale required actions
  const stalePendingActions = state.requiredActions.filter(
    action => action.status === 'pending'
  );
  
  if (stalePendingActions.length > 0) {
    issues.push(`${stalePendingActions.length} pending required actions`);
    recommendations.push('Complete pending actions or reset workflow state');
  }

  // Check phase consistency
  if (state.phase === 'implementation' && state.requiredActions.length === 0) {
    recommendations.push('Consider adding required actions for current implementation phase');
  }

  const isValid = issues.length === 0;

  return {
    isValid,
    issues,
    recommendations,
  };
}