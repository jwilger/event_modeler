import { Octokit } from '@octokit/rest';
import { execSync } from 'child_process';
import { WorkflowResponse } from '../types.js';
import { getProjectConfig, getMissingConfigFields, createConfigRequest } from '../config.js';
import { workflowMonitorReviews, requestCopilotReReview, type ReviewInfo } from './workflow-monitor-reviews.js';
import { getRepoInfo } from '../utils/github.js';
import { isBranchMerged } from '../utils/git.js';

interface TodoItem {
  text: string;
  checked: boolean;
  index: number;
}

interface NextStepAction {
  action: 'work_on_todo' | 'todos_complete' | 'select_work' | 'epic_analysis' | 'complete_epic' | 'requires_llm_decision' | 'requires_config' | 'address_pr_feedback' | 'review_pr';
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
  epicNumber?: number;
  epicTitle?: string;
  subIssues?: Array<{
    number: number;
    title: string;
    status: string;
  }>;
  // Fields for LLM decision requests
  decisionType?: 'select_next_issue' | 'prioritize_work';
  decisionId?: string; // Unique ID to track this decision
  choices?: Array<{
    id: string | number;
    title: string;
    description?: string;
    metadata?: Record<string, any>;
  }>;
  decisionContext?: {
    prompt: string;
    additionalInfo?: Record<string, any>;
  };
  // Fields for config requests
  missingConfig?: string[];
  configSuggestions?: string[];
  // Fields for PR feedback
  prNumber?: number;
  reviewStatus?: string;
  reviews?: ReviewInfo[];
  prUrl?: string;
  author?: string;
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
    // Check configuration first
    const { config, isComplete } = getProjectConfig();
    
    if (!isComplete) {
      const missingFields = getMissingConfigFields(config);
      const configRequest = createConfigRequest(missingFields);
      
      return {
        requestedData: {
          nextSteps: [{
            action: 'requires_config',
            missingConfig: configRequest.missingFields,
            configSuggestions: configRequest.suggestions,
            suggestion: 'Configuration is incomplete. Please run workflow_configure to set missing values.'
          }],
          context: {
            currentConfig: config
          }
        },
        automaticActions: ['Configuration check failed - missing required fields'],
        issuesFound: [`Missing configuration: ${missingFields.join(', ')}`],
        suggestedActions: configRequest.suggestions,
        allPRStatus: []
      };
    }
    // Get GitHub token from gh CLI
    const token = execSync('gh auth token', { encoding: 'utf8' }).trim();
    const octokit = new Octokit({ auth: token });

    // Get current user
    const currentUser = await getCurrentUser();
    automaticActions.push(`Identified current user: ${currentUser}`);

    // Get repository info from git remote
    const { owner: ownerName, repo: repoName } = getRepoInfo();
    const owner = { login: ownerName };
    const name = repoName;
    automaticActions.push(`Working in repository: ${owner.login}/${name}`);

    // Get current git status early as we need it for context
    const gitStatus = execSync('git status --porcelain', { encoding: 'utf8' });
    const currentBranch = execSync('git branch --show-current', { encoding: 'utf8' }).trim();
    const hasUncommittedChanges = gitStatus.length > 0;
    
    // Check if current branch has been merged
    if (currentBranch !== 'main') {
      const isMerged = await isBranchMerged(currentBranch);
      if (isMerged) {
        automaticActions.push(`Current branch '${currentBranch}' has been merged to main`);
        
        return {
          requestedData: {
            nextSteps: [{
              action: 'select_work',
              suggestion: `Your branch '${currentBranch}' has been merged. Switch to main and pull latest changes before starting new work.`,
              reason: 'Current branch has been merged to main'
            }],
            context: {
              currentBranch,
              hasUncommittedChanges,
              branchMerged: true
            }
          },
          automaticActions,
          issuesFound: [`Branch '${currentBranch}' has been merged - switch to main`],
          suggestedActions: [
            'Run: git checkout main && git pull origin main',
            'Then run workflow_next again to find next task'
          ],
          allPRStatus: []
        };
      }
    }

    // First, check if there are any PRs needing attention
    const reviewStatus = await workflowMonitorReviews({ includeApproved: false, includeDrafts: false });
    const allOpenPRs = reviewStatus.requestedData.reviewsNeedingAttention;
    
    // Separate PRs by author and review status
    const myPRsNeedingAttention = allOpenPRs.filter(pr => 
      pr.author === currentUser && 
      (pr.reviewStatus === 'changes_requested' || pr.reviewStatus === 'has_comments')
    );
    
