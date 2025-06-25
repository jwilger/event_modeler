import { execSync } from 'child_process';
import { WorkflowResponse } from '../types.js';

interface GitStashInput {
  action: 'list' | 'save' | 'pop' | 'apply' | 'drop' | 'clear' | 'show';
  message?: string;          // For save action
  stashRef?: string | number; // For pop/apply/drop/show - can be index or stash@{n}
  includeUntracked?: boolean; // For save action
  keepIndex?: boolean;       // For save action (--keep-index)
  quiet?: boolean;           // For pop/apply
}

interface StashInfo {
  ref: string;        // e.g., "stash@{0}"
  branch: string;     // Branch where stash was created
  message: string;    // Stash message
  date: string;       // When stashed
  hash: string;       // Commit hash
}

interface GitStashResponse extends WorkflowResponse {
  requestedData: {
    stashes?: StashInfo[];
    currentStash?: StashInfo;
    stashContent?: string;
    stashApplied?: boolean;
    stashDropped?: boolean;
    stashSaved?: boolean;
    error?: string;
  };
}

function parseStashList(): StashInfo[] {
  try {
    const output = execSync('git stash list --format="%gd\0%h\0%gs\0%ci"', { encoding: 'utf8' });
    if (!output.trim()) return [];

    return output.trim().split('\n').map(line => {
      const [ref, hash, subject, date] = line.split('\0');
      // Extract branch and message from subject (format: "On branch: message" or "WIP on branch: message")
      const subjectMatch = subject.match(/^(?:WIP )?[Oo]n (.+?):\s*(.+)$/);
      const branch = subjectMatch?.[1] || 'unknown';
      const message = subjectMatch?.[2] || subject;

      return {
        ref,
        branch,
        message,
        date,
        hash
      };
    });
  } catch {
    return [];
  }
}

function getCurrentBranch(): string {
  try {
    return execSync('git branch --show-current', { encoding: 'utf8' }).trim();
  } catch {
    return 'unknown';
  }
}

function getStashContext(): { branch: string; issueNumber?: number } {
  const branch = getCurrentBranch();
  const issueMatch = branch.match(/(\d+)/);
  return {
    branch,
    issueNumber: issueMatch ? parseInt(issueMatch[1]) : undefined
  };
}

function generateStashMessage(userMessage?: string): string {
  const context = getStashContext();
  const timestamp = new Date().toISOString().slice(0, 19).replace('T', ' ');
  
  if (userMessage) {
    return userMessage;
  }

  // Auto-generate message based on context
  if (context.issueNumber) {
    return `WIP: Issue #${context.issueNumber} - ${timestamp}`;
  }
  
  return `WIP: ${context.branch} - ${timestamp}`;
}

