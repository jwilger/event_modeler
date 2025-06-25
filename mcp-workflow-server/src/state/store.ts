import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'fs';
import { homedir } from 'os';
import { join, dirname } from 'path';

export interface WorkflowState {
  lastCheckedPRs: {
    [prNumber: number]: {
      lastChecked: string;
      lastReviewCount: number;
      lastCheckRunStatus: string;
    };
  };
  branchCreationDates: {
    [branchName: string]: string;
  };
  lastStatusCheck: string;
  version: number;
}

const STATE_VERSION = 1;
const STATE_PATH = join(homedir(), '.event-modeler', 'mcp-workflow-state.json');

export class StateStore {
  private state: WorkflowState;

  constructor() {
    this.state = this.loadState();
  }

  private loadState(): WorkflowState {
    try {
      if (existsSync(STATE_PATH)) {
        const data = readFileSync(STATE_PATH, 'utf-8');
        const loaded = JSON.parse(data) as WorkflowState;
        
        // Handle version migrations if needed
        if (loaded.version !== STATE_VERSION) {
          return this.migrateState(loaded);
        }
        
        return loaded;
      }
    } catch (error) {
      console.error('Failed to load state, starting fresh:', error);
    }

    // Return default state
    return {
      lastCheckedPRs: {},
      branchCreationDates: {},
      lastStatusCheck: new Date().toISOString(),
      version: STATE_VERSION,
    };
  }

  /**
   * Migrates the state from an older version to the current version.
   * 
   * This method is called when the loaded state version does not match the current
   * `STATE_VERSION`. The migration logic should transform the old state into a format
   * compatible with the current version. Each version upgrade should be handled explicitly
   * to ensure data integrity and compatibility.
   * 
   * TODO: Implement migration logic for future state version updates.
   * Example:
   * if (_oldState.version === 1) {
   *   // Perform migration from version 1 to version 2
   * }
   * 
   * @param _oldState - The state object from a previous version.
   * @returns The migrated state object compatible with the current version.
   */
  private migrateState(_oldState: unknown): WorkflowState {
    // For now, we start fresh when version mismatch occurs
    // In the future, implement specific migration logic based on version
    console.error('State migration needed but not implemented - starting with fresh state');
    return {
      lastCheckedPRs: {},
      branchCreationDates: {},
      lastStatusCheck: new Date().toISOString(),
      version: STATE_VERSION,
    };
  }

  private saveState(): void {
    try {
      const dir = dirname(STATE_PATH);
      if (!existsSync(dir)) {
        mkdirSync(dir, { recursive: true });
      }
      
      writeFileSync(STATE_PATH, JSON.stringify(this.state, null, 2));
    } catch (error) {
      console.error('Failed to save state:', error);
    }
  }

  updatePRStatus(prNumber: number, reviewCount: number, checkRunStatus: string): void {
    this.state.lastCheckedPRs[prNumber] = {
      lastChecked: new Date().toISOString(),
      lastReviewCount: reviewCount,
      lastCheckRunStatus: checkRunStatus,
    };
    this.saveState();
  }

  getPRStatus(prNumber: number): WorkflowState['lastCheckedPRs'][number] | undefined {
    return this.state.lastCheckedPRs[prNumber];
  }

  recordBranchCreation(branchName: string): void {
    if (!this.state.branchCreationDates[branchName]) {
      this.state.branchCreationDates[branchName] = new Date().toISOString();
      this.saveState();
    }
  }

  getBranchCreationDate(branchName: string): string | undefined {
    return this.state.branchCreationDates[branchName];
  }

  updateLastStatusCheck(): void {
    this.state.lastStatusCheck = new Date().toISOString();
    this.saveState();
  }

  getLastStatusCheck(): string {
    return this.state.lastStatusCheck;
  }

  clearPRStatus(prNumber: number): void {
    delete this.state.lastCheckedPRs[prNumber];
    this.saveState();
  }
}