import { execSync } from 'child_process';
import { WorkflowResponse } from '../types.js';
import { promises as fs } from 'fs';
import * as path from 'path';

interface GitCommitInput {
  action: 'stage' | 'unstage' | 'status' | 'commit' | 'amend';
  files?: string[]; // For stage/unstage - if not provided, stage all
  message?: string; // For commit
  issueNumber?: number; // For commit - will be auto-detected if not provided
  all?: boolean; // For stage - stage all tracked files
}

interface FileStatus {
  path: string;
  status: 'modified' | 'added' | 'deleted' | 'renamed' | 'untracked';
  staged: boolean;
}

interface GitCommitResponse extends WorkflowResponse {
  requestedData: {
    stagedFiles?: string[];
    unstagedFiles?: string[];
    fileStatuses?: FileStatus[];
    commitHash?: string;
    commitMessage?: string;
    issueNumber?: number;
  };
}

function parseGitStatus(): FileStatus[] {
  const statusOutput = execSync('git status --porcelain', { encoding: 'utf8' });
  const files: FileStatus[] = [];
  
  for (const line of statusOutput.split('\n')) {
    if (!line.trim()) continue;
    
    const indexStatus = line[0];
    const workingStatus = line[1];
    const filePath = line.substring(3);
    
    let status: FileStatus['status'] = 'modified';
    let staged = false;
    
    // Determine staging status and file status
    if (indexStatus === 'M') {
      staged = true;
      status = 'modified';
    } else if (indexStatus === 'A') {
      staged = true;
      status = 'added';
    } else if (indexStatus === 'D') {
      staged = true;
      status = 'deleted';
    } else if (indexStatus === 'R') {
      staged = true;
      status = 'renamed';
    } else if (workingStatus === 'M') {
      staged = false;
      status = 'modified';
    } else if (workingStatus === 'D') {
      staged = false;
      status = 'deleted';
    } else if (indexStatus === '?' && workingStatus === '?') {
      staged = false;
      status = 'untracked';
    } else if (indexStatus === ' ' && workingStatus === ' ') {
      continue; // Skip clean files
    }
    
    files.push({
      path: filePath,
      status,
      staged,
    });
  }
  
  return files;
}

function getCurrentBranch(): string {
  try {
    return execSync('git branch --show-current', { encoding: 'utf8' }).trim();
  } catch {
    throw new Error('Failed to get current branch');
  }
}

function extractIssueNumber(branchName: string): number | undefined {
  // Match patterns like -123 at the end or issue-123
  const match = branchName.match(/-(\d+)(?:$|[^0-9])/);
  if (match) {
    return parseInt(match[1]);
  }
  return undefined;
}

async function runPreCommitChecks(files: string[]): Promise<{ passed: boolean; output: string }> {
  const rustFiles = files.filter(f => f.endsWith('.rs'));
  const tsFiles = files.filter(f => f.endsWith('.ts') || f.endsWith('.tsx'));
  
  let output = '';
  let passed = true;
  
  // Check Rust files
  if (rustFiles.length > 0) {
    try {
      // Run cargo fmt check
      execSync('cargo fmt -- --check', { encoding: 'utf8' });
      output += 'cargo fmt: ✓\n';
    } catch {
      output += 'cargo fmt: ✗ (run `cargo fmt` to fix)\n';
      passed = false;
    }
    
    try {
      // Run cargo clippy
      execSync('cargo clippy --workspace --all-targets -- -D warnings', { encoding: 'utf8' });
      output += 'cargo clippy: ✓\n';
    } catch {
      output += 'cargo clippy: ✗ (fix clippy warnings)\n';
      passed = false;
    }
  }
  
  // Check TypeScript files
  if (tsFiles.length > 0) {
    try {
      // Check if we're in the mcp-workflow-server directory
      const isInMcpDir = process.cwd().includes('mcp-workflow-server') || 
                        await fs.access(path.join(process.cwd(), 'mcp-workflow-server')).then(() => true).catch(() => false);
      
      if (isInMcpDir) {
        const mcpDir = process.cwd().includes('mcp-workflow-server') ? '.' : 'mcp-workflow-server';
        
        // Run npm run lint
        execSync(`cd ${mcpDir} && npm run lint`, { encoding: 'utf8' });
        output += 'npm run lint: ✓\n';
        
        // Run npm run typecheck
        execSync(`cd ${mcpDir} && npm run typecheck`, { encoding: 'utf8' });
        output += 'npm run typecheck: ✓\n';
      }
    } catch (error) {
      output += `TypeScript checks: ✗ (${error instanceof Error ? error.message : 'unknown error'})\n`;
      passed = false;
    }
  }
  
  return { passed, output };
}

