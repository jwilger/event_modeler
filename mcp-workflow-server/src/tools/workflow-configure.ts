import { WorkflowResponse } from '../types.js';
import { loadConfig, saveConfig, getMissingConfigFields, WorkflowConfig } from '../config.js';

interface ConfigureInput {
  projectNumber?: number;
  projectId?: string;
  statusFieldId?: string;
  todoOptionId?: string;
  inProgressOptionId?: string;
  doneOptionId?: string;
}

interface WorkflowConfigureResponse extends WorkflowResponse {
  requestedData: {
    config: WorkflowConfig;
    missingFields: string[];
    updated: boolean;
  };
}

export async function workflowConfigure(
  input: ConfigureInput = {}
): Promise<WorkflowConfigureResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];

  try {
    // Load current config
    const config = loadConfig();
    automaticActions.push('Loaded current configuration');

    // Update config with provided values
    let updated = false;

    if (input.projectNumber !== undefined) {
      config.github.projectNumber = input.projectNumber;
      automaticActions.push(`Set project number: ${input.projectNumber}`);
      updated = true;
    }

    if (input.projectId !== undefined) {
      config.github.projectId = input.projectId;
      automaticActions.push(`Set project ID: ${input.projectId}`);
      updated = true;
    }

    if (input.statusFieldId !== undefined) {
      config.github.statusFieldId = input.statusFieldId;
      automaticActions.push(`Set status field ID: ${input.statusFieldId}`);
      updated = true;
    }

    if (input.todoOptionId !== undefined) {
      config.github.statusOptions = config.github.statusOptions || {};
      config.github.statusOptions.todo = input.todoOptionId;
      automaticActions.push(`Set todo option ID: ${input.todoOptionId}`);
      updated = true;
    }

    if (input.inProgressOptionId !== undefined) {
      config.github.statusOptions = config.github.statusOptions || {};
      config.github.statusOptions.inProgress = input.inProgressOptionId;
      automaticActions.push(`Set in-progress option ID: ${input.inProgressOptionId}`);
      updated = true;
    }

    if (input.doneOptionId !== undefined) {
      config.github.statusOptions = config.github.statusOptions || {};
      config.github.statusOptions.done = input.doneOptionId;
      automaticActions.push(`Set done option ID: ${input.doneOptionId}`);
      updated = true;
    }

    // Save if updated
    if (updated) {
      saveConfig(config);
      automaticActions.push('Configuration saved to .mcp-workflow.json');
      suggestedActions.push('Configuration updated successfully');
    } else {
      automaticActions.push('No configuration changes made');
    }

    // Check what's still missing
    const missingFields = getMissingConfigFields(config);

    if (missingFields.length > 0) {
      suggestedActions.push('Still missing configuration for: ' + missingFields.join(', '));

      if (missingFields.includes('github.projectNumber')) {
        suggestedActions.push('Run: gh project list --owner <owner> to find your project number');
      }
      if (missingFields.includes('github.projectId')) {
        suggestedActions.push(
          "Run: gh project list --owner <owner> --format json | jq '.projects[] | select(.number == <number>) | .id'"
        );
      }
      if (missingFields.includes('github.statusFieldId')) {
        suggestedActions.push(
          'Run: gh project field-list <project-number> --owner <owner> to find the Status field ID'
        );
      }
    } else {
      automaticActions.push('All required configuration is present');
    }

    return {
      requestedData: {
        config,
        missingFields,
        updated,
      },
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus: [],
    };
  } catch (error) {
    issuesFound.push(`Error: ${error instanceof Error ? error.message : String(error)}`);

    return {
      requestedData: {
        config: { github: {} },
        missingFields: [],
        updated: false,
      },
      automaticActions,
      issuesFound,
      suggestedActions: ['Fix the error and try again'],
      allPRStatus: [],
    };
  }
}