    const othersPRsToReview = allOpenPRs.filter(pr => {
      if (pr.author === currentUser) return false;
      
      // Check if current user has already reviewed
      const myReview = pr.reviews.find(r => r.reviewer === currentUser);
      
      if (!myReview) {
        // User hasn't reviewed yet
        return true;
      }
      
      // Check if there are new changes since user's review
      const myReviewDate = new Date(myReview.submittedAt);
      const prLastUpdated = new Date(pr.lastUpdated);
      
      // PR was updated after user's review
      return prLastUpdated > myReviewDate;
    });
    
    // Check if any of my PRs need Copilot re-review requested
    for (const pr of allOpenPRs.filter(p => p.author === currentUser)) {
      // Get all Copilot reviews sorted by date
      const copilotReviews = pr.reviews
        .filter(r => 
          r.reviewer === 'copilot-pull-request-reviewer[bot]' || 
          r.reviewer === 'copilot-pull-request-reviewer'
        )
        .sort((a, b) => new Date(b.submittedAt).getTime() - new Date(a.submittedAt).getTime());
      
      if (copilotReviews.length > 0) {
        const latestCopilotReview = copilotReviews[0];
        const reviewDate = new Date(latestCopilotReview.submittedAt);
        const prLastUpdated = new Date(pr.lastUpdated);
        
        // Check if PR was updated after Copilot's latest review
        if (prLastUpdated > reviewDate) {
          // Get commit count to see if there are actual new commits
          try {
            const { owner: ownerName, repo: repoName } = getRepoInfo();
            
            // Get commits since the review
            const commitsQuery = `
              query($owner: String!, $repo: String!, $prNumber: Int!, $since: GitTimestamp!) {
                repository(owner: $owner, name: $repo) {
                  pullRequest(number: $prNumber) {
                    commits(first: 1, since: $since) {
                      totalCount
                    }
                  }
                }
              }
            `;
            
            const result = await octokit.graphql(commitsQuery, {
              owner: ownerName,
              repo: repoName,
              prNumber: pr.prNumber,
              since: reviewDate.toISOString()
            });
            
            const newCommitCount = (result as any).repository.pullRequest.commits.totalCount;
            
            if (newCommitCount > 0) {
              // Only request re-review if there are actual new commits
              automaticActions.push(`PR #${pr.prNumber} has ${newCommitCount} new commits since Copilot's last review, requesting re-review`);
              const reReviewRequested = await requestCopilotReReview(pr.prNumber);
              if (reReviewRequested) {
                automaticActions.push(`Successfully requested Copilot re-review for PR #${pr.prNumber}`);
              } else {
                issuesFound.push(`Failed to request Copilot re-review for PR #${pr.prNumber}`);
              }
            }
          } catch (error) {
            // If we can't check commits, skip re-review request
            console.error('Failed to check for new commits:', error);
          }
        }
      }
    }
    
