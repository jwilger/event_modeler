import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join } from 'path';
import { execSync } from 'child_process';

export interface WorkflowAction {
  type: string;
  status: 'pending' | 'completed' | 'failed';
  enforcement: 'auto' | 'suggest' | 'warn';
  completedAt?: string;
  failureReason?: string;
}

export interface WorkflowState {
  currentIssue?: number;
  currentBranch?: string;
  phase: 'ready' | 'implementation' | 'pr_created' | 'under_review' | 'merge_ready';
  requiredActions: WorkflowAction[];
  completedActions: WorkflowAction[];
  enforcementPolicies: {
    create_pr_when_commits_exist: 'auto' | 'suggest' | 'warn';
    assign_issue_on_status_change: 'auto' | 'suggest' | 'warn';
    request_review_when_pr_ready: 'auto' | 'suggest' | 'warn';
  };
}

export interface WorkflowConfig {
  github: {
    projectNumber?: number;
    projectId?: string;
    statusFieldId?: string;
    statusOptions?: {
      todo?: string;
      inProgress?: string;
      done?: string;
    };
  };
  workflowState?: WorkflowState;
}

interface ConfigRequest {
  action: 'requires_config';
  missingFields: string[];
  currentConfig: WorkflowConfig;
  suggestions: string[];
}

export interface ConfigResponse {
  requiresConfig: boolean;
  configRequest?: ConfigRequest;
}

const CONFIG_FILE = '.mcp-workflow.json';

let cachedConfig: WorkflowConfig | null = null;

export function getProjectRoot(): string {
  try {
    const root = execSync('git rev-parse --show-toplevel', { encoding: 'utf8' }).trim();
    return root;
  } catch {
    // If not in a git repo, use current directory
    return process.cwd();
  }
}

export function loadConfig(): WorkflowConfig {
  if (cachedConfig) {
    return cachedConfig;
  }

  const configPath = join(getProjectRoot(), CONFIG_FILE);
  
  if (existsSync(configPath)) {
    try {
      const content = readFileSync(configPath, 'utf8');
      const parsedConfig = JSON.parse(content);
      
      // Ensure workflow state exists with defaults
      if (!parsedConfig.workflowState) {
        parsedConfig.workflowState = getDefaultWorkflowState();
      }
      
      cachedConfig = parsedConfig;
      return cachedConfig!;
    } catch (error) {
      console.error('Error reading config file:', error);
    }
  }

  // Return empty config if file doesn't exist or is invalid
  return {
    github: {},
    workflowState: getDefaultWorkflowState()
  };
}

export function saveConfig(config: WorkflowConfig): void {
  const configPath = join(getProjectRoot(), CONFIG_FILE);
  writeFileSync(configPath, JSON.stringify(config, null, 2) + '\n');
  cachedConfig = config;
}

export function getMissingConfigFields(config: WorkflowConfig): string[] {
  const missing: string[] = [];
  
  if (!config.github.projectNumber) {
    missing.push('github.projectNumber');
  }
  if (!config.github.projectId) {
    missing.push('github.projectId');
  }
  if (!config.github.statusFieldId) {
    missing.push('github.statusFieldId');
  }
  if (!config.github.statusOptions?.todo) {
    missing.push('github.statusOptions.todo');
  }
  if (!config.github.statusOptions?.inProgress) {
    missing.push('github.statusOptions.inProgress');
  }
  if (!config.github.statusOptions?.done) {
    missing.push('github.statusOptions.done');
  }
  
  return missing;
}

export function createConfigRequest(missingFields: string[]): ConfigRequest {
  const config = loadConfig();
  
  const suggestions: string[] = [];
  
  if (missingFields.includes('github.projectNumber')) {
    suggestions.push('Run "gh project list --owner <owner>" to find your project number');
  }
  if (missingFields.includes('github.projectId')) {
    suggestions.push('Run "gh project list --owner <owner> --format json" to find the project ID');
  }
  if (missingFields.includes('github.statusFieldId') || missingFields.some(f => f.startsWith('github.statusOptions'))) {
    suggestions.push('Run "gh project field-list <project-number> --owner <owner>" to find field IDs and option values');
  }
  
  return {
    action: 'requires_config',
    missingFields,
    currentConfig: config,
    suggestions
  };
}

export function hasRequiredConfig(): boolean {
  const config = loadConfig();
  const missing = getMissingConfigFields(config);
  return missing.length === 0;
}

// Helper to get project values with config check
export function getProjectConfig(): { config: WorkflowConfig; isComplete: boolean } {
  const config = loadConfig();
  const isComplete = getMissingConfigFields(config).length === 0;
  return { config, isComplete };
}

// Workflow state management functions
export function getDefaultWorkflowState(): WorkflowState {
  return {
    phase: 'ready',
    requiredActions: [],
    completedActions: [],
    enforcementPolicies: {
      create_pr_when_commits_exist: 'auto',
      assign_issue_on_status_change: 'auto',
      request_review_when_pr_ready: 'suggest',
    },
  };
}

export function getWorkflowState(): WorkflowState {
  const config = loadConfig();
  return config.workflowState || getDefaultWorkflowState();
}

export function updateWorkflowState(updates: Partial<WorkflowState>): void {
  const config = loadConfig();
  const currentState = config.workflowState || getDefaultWorkflowState();
  
  config.workflowState = {
    ...currentState,
    ...updates,
  };
  
  saveConfig(config);
}

export function addRequiredAction(action: WorkflowAction): void {
  const state = getWorkflowState();
  
  // Don't add if already exists
  const exists = state.requiredActions.some(a => a.type === action.type);
  if (exists) return;
  
  state.requiredActions.push(action);
  updateWorkflowState(state);
}

export function completeAction(actionType: string): void {
  const state = getWorkflowState();
  
  // Move from required to completed
  const actionIndex = state.requiredActions.findIndex(a => a.type === actionType);
  if (actionIndex >= 0) {
    const action = state.requiredActions[actionIndex];
    action.status = 'completed';
    action.completedAt = new Date().toISOString();
    
    state.completedActions.push(action);
    state.requiredActions.splice(actionIndex, 1);
    
    updateWorkflowState(state);
  }
}

export function getRequiredActions(enforcement?: 'auto' | 'suggest' | 'warn'): WorkflowAction[] {
  const state = getWorkflowState();
  if (!enforcement) {
    return state.requiredActions;
  }
  return state.requiredActions.filter(a => a.enforcement === enforcement);
}

export function resetWorkflowState(): void {
  const config = loadConfig();
  config.workflowState = getDefaultWorkflowState();
  saveConfig(config);
}