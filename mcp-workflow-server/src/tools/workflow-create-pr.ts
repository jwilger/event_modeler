import { execSync } from 'child_process';
import { Octokit } from '@octokit/rest';
import { WorkflowResponse } from '../types.js';
import { getProjectConfig } from '../config.js';
import { getRepoInfo } from '../utils/github.js';

interface CreatePRInput {
  baseBranch?: string; // defaults to main/master
  draft?: boolean;
}

interface WorkflowCreatePRResponse extends WorkflowResponse {
  requestedData: {
    pr?: {
      number: number;
      url: string;
      title: string;
      draft: boolean;
    };
    error?: string;
  };
}

async function getCurrentUser(octokit: Octokit): Promise<string> {
  try {
    const { data } = await octokit.users.getAuthenticated();
    return data.login;
  } catch {
    throw new Error('Failed to get current GitHub user. Make sure authentication is configured.');
  }
}

function getDefaultBranch(): string {
  try {
    // Try to get the default branch from git
    const remotes = execSync('git remote', { encoding: 'utf8' }).trim().split('\n');
    const remote = remotes.includes('origin') ? 'origin' : remotes[0];
    
    // Get the default branch
    const defaultBranch = execSync(`git symbolic-ref refs/remotes/${remote}/HEAD | sed 's@^refs/remotes/${remote}/@@'`, { encoding: 'utf8' }).trim();
    return defaultBranch || 'main';
  } catch {
    // Fallback to common defaults
    return 'main';
  }
}

function extractIssueNumber(branchName: string): number | null {
  // Try different patterns to extract issue number
  const patterns = [
    /issue-(\d+)/i,
    /\/#?(\d+)/,
    /-(\d+)$/,
    /phase-(\d+)/i
  ];
  
  for (const pattern of patterns) {
    const match = branchName.match(pattern);
    if (match) {
      const num = parseInt(match[1]);
      // For phase patterns, we need to look up the actual issue number
      // Phase 3 corresponds to issue 70 based on our epic
      if (pattern.toString().includes('phase')) {
        const phaseToIssue: Record<number, number> = {
          2: 69,
          3: 70,
          4: 71,
          5: 72
        };
        return phaseToIssue[num] || null;
      }
      return num;
    }
  }
  
  return null;
}