    // Prioritize: 1) My PRs with feedback, 2) Others' PRs needing review
    if (myPRsNeedingAttention.length > 0) {
      const pr = myPRsNeedingAttention[0]; // Take the highest priority PR
      automaticActions.push(`Found PR #${pr.prNumber} authored by you with review feedback needing attention`);
      
      return {
        requestedData: {
          nextSteps: [{
            action: 'address_pr_feedback',
            prNumber: pr.prNumber,
            title: pr.title,
            reviewStatus: pr.reviewStatus,
            reviews: pr.reviews,
            suggestion: `Address review feedback on your PR #${pr.prNumber}: ${pr.suggestedAction}`,
            prUrl: pr.url
          }],
          context: {
            currentBranch,
            hasUncommittedChanges,
            myPRsNeedingAttention,
            othersPRsToReview
          }
        },
        automaticActions,
        issuesFound,
        suggestedActions: [`Address ${pr.reviewStatus === 'changes_requested' ? 'requested changes' : 'review comments'} on PR #${pr.prNumber}`],
        allPRStatus: []
      };
    } else if (othersPRsToReview.length > 0) {
      const pr = othersPRsToReview[0];
      const hasReviewedBefore = pr.reviews.some(r => r.reviewer === currentUser);
      
      automaticActions.push(`Found PR #${pr.prNumber} by ${pr.author} needing review`);
      
      return {
        requestedData: {
          nextSteps: [{
            action: 'review_pr',
            prNumber: pr.prNumber,
            title: pr.title,
            author: pr.author,
            reviewStatus: pr.reviewStatus,
            suggestion: hasReviewedBefore 
              ? `Check new updates on PR #${pr.prNumber} by ${pr.author} since your last review`
              : `Review PR #${pr.prNumber} by ${pr.author}`,
            prUrl: pr.url
          }],
          context: {
            currentBranch,
            hasUncommittedChanges,
            myPRsNeedingAttention,
            othersPRsToReview
          }
        },
        automaticActions,
        issuesFound,
        suggestedActions: [hasReviewedBefore 
          ? `Check new updates on PR #${pr.prNumber}`
          : `Review PR #${pr.prNumber} by ${pr.author}`],
        allPRStatus: []
      };
    }

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
                    labels(first: 10) {
                      nodes {
                        name
                      }
                    }
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
      projectNumber: config.github.projectNumber!
    });

    // Filter to issues assigned to current user and in progress
    const items = (projectData as any).user.projectV2.items.nodes;
    
    // Separate regular issues and epics
    const allInProgressIssues = items.filter((item: any) => {
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
    
    const inProgressIssues = allInProgressIssues.filter((item: any) => {
      const isEpic = item.content.labels && item.content.labels.nodes.some(
        (label: any) => label.name === 'epic'
      );
      return !isEpic;
    });
    
    const inProgressEpics = allInProgressIssues.filter((item: any) => {
      const isEpic = item.content.labels && item.content.labels.nodes.some(
        (label: any) => label.name === 'epic'
      );
      return isEpic;
    });

    automaticActions.push(`Found ${inProgressIssues.length} issues assigned to ${currentUser} in progress`);
    
    // Check for existing PR on current branch using GraphQL
    let existingPR = null;
    try {
      const prQuery = `
        query($owner: String!, $repo: String!, $headRef: String!) {
          repository(owner: $owner, name: $repo) {
            pullRequests(headRefName: $headRef, states: [OPEN], first: 1) {
              nodes {
                number
                title
                state
              }
            }
          }
        }
      `;
      
      const prResult = await octokit.graphql(prQuery, {
        owner: owner.login,
        repo: name,
        headRef: currentBranch
      });
      
      const prs = (prResult as any).repository.pullRequests.nodes;
      if (prs.length > 0) {
        existingPR = {
          number: prs[0].number,
          title: prs[0].title,
          state: prs[0].state
        };
      }
    } catch (error) {
      // No PR found or error checking
    }

    if (inProgressIssues.length === 0) {
      // No regular issues in progress, check for epics
      if (inProgressEpics.length > 0) {
        // Analyze the first epic
        const epic = inProgressEpics[0].content;
        
        // Get all sub-issues linked to this epic
        const epicQuery = `
          query($owner: String!, $repo: String!, $epicNumber: Int!) {
            repository(owner: $owner, name: $repo) {
              issue(number: $epicNumber) {
                title
                body
                subIssues(first: 100) {
                  nodes {
                    number
                    title
                    state
                    labels(first: 10) {
                      nodes {
                        name
                      }
                    }
                  }
                }
              }
            }
          }
        `;
        
        try {
          const epicData = await octokit.graphql(epicQuery, {
            owner: owner.login,
            repo: name,
            epicNumber: epic.number
          });
          
          const epicIssue = (epicData as any).repository.issue;
          const subIssues = epicIssue.subIssues?.nodes || [];
          const openSubIssues = subIssues.filter((issue: any) => issue.state === 'OPEN');
          
          if (openSubIssues.length === 0) {
            // Epic has no open sub-issues
            return {
              requestedData: {
                nextSteps: [{
                  action: 'complete_epic',
                  epicNumber: epic.number,
                  epicTitle: epic.title,
                  suggestion: 'All sub-issues for this epic are complete. Consider marking the epic as done.'
                }],
                context: {
                  currentBranch,
                  hasUncommittedChanges,
                  existingPR: existingPR ? { number: existingPR.number, title: existingPR.title } : null
                }
              },
              automaticActions,
              issuesFound,
              suggestedActions: [`Mark epic #${epic.number} as complete`],
              allPRStatus: []
            };
          }
          
          // Epic has open sub-issues - request LLM decision if multiple options
          if (openSubIssues.length > 1) {
            // Multiple sub-issues available, request LLM decision
            const decisionId = `epic-${epic.number}-next-issue-${Date.now()}`;
            
            return {
              requestedData: {
                nextSteps: [{
                  action: 'requires_llm_decision',
                  decisionType: 'select_next_issue',
                  decisionId,
                  epicNumber: epic.number,
                  epicTitle: epic.title,
                  choices: openSubIssues.map((issue: any) => ({
                    id: issue.number,
                    title: issue.title,
                    description: `Issue #${issue.number}`,
                    metadata: {
                      state: issue.state,
                      labels: issue.labels?.nodes?.map((l: any) => l.name) || []
                    }
                  })),
                  decisionContext: {
                    prompt: `Which sub-issue of the epic "${epic.title}" should be worked on next? Consider dependencies, logical ordering, and which issues might be foundation work that enables others.`,
                    additionalInfo: {
                      currentBranch,
                      existingPR
                    }
                  }
                }],
                context: {
                  currentBranch,
                  hasUncommittedChanges,
                  existingPR: existingPR ? { number: existingPR.number, title: existingPR.title } : null
                }
              },
              automaticActions,
              issuesFound,
              suggestedActions: ['Awaiting decision on which sub-issue to work on next'],
              allPRStatus: []
            };
          }
          
          // Only one sub-issue, suggest it directly
          const nextIssue = openSubIssues[0];
          
          return {
            requestedData: {
              nextSteps: [{
                action: 'epic_analysis',
                epicNumber: epic.number,
                epicTitle: epic.title,
                suggestion: `Work on sub-issue #${nextIssue.number}: ${nextIssue.title}`,
                subIssues: openSubIssues.map((issue: any) => ({
                  number: issue.number,
                  title: issue.title,
                  status: issue.state
                }))
              }],
              context: {
                currentBranch,
                hasUncommittedChanges,
                existingPR: existingPR ? { number: existingPR.number, title: existingPR.title } : null
              }
            },
            automaticActions,
            issuesFound,
            suggestedActions: [`Start work on issue #${nextIssue.number} from epic #${epic.number}`],
            allPRStatus: []
          };
        } catch (error) {
          // Fallback to searching for issues that mention the epic using GraphQL
          automaticActions.push('Primary query failed, using search fallback');
          
          const searchQuery = `
            query($query: String!) {
              search(query: $query, type: ISSUE, first: 100) {
                nodes {
                  ... on Issue {
                    number
                    title
                    state
                    repository {
                      name
                      owner {
                        login
                      }
                    }
                  }
                }
              }
            }
          `;
          
          const searchResult = await octokit.graphql(searchQuery, {
            query: `repo:${owner.login}/${name} is:issue is:open "#${epic.number}" in:body`
          });
          
          const relatedIssues = ((searchResult as any).search.nodes || [])
            .filter((issue: any) => 
              issue.repository?.owner?.login === owner.login && 
              issue.repository?.name === name
            )
            .map((issue: any) => ({
              number: issue.number,
              title: issue.title,
              state: issue.state
            }));
          
          if (relatedIssues.length === 0) {
            return {
              requestedData: {
                nextSteps: [{
                  action: 'complete_epic',
                  epicNumber: epic.number,
                  epicTitle: epic.title,
                  suggestion: 'No open issues found for this epic. Consider marking it as done.'
                }],
                context: {
                  currentBranch,
                  hasUncommittedChanges,
                  existingPR: existingPR ? { number: existingPR.number, title: existingPR.title } : null
                }
              },
              automaticActions,
              issuesFound,
              suggestedActions: [`Mark epic #${epic.number} as complete`],
              allPRStatus: []
            };
          }
          
          const nextIssue = relatedIssues[0];
          return {
            requestedData: {
              nextSteps: [{
                action: 'epic_analysis',
                epicNumber: epic.number,
                epicTitle: epic.title,
                suggestion: `Work on sub-issue #${nextIssue.number}: ${nextIssue.title}`,
                subIssues: relatedIssues.map((issue: any) => ({
                  number: issue.number,
                  title: issue.title,
                  status: issue.state
                }))
              }],
              context: {
                currentBranch,
                hasUncommittedChanges,
                existingPR: existingPR ? { number: existingPR.number, title: existingPR.title } : null
              }
            },
            automaticActions,
            issuesFound,
            suggestedActions: [`Start work on issue #${nextIssue.number} from epic #${epic.number}`],
            allPRStatus: []
          };
        }
      }
      
      // No issues or epics in progress
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
      const suggestion = existingPR 
        ? `All todos complete. PR #${existingPR.number} exists - check if ready to merge.`
        : 'All todos complete. Create PR for the completed work.';
        
      return {
        requestedData: {
          nextSteps: [{
            action: 'todos_complete',
            issueNumber: issue.number,
            title: issue.title,
            status: 'In Progress',
            suggestion
          }],
          context: {
            totalTodos: todos.length,
            completedTodos: todos.length,
            hasPR: !!existingPR,
            existingPR: existingPR ? { number: existingPR.number, title: existingPR.title } : null,
            currentBranch,
            hasUncommittedChanges
          }
        },
        automaticActions,
        issuesFound,
        suggestedActions: existingPR 
          ? [`Check PR #${existingPR.number} for review status`]
          : ['Create a pull request for the completed work'],
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
          hasUncommittedChanges,
          existingPR: existingPR ? { number: existingPR.number, title: existingPR.title } : null
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