export async function gitStash(input: GitStashInput): Promise<GitStashResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];
  
  try {
    switch (input.action) {
      case 'list': {
        automaticActions.push('Retrieving list of stashes');
        const stashes = parseStashList();
        
        if (stashes.length === 0) {
          automaticActions.push('No stashes found');
        } else {
          automaticActions.push(`Found ${stashes.length} stash${stashes.length > 1 ? 'es' : ''}`);
          suggestedActions.push('Use "apply" or "pop" to restore stashed changes');
        }
        
        return {
          requestedData: { stashes },
          automaticActions,
          issuesFound,
          suggestedActions,
          allPRStatus: []
        };
      }
      
      case 'save': {
        const message = generateStashMessage(input.message);
        let command = 'git stash push';
        
        if (input.includeUntracked) {
          command += ' --include-untracked';
        }
        
        if (input.keepIndex) {
          command += ' --keep-index';
        }
        
        command += ` -m "${message}"`;
        
        automaticActions.push(`Stashing changes with message: "${message}"`);
        
        try {
          execSync(command, { encoding: 'utf8' });
          automaticActions.push('Changes stashed successfully');
          
          const stashes = parseStashList();
          const currentStash = stashes[0]; // Most recent stash
          
          suggestedActions.push('Your changes have been saved. Use "git stash pop" to restore them later');
          
          return {
            requestedData: { 
              stashSaved: true,
              currentStash,
              stashes
            },
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: []
          };
        } catch (error) {
          if (error instanceof Error && error.message.includes('No local changes to save')) {
            issuesFound.push('No changes to stash');
            return {
              requestedData: { stashSaved: false },
              automaticActions,
              issuesFound,
              suggestedActions,
              allPRStatus: []
            };
          }
          throw error;
        }
      }
      
      case 'pop':
      case 'apply': {
        const stashRef = input.stashRef !== undefined ? 
          (typeof input.stashRef === 'number' ? `stash@{${input.stashRef}}` : input.stashRef) : 
          'stash@{0}';
        
        const command = `git stash ${input.action}${input.quiet ? ' --quiet' : ''} ${stashRef}`;
        automaticActions.push(`${input.action === 'pop' ? 'Popping' : 'Applying'} stash: ${stashRef}`);
        
        try {
          const output = execSync(command, { encoding: 'utf8' });
          automaticActions.push(`Stash ${input.action === 'pop' ? 'popped' : 'applied'} successfully`);
          
          if (output.includes('CONFLICT')) {
            issuesFound.push('Conflicts detected while applying stash');
            suggestedActions.push('Resolve conflicts and run "git add" on resolved files');
          }
          
          return {
            requestedData: { stashApplied: true },
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: []
          };
        } catch (error) {
          if (error instanceof Error && error.message.includes('is not a stash reference')) {
            issuesFound.push(`Invalid stash reference: ${stashRef}`);
            suggestedActions.push('Run "list" action to see available stashes');
          } else if (error instanceof Error && error.message.includes('CONFLICT')) {
            issuesFound.push('Merge conflicts occurred while applying stash');
            suggestedActions.push('Resolve conflicts manually or use "git stash drop" to discard');
          } else {
            issuesFound.push(`Failed to ${input.action} stash: ${error instanceof Error ? error.message : String(error)}`);
          }
          
          return {
            requestedData: { stashApplied: false, error: error instanceof Error ? error.message : String(error) },
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: []
          };
        }
      }
      
      case 'drop': {
        const stashRef = input.stashRef !== undefined ? 
          (typeof input.stashRef === 'number' ? `stash@{${input.stashRef}}` : input.stashRef) : 
          'stash@{0}';
        
        automaticActions.push(`Dropping stash: ${stashRef}`);
        
        try {
          execSync(`git stash drop ${stashRef}`, { encoding: 'utf8' });
          automaticActions.push('Stash dropped successfully');
          
          return {
            requestedData: { stashDropped: true },
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: []
          };
        } catch (error) {
          issuesFound.push(`Failed to drop stash: ${error instanceof Error ? error.message : String(error)}`);
          return {
            requestedData: { stashDropped: false, error: error instanceof Error ? error.message : String(error) },
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: []
          };
        }
      }
      
      case 'clear': {
        automaticActions.push('Clearing all stashes');
        
        try {
          execSync('git stash clear', { encoding: 'utf8' });
          automaticActions.push('All stashes cleared');
          
          return {
            requestedData: { stashDropped: true },
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: []
          };
        } catch (error) {
          issuesFound.push(`Failed to clear stashes: ${error instanceof Error ? error.message : String(error)}`);
          return {
            requestedData: { stashDropped: false, error: error instanceof Error ? error.message : String(error) },
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: []
          };
        }
      }
      
      case 'show': {
        const stashRef = input.stashRef !== undefined ? 
          (typeof input.stashRef === 'number' ? `stash@{${input.stashRef}}` : input.stashRef) : 
          'stash@{0}';
        
        automaticActions.push(`Showing content of stash: ${stashRef}`);
        
        try {
          const content = execSync(`git stash show -p ${stashRef}`, { encoding: 'utf8' });
          automaticActions.push('Stash content retrieved');
          
          return {
            requestedData: { stashContent: content },
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: []
          };
        } catch (error) {
          issuesFound.push(`Failed to show stash: ${error instanceof Error ? error.message : String(error)}`);
          return {
            requestedData: { error: error instanceof Error ? error.message : String(error) },
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: []
          };
        }
      }
      
      default:
        issuesFound.push(`Unknown action: ${input.action}`);
        return {
          requestedData: { error: `Unknown action: ${input.action}` },
          automaticActions,
          issuesFound,
          suggestedActions,
          allPRStatus: []
        };
    }
  } catch (error) {
    issuesFound.push(`Git stash operation failed: ${error instanceof Error ? error.message : String(error)}`);
    return {
      requestedData: { error: error instanceof Error ? error.message : String(error) },
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus: []
    };
  }
}