export async function workflowCreatePR(input: CreatePRInput = {}): Promise<WorkflowCreatePRResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];

  try {
    // Check configuration (but don't fail - just skip project board updates)
    const projectConfig = getProjectConfig();

    // Get current branch
    const currentBranch = execSync('git branch --show-current', { encoding: 'utf8' }).trim();
    automaticActions.push(`Current branch: ${currentBranch}`);
    
    if (currentBranch === 'main' || currentBranch === 'master') {
      throw new Error('Cannot create PR from default branch. Please create a feature branch first.');
    }

    // Check for uncommitted changes
    const gitStatus = execSync('git status --porcelain', { encoding: 'utf8' });
    if (gitStatus.length > 0) {
      throw new Error('You have uncommitted changes. Please commit or stash them before creating a PR.');
    }

    // Get repository info
    const { owner, repo } = getRepoInfo();

    // Set up GitHub API
    const token = execSync('gh auth token', { encoding: 'utf8' }).trim();
    const octokit = new Octokit({ auth: token });

    // Check if branch is pushed to remote
    try {
      execSync(`git rev-parse origin/${currentBranch}`, { encoding: 'utf8' });
      automaticActions.push('Branch already exists on remote');
    } catch {
      // Branch doesn't exist on remote, push it
      automaticActions.push('Pushing branch to remote...');
      try {
        execSync(`git push -u origin ${currentBranch}`, { encoding: 'utf8' });
        automaticActions.push('Branch pushed successfully');
      } catch (pushError) {
        throw new Error(`Failed to push branch: ${pushError instanceof Error ? pushError.message : 'Unknown error'}`);
      }
    }

    // Check if PR already exists for this branch
    const existingPRs = await octokit.pulls.list({
      owner,
      repo,
      head: `${owner}:${currentBranch}`,
      state: 'open'
    });

    if (existingPRs.data.length > 0) {
      const existingPR = existingPRs.data[0];
      automaticActions.push('PR already exists for this branch');
      return {
        requestedData: {
          pr: {
            number: existingPR.number,
            url: existingPR.html_url,
            title: existingPR.title,
            draft: existingPR.draft || false
          }
        },
        automaticActions,
        issuesFound: ['PR already exists for this branch'],
        suggestedActions: [`View existing PR: ${existingPR.html_url}`],
        allPRStatus: []
      };
    }

    // Get base branch
    const baseBranch = input.baseBranch || getDefaultBranch();
    automaticActions.push(`Base branch: ${baseBranch}`);

    // Get commits between base and current branch
    const commits = execSync(`git log ${baseBranch}..HEAD --pretty=format:"%H%x00%s%x00%b%x00" --reverse`, { encoding: 'utf8' })
      .trim()
      .split('\x00\n')
      .filter(line => line.length > 0)
      .map(line => {
        const [hash, subject, body] = line.split('\x00');
        return { hash, subject: subject || '', body: body || '' };
      })
      .filter(commit => commit.hash); // Filter out empty commits

    if (commits.length === 0) {
      throw new Error('No commits found between base branch and current branch. Make sure you have commits to create a PR.');
    }

    automaticActions.push(`Found ${commits.length} commits`);

    // Try to find related issue
    const issueNumber = extractIssueNumber(currentBranch);
    let issue = null;
    let issueBody = '';

    if (issueNumber) {
      try {
        const issueResponse = await octokit.issues.get({
          owner,
          repo,
          issue_number: issueNumber
        });
        issue = issueResponse.data;
        issueBody = issue.body || '';
        automaticActions.push(`Found related issue #${issueNumber}: ${issue.title}`);
      } catch {
        automaticActions.push(`Could not fetch issue #${issueNumber}`);
      }
    }

    // Generate PR title
    let prTitle = '';
    if (issue) {
      prTitle = issue.title;
    } else if (commits.length === 1) {
      prTitle = commits[0].subject;
    } else {
      // Use branch name to generate title
      prTitle = currentBranch
        .replace(/^feature\//, '')
        .replace(/[-_]/g, ' ')
        .replace(/\b\w/g, char => char.toUpperCase());
    }

    // Extract acceptance criteria from issue body
    const acceptanceCriteria: string[] = [];
    if (issueBody) {
      const lines = issueBody.split('\n');
      let inAcceptanceCriteria = false;
      
      for (const line of lines) {
        if (line.match(/^#+\s*(acceptance\s*criteria|ac)/i)) {
          inAcceptanceCriteria = true;
          continue;
        }
        if (inAcceptanceCriteria && line.match(/^#+\s/)) {
          inAcceptanceCriteria = false;
        }
        if (inAcceptanceCriteria && line.match(/^\s*-\s*\[\s*\]/)) {
          acceptanceCriteria.push(line);
        }
      }
    }

    // Generate PR body
    let prBody = '## Summary\n\n';
    
    if (issue) {
      prBody += `This PR implements ${issue.title}.\n\n`;
    } else {
      prBody += `This PR includes the following changes:\n\n`;
    }

    // Add commit summary
    if (commits.length > 1) {
      prBody += '## Commits\n\n';
      commits.forEach(commit => {
        prBody += `- ${commit.subject}\n`;
      });
      prBody += '\n';
    }

    // Add related issue section
    if (issue) {
      prBody += '## Related Issue\n\n';
      prBody += `Closes #${issueNumber}\n\n`;
    }

    // Add changes section
    prBody += '## Changes\n\n';
    
    // Try to get file changes summary
    const diffStat = execSync(`git diff ${baseBranch}...HEAD --stat`, { encoding: 'utf8' });
    const changedFiles = diffStat
      .split('\n')
      .filter(line => line.includes('|'))
      .map(line => line.split('|')[0].trim());
    
    if (changedFiles.length > 0) {
      // Group changes by directory
      const changesByDir: Record<string, string[]> = {};
      changedFiles.forEach(file => {
        const dir = file.includes('/') ? file.split('/')[0] : 'root';
        if (!changesByDir[dir]) {
          changesByDir[dir] = [];
        }
        changesByDir[dir].push(file);
      });
      
      Object.entries(changesByDir).forEach(([dir, files]) => {
        if (files.length === 1) {
          prBody += `- Updated \`${files[0]}\`\n`;
        } else {
          prBody += `- Updated ${files.length} files in \`${dir}/\`\n`;
        }
      });
    } else {
      prBody += '- See commit history for detailed changes\n';
    }
    prBody += '\n';

    // Add test plan
    prBody += '## Test Plan\n\n';
    
    if (acceptanceCriteria.length > 0) {
      prBody += 'Based on acceptance criteria:\n';
      acceptanceCriteria.forEach(criterion => {
        prBody += `${criterion}\n`;
      });
    } else {
      prBody += '- [ ] Code builds successfully\n';
      prBody += '- [ ] Tests pass\n';
      prBody += '- [ ] Manual testing completed\n';
    }
    
    prBody += '\n🤖 Generated with [MCP Workflow Server](https://github.com/jwilger/event_modeler)\n';

    // Create the PR
    const pr = await octokit.pulls.create({
      owner,
      repo,
      title: prTitle,
      body: prBody,
      head: currentBranch,
      base: baseBranch,
      draft: input.draft || false
    });

    automaticActions.push(`Created PR #${pr.data.number}: ${pr.data.title}`);
    suggestedActions.push(`View PR: ${pr.data.html_url}`);

    // Auto-assign PR to creator
    try {
      const currentUser = await getCurrentUser(octokit);
      await octokit.issues.addAssignees({
        owner,
        repo,
        issue_number: pr.data.number,
        assignees: [currentUser]
      });
      automaticActions.push(`Assigned PR to @${currentUser}`);
    } catch (error) {
      automaticActions.push(`Could not auto-assign PR: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }

    // Add PR to project board
    try {
      if (projectConfig.isComplete) {
        // First, add the PR to the project
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
        
        const addResult = await octokit.graphql(addToProjectMutation, {
          projectId: projectConfig.config.github.projectId!,
          contentId: pr.data.node_id
        });
        
        const itemId = (addResult as { addProjectV2ItemById: { item: { id: string } } }).addProjectV2ItemById.item.id;
        automaticActions.push('Added PR to project board');
        
        // Then update the status to "In Progress" (since PR is in review)
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
          projectId: projectConfig.config.github.projectId!,
          itemId: itemId,
          fieldId: projectConfig.config.github.statusFieldId!,
          value: { singleSelectOptionId: projectConfig.config.github.statusOptions!.inProgress! }
        });
        
        automaticActions.push('Set PR status to "In Progress" on project board');
      } else {
        automaticActions.push('Project configuration incomplete - skipping project board update');
      }
    } catch (error) {
      automaticActions.push(`Could not add PR to project board: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }

    // Try to set labels if issue has labels
    if (issue && issue.labels && issue.labels.length > 0) {
      try {
        const labelNames = issue.labels
          .filter(label => typeof label !== 'string')
          .map(label => (label as { name: string }).name);
        
        await octokit.issues.update({
          owner,
          repo,
          issue_number: pr.data.number,
          labels: labelNames
        });
        automaticActions.push(`Applied ${labelNames.length} labels from issue`);
      } catch {
        automaticActions.push('Could not apply labels from issue');
      }
    }

    return {
      requestedData: {
        pr: {
          number: pr.data.number,
          url: pr.data.html_url,
          title: pr.data.title,
          draft: pr.data.draft || false
        }
      },
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus: []
    };

  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    issuesFound.push(`Error: ${errorMessage}`);
    
    return {
      requestedData: {
        error: errorMessage
      },
      automaticActions,
      issuesFound,
      suggestedActions: ['Fix the error and try again'],
      allPRStatus: []
    };
  }
}