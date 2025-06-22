import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join } from 'path';
import { execSync } from 'child_process';

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
      cachedConfig = JSON.parse(content);
      return cachedConfig!;
    } catch (error) {
      console.error('Error reading config file:', error);
    }
  }

  // Return empty config if file doesn't exist or is invalid
  return {
    github: {}
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