function formatCommitMessage(message: string, issueNumber?: number): string {
  let formattedMessage = message.trim();
  
  // Add issue reference if not already present
  if (issueNumber && !formattedMessage.includes(`#${issueNumber}`)) {
    formattedMessage += ` (#${issueNumber})`;
  }
  
  // Add Claude footer
  formattedMessage += '\n\n🤖 Generated with [Claude Code](https://claude.ai/code)\n\nCo-Authored-By: Claude <noreply@anthropic.com>';
  
  return formattedMessage;
}

export async function gitCommit(input: GitCommitInput): Promise<GitCommitResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];
  
  try {
    switch (input.action) {
      case 'status': {
        const fileStatuses = parseGitStatus();
        const stagedFiles = fileStatuses.filter(f => f.staged).map(f => f.path);
        const unstagedFiles = fileStatuses.filter(f => !f.staged && f.status !== 'untracked').map(f => f.path);
        
        automaticActions.push(`Found ${fileStatuses.length} files with changes`);
        automaticActions.push(`${stagedFiles.length} staged, ${unstagedFiles.length} unstaged`);
        
        return {
          requestedData: {
            fileStatuses,
            stagedFiles,
            unstagedFiles,
          },
          automaticActions,
          issuesFound,
          suggestedActions,
          allPRStatus: [],
        };
      }
      
      case 'stage': {
        const fileStatuses = parseGitStatus();
        let filesToStage: string[] = [];
        
        if (input.files && input.files.length > 0) {
          // Stage specific files
          filesToStage = input.files;
        } else if (input.all) {
          // Stage all tracked files (git add -u)
          execSync('git add -u', { encoding: 'utf8' });
          automaticActions.push('Staged all tracked files with modifications');
          
          const stagedFiles = parseGitStatus().filter(f => f.staged).map(f => f.path);
          return {
            requestedData: { stagedFiles },
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: [],
          };
        } else {
          // Stage all changes (git add .)
          filesToStage = fileStatuses.filter(f => !f.staged).map(f => f.path);
        }
        
        if (filesToStage.length === 0) {
          issuesFound.push('No files to stage');
          return {
            requestedData: {},
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: [],
          };
        }
        
        // Stage the files
        for (const file of filesToStage) {
          try {
            execSync(`git add "${file}"`, { encoding: 'utf8' });
            automaticActions.push(`Staged: ${file}`);
          } catch (error) {
            issuesFound.push(`Failed to stage ${file}: ${error instanceof Error ? error.message : 'unknown error'}`);
          }
        }
        
        const stagedFiles = parseGitStatus().filter(f => f.staged).map(f => f.path);
        
        return {
          requestedData: { stagedFiles },
          automaticActions,
          issuesFound,
          suggestedActions,
          allPRStatus: [],
        };
      }
      
      case 'unstage': {
        const filesToUnstage = input.files || parseGitStatus().filter(f => f.staged).map(f => f.path);
        
        if (filesToUnstage.length === 0) {
          issuesFound.push('No staged files to unstage');
          return {
            requestedData: {},
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: [],
          };
        }
        
        // Unstage the files
        for (const file of filesToUnstage) {
          try {
            execSync(`git reset HEAD "${file}"`, { encoding: 'utf8' });
            automaticActions.push(`Unstaged: ${file}`);
          } catch (error) {
            issuesFound.push(`Failed to unstage ${file}: ${error instanceof Error ? error.message : 'unknown error'}`);
          }
        }
        
        const unstagedFiles = filesToUnstage;
        
        return {
          requestedData: { unstagedFiles },
          automaticActions,
          issuesFound,
          suggestedActions,
          allPRStatus: [],
        };
      }
      
      case 'commit': {
        if (!input.message) {
          throw new Error('Commit message is required');
        }
        
        // Check for staged files
        const stagedFiles = parseGitStatus().filter(f => f.staged).map(f => f.path);
        if (stagedFiles.length === 0) {
          issuesFound.push('No staged files to commit');
          suggestedActions.push('Stage files first using stage action');
          return {
            requestedData: {},
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: [],
          };
        }
        
        automaticActions.push(`Committing ${stagedFiles.length} files`);
        
        // Run pre-commit checks
        const { passed, output } = await runPreCommitChecks(stagedFiles);
        automaticActions.push('Pre-commit checks:');
        automaticActions.push(...output.split('\n').filter(line => line.trim()));
        
        if (!passed) {
          issuesFound.push('Pre-commit checks failed');
          suggestedActions.push('Fix the issues and try again');
          return {
            requestedData: {},
            automaticActions,
            issuesFound,
            suggestedActions,
            allPRStatus: [],
          };
        }
        
        // Auto-detect issue number if not provided
        const issueNumber = input.issueNumber || extractIssueNumber(getCurrentBranch());
        if (issueNumber) {
          automaticActions.push(`Detected issue number: #${issueNumber}`);
        }
        
        // Format commit message
        const commitMessage = formatCommitMessage(input.message, issueNumber);
        
        // Create commit
        try {
          // Write message to temp file to handle multiline properly
          const tempFile = `/tmp/git-commit-msg-${Date.now()}.txt`;
          await fs.writeFile(tempFile, commitMessage);
          
          execSync(`git commit -F "${tempFile}"`, { encoding: 'utf8' });
          await fs.unlink(tempFile);
          
          const commitHash = execSync('git rev-parse HEAD', { encoding: 'utf8' }).trim();
          automaticActions.push(`Created commit: ${commitHash.substring(0, 7)}`);
          
          return {
            requestedData: {
              commitHash,
              commitMessage,
              issueNumber,
            },
            automaticActions,
            issuesFound,
            suggestedActions: ['Push changes when ready'],
            allPRStatus: [],
          };
        } catch (error) {
          throw new Error(`Failed to create commit: ${error instanceof Error ? error.message : 'unknown error'}`);
        }
      }
      
      case 'amend': {
        // Check if we have a commit to amend
        try {
          execSync('git rev-parse HEAD', { encoding: 'utf8' });
        } catch {
          throw new Error('No commits to amend');
        }
        
        // Get current commit message if no new message provided
        let commitMessage: string;
        if (input.message) {
          const issueNumber = input.issueNumber || extractIssueNumber(getCurrentBranch());
          commitMessage = formatCommitMessage(input.message, issueNumber);
        } else {
          // Keep existing message
          commitMessage = execSync('git log -1 --pretty=%B', { encoding: 'utf8' }).trim();
        }
        
        // Check for staged files
        const stagedFiles = parseGitStatus().filter(f => f.staged).map(f => f.path);
        if (stagedFiles.length > 0) {
          automaticActions.push(`Adding ${stagedFiles.length} files to previous commit`);
          
          // Run pre-commit checks
          const { passed, output } = await runPreCommitChecks(stagedFiles);
          automaticActions.push('Pre-commit checks:');
          automaticActions.push(...output.split('\n').filter(line => line.trim()));
          
          if (!passed) {
            issuesFound.push('Pre-commit checks failed');
            suggestedActions.push('Fix the issues and try again');
            return {
              requestedData: {},
              automaticActions,
              issuesFound,
              suggestedActions,
              allPRStatus: [],
            };
          }
        }
        
        // Amend commit
        try {
          const tempFile = `/tmp/git-commit-msg-${Date.now()}.txt`;
          await fs.writeFile(tempFile, commitMessage);
          
          execSync(`git commit --amend -F "${tempFile}"`, { encoding: 'utf8' });
          await fs.unlink(tempFile);
          
          const commitHash = execSync('git rev-parse HEAD', { encoding: 'utf8' }).trim();
          automaticActions.push(`Amended commit: ${commitHash.substring(0, 7)}`);
          
          return {
            requestedData: {
              commitHash,
              commitMessage,
            },
            automaticActions,
            issuesFound,
            suggestedActions: ['Force push if already pushed: git push --force-with-lease'],
            allPRStatus: [],
          };
        } catch (error) {
          throw new Error(`Failed to amend commit: ${error instanceof Error ? error.message : 'unknown error'}`);
        }
      }
      
      default:
        throw new Error(`Unknown action: ${input.action}`);
    }
  } catch (error) {
    issuesFound.push(`Error: ${error instanceof Error ? error.message : String(error)}`);
    
    return {
      requestedData: {},
      automaticActions,
      issuesFound,
      suggestedActions: ['Fix the error and try again'],
      allPRStatus: [],
    };
  }
}