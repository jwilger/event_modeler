import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { execSync } from 'child_process';
import { Octokit } from '@octokit/rest';

// Mock child_process
vi.mock('child_process', () => ({
  execSync: vi.fn()
}));

// Mock Octokit
vi.mock('@octokit/rest');

// Mock config
vi.mock('../../config.js', () => ({
  getProjectConfig: vi.fn(() => ({
    config: {
      github: {
        projectNumber: 9,
        projectId: 'PVT_test',
        statusFieldId: 'PVTSSF_test',
        statusOptions: {
          todo: 'PVTSSO_todo',
          inProgress: 'PVTSSO_inprogress',
          done: 'PVTSSO_done',
        },
      },
    },
    isComplete: true,
  })),
}));

// Mock workflow-monitor-reviews
vi.mock('../workflow-monitor-reviews.js', () => ({
  workflowMonitorReviews: vi.fn(() => ({
    requestedData: {
      reviewsNeedingAttention: []
    }
  })),
  requestCopilotReReview: vi.fn(() => false)
}));

// Mock utils
vi.mock('../../utils/github.js', () => ({
  getRepoInfo: vi.fn(() => ({ owner: 'jwilger', repo: 'event_modeler' }))
}));

vi.mock('../../utils/git.js', () => ({
  isBranchMerged: vi.fn(() => Promise.resolve(false))
}));

describe('workflowNext', () => {
  let mockGraphql: ReturnType<typeof vi.fn>;
  let mockExecSync: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.clearAllMocks();
    
    mockExecSync = vi.mocked(execSync);
    mockGraphql = vi.fn();
    
    // Mock Octokit constructor
    vi.mocked(Octokit).mockImplementation(() => ({
      graphql: mockGraphql
    }) as unknown as Octokit);
    
    // Default mock setup for execSync
    mockExecSync.mockImplementation((cmd: string) => {
      if (cmd === 'gh auth token') return 'mock-token\n';
      if (cmd === 'gh api user --jq .login') return 'testuser\n';
      if (cmd === 'gh repo view --json owner,name') {
        return JSON.stringify({ owner: { login: 'jwilger' }, name: 'event_modeler' });
      }
      if (cmd === 'git status --porcelain') return '';
      if (cmd === 'git branch --show-current') return 'feature/test-branch\n';
      if (cmd.startsWith('gh pr list --head')) return '[]';
      return '';
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should return select_work action when no issues are in progress', async () => {
    const { workflowNext } = await import('../workflow-next.js');
    
    mockGraphql.mockResolvedValue({
      user: {
        projectV2: {
          items: {
            nodes: []
          }
        }
      }
    });

    const result = await workflowNext();

    expect(result.requestedData.nextSteps).toHaveLength(1);
    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'select_work',
      projectUrl: 'https://github.com/users/jwilger/projects/9',
      reason: 'No issues in progress. Visit project board to select next item.'
    });
  });

  it('should return work_on_todo action when there are uncompleted todos', async () => {
    const { workflowNext } = await import('../workflow-next.js');
    
    mockGraphql.mockResolvedValue({
      user: {
        projectV2: {
          items: {
            nodes: [{
              id: 'test-item-id',
              content: {
                number: 42,
                title: 'Test Issue',
                body: '## Tasks\n- [x] Completed task\n- [ ] Next task to do\n- [ ] Another pending task',
                state: 'OPEN',
                assignees: {
                  nodes: [{ login: 'testuser' }]
                },
                labels: {
                  nodes: []
                }
              },
              fieldValues: {
                nodes: [{
                  name: 'In Progress',
                  field: { name: 'Status' }
                }]
              }
            }]
          }
        }
      }
    });

    const result = await workflowNext();

    expect(result.requestedData.nextSteps).toHaveLength(1);
    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'work_on_todo',
      issueNumber: 42,
      title: 'Test Issue',
      todoItem: 'Next task to do',
      todoIndex: 1,
      totalTodos: 3,
      completedTodos: 1
    });
  });

  it('should return todos_complete action when all todos are done', async () => {
    const { workflowNext } = await import('../workflow-next.js');
    
    mockGraphql.mockResolvedValue({
      user: {
        projectV2: {
          items: {
            nodes: [{
              id: 'test-item-id',
              content: {
                number: 42,
                title: 'Test Issue',
                body: '## Tasks\n- [x] Completed task 1\n- [x] Completed task 2\n- [x] Completed task 3',
                state: 'OPEN',
                assignees: {
                  nodes: [{ login: 'testuser' }]
                },
                labels: {
                  nodes: []
                }
              },
              fieldValues: {
                nodes: [{
                  name: 'In Progress',
                  field: { name: 'Status' }
                }]
              }
            }]
          }
        }
      }
    });

    const result = await workflowNext();

    expect(result.requestedData.nextSteps).toHaveLength(1);
    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'todos_complete',
      issueNumber: 42,
      title: 'Test Issue',
      suggestion: 'All todos complete. Create PR for the completed work.'
    });
  });

  it('should handle errors gracefully', async () => {
    const { workflowNext } = await import('../workflow-next.js');
    
    mockExecSync.mockImplementation(() => {
      throw new Error('gh not authenticated');
    });

    const result = await workflowNext();

    expect(result.issuesFound).toContain('Error: gh not authenticated');
    expect(result.suggestedActions).toContain('Check that gh CLI is authenticated and has access to the repository');
    expect(result.requestedData.nextSteps).toHaveLength(0);
  });

  it('should parse various todo formats correctly', async () => {
    const { workflowNext } = await import('../workflow-next.js');
    
    mockGraphql.mockResolvedValue({
      user: {
        projectV2: {
          items: {
            nodes: [{
              id: 'test-item-id',
              content: {
                number: 42,
                title: 'Test Issue',
                body: `## Tasks
- [x] Completed with capital X
- [X] ANOTHER COMPLETED
- [ ] Pending task
  - [ ] Nested todo (should be captured)
    - [x] Deeply nested completed
- [ ] Another valid todo with space in brackets
-[] Another invalid (no space)`,
                state: 'OPEN',
                assignees: {
                  nodes: [{ login: 'testuser' }]
                },
                labels: {
                  nodes: []
                }
              },
              fieldValues: {
                nodes: [{
                  name: 'In Progress',
                  field: { name: 'Status' }
                }]
              }
            }]
          }
        }
      }
    });

    const result = await workflowNext();

    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'work_on_todo',
      todoItem: 'Pending task',
      totalTodos: 6,  // 6 valid todos total
      completedTodos: 3
    });
  });

  it('should detect existing PR and update suggestion accordingly', async () => {
    const { workflowNext } = await import('../workflow-next.js');
    
    // Mock PR exists for the branch
    mockExecSync.mockImplementation((cmd: string) => {
      if (cmd === 'gh auth token') return 'mock-token\n';
      if (cmd === 'gh api user --jq .login') return 'testuser\n';
      if (cmd === 'gh repo view --json owner,name') {
        return JSON.stringify({ owner: { login: 'jwilger' }, name: 'event_modeler' });
      }
      if (cmd === 'git status --porcelain') return '';
      if (cmd === 'git branch --show-current') return 'feature/test-branch\n';
      if (cmd.startsWith('gh pr list --head')) {
        return JSON.stringify([{
          number: 123,
          title: 'Test PR',
          state: 'OPEN'
        }]);
      }
      return '';
    });
    
    mockGraphql
      .mockResolvedValueOnce({
        user: {
          projectV2: {
            items: {
              nodes: [{
                id: 'test-item-id',
                content: {
                  number: 42,
                  title: 'Test Issue',
                  body: '## Tasks\n- [x] Completed task 1\n- [x] Completed task 2',
                  state: 'OPEN',
                  assignees: {
                    nodes: [{ login: 'testuser' }]
                  },
                  labels: {
                    nodes: []
                  }
                },
                fieldValues: {
                  nodes: [{
                    name: 'In Progress',
                    field: { name: 'Status' }
                  }]
                }
              }]
            }
          }
        }
      })
      .mockResolvedValueOnce({
        repository: {
          pullRequests: {
            nodes: [{
              number: 123,
              title: 'Test PR',
              state: 'OPEN'
            }]
          }
        }
      });

    const result = await workflowNext();

    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'todos_complete',
      issueNumber: 42,
      title: 'Test Issue',
      suggestion: 'All todos complete. PR #123 exists - check if ready to merge.'
    });
    expect(result.requestedData.context.hasPR).toBe(true);
    expect(result.requestedData.context.existingPR).toEqual({
      number: 123,
      title: 'Test PR'
    });
  });

  it('should filter out epic issues', async () => {
    const { workflowNext } = await import('../workflow-next.js');
    
    mockGraphql.mockResolvedValue({
      user: {
        projectV2: {
          items: {
            nodes: [
              {
                id: 'epic-item-id',
                content: {
                  number: 1,
                  title: 'Epic Issue',
                  body: 'Epic without todos',
                  state: 'OPEN',
                  assignees: {
                    nodes: [{ login: 'testuser' }]
                  },
                  labels: {
                    nodes: [{ name: 'epic' }]
                  }
                },
                fieldValues: {
                  nodes: [{
                    name: 'In Progress',
                    field: { name: 'Status' }
                  }]
                }
              },
              {
                id: 'regular-item-id',
                content: {
                  number: 2,
                  title: 'Regular Issue',
                  body: '- [ ] Task to do',
                  state: 'OPEN',
                  assignees: {
                    nodes: [{ login: 'testuser' }]
                  },
                  labels: {
                    nodes: []
                  }
                },
                fieldValues: {
                  nodes: [{
                    name: 'In Progress',
                    field: { name: 'Status' }
                  }]
                }
              }
            ]
          }
        }
      }
    });

    const result = await workflowNext();

    // Should pick the regular issue, not the epic
    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'work_on_todo',
      issueNumber: 2,
      title: 'Regular Issue',
      todoItem: 'Task to do'
    });
  });

  it('should analyze epic when no regular issues are in progress', async () => {
    const { workflowNext } = await import('../workflow-next.js');
    
    // Mock GitHub CLI to return issue search results
    mockExecSync.mockImplementation((cmd: string) => {
      if (cmd === 'gh auth token') return 'mock-token\n';
      if (cmd === 'gh api user --jq .login') return 'testuser\n';
      if (cmd === 'gh repo view --json owner,name') {
        return JSON.stringify({ owner: { login: 'jwilger' }, name: 'event_modeler' });
      }
      if (cmd === 'git status --porcelain') return '';
      if (cmd === 'git branch --show-current') return 'main\n';
      if (cmd.startsWith('gh pr list --head')) return '[]';
      if (cmd.includes('gh issue list --search')) {
        return JSON.stringify([
          { number: 10, title: 'Sub-issue 1', state: 'OPEN' },
          { number: 11, title: 'Sub-issue 2', state: 'OPEN' }
        ]);
      }
      return '';
    });
    
    // Mock GraphQL to handle different query types
    mockGraphql.mockImplementation((query: string) => {
      // Check if this is the project query
      if (query.includes('projectV2')) {
        return Promise.resolve({
          user: {
            projectV2: {
              items: {
                nodes: [{
                  id: 'epic-item-id',
                  content: {
                    number: 100,
                    title: 'Epic: Test Epic',
                    body: 'Epic description',
                    state: 'OPEN',
                    assignees: {
                      nodes: [{ login: 'testuser' }]
                    },
                    labels: {
                      nodes: [{ name: 'epic' }]
                    }
                  },
                  fieldValues: {
                    nodes: [{
                      name: 'In Progress',
                      field: { name: 'Status' }
                    }]
                  }
                }]
              }
            }
          }
        });
      }
      // Check if this is the epic sub-issues query
      else if (query.includes('subIssues')) {
        return Promise.resolve({
          repository: {
            issue: {
              title: 'Epic: Test Epic',
              body: 'Epic description',
              subIssues: {
                nodes: [
                  { number: 10, title: 'Sub-issue 1', state: 'OPEN', labels: { nodes: [] } },
                  { number: 11, title: 'Sub-issue 2', state: 'OPEN', labels: { nodes: [] } }
                ]
              }
            }
          }
        });
      }
      // Check if this is the search query
      else if (query.includes('search')) {
        return Promise.resolve({
          search: {
            nodes: [
              { 
                number: 10, 
                title: 'Sub-issue 1', 
                state: 'OPEN',
                repository: {
                  name: 'event_modeler',
                  owner: { login: 'jwilger' }
                }
              },
              { 
                number: 11, 
                title: 'Sub-issue 2', 
                state: 'OPEN',
                repository: {
                  name: 'event_modeler',
                  owner: { login: 'jwilger' }
                }
              }
            ]
          }
        });
      }
      // Default response
      return Promise.resolve({});
    });

    const result = await workflowNext();

    // Debug output
    if (!result.requestedData.nextSteps[0]) {
      console.error('No next steps returned');
      console.error('Issues found:', result.issuesFound);
      console.error('Automatic actions:', result.automaticActions);
    }

    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'requires_llm_decision',  // With 2 sub-issues, it asks for LLM decision
      decisionType: 'select_next_issue',
      epicNumber: 100,
      epicTitle: 'Epic: Test Epic',
    });
    expect(result.requestedData.nextSteps[0].choices).toHaveLength(2);
  });

  it('should suggest completing epic when no sub-issues are open', async () => {
    const { workflowNext } = await import('../workflow-next.js');
    
    // Mock empty issue search
    mockExecSync.mockImplementation((cmd: string) => {
      if (cmd === 'gh auth token') return 'mock-token\n';
      if (cmd === 'gh api user --jq .login') return 'testuser\n';
      if (cmd === 'gh repo view --json owner,name') {
        return JSON.stringify({ owner: { login: 'jwilger' }, name: 'event_modeler' });
      }
      if (cmd === 'git status --porcelain') return '';
      if (cmd === 'git branch --show-current') return 'main\n';
      if (cmd.startsWith('gh pr list --head')) return '[]';
      if (cmd.includes('gh issue list --search')) {
        return '[]'; // No sub-issues found
      }
      return '';
    });
    
    // Mock GraphQL to handle different query types
    mockGraphql.mockImplementation((query: string) => {
      // Check if this is the project query
      if (query.includes('projectV2')) {
        return Promise.resolve({
          user: {
            projectV2: {
              items: {
                nodes: [{
                  id: 'epic-item-id',
                  content: {
                    number: 100,
                    title: 'Epic: Completed Epic',
                    body: 'Epic with no open issues',
                    state: 'OPEN',
                    assignees: {
                      nodes: [{ login: 'testuser' }]
                    },
                    labels: {
                      nodes: [{ name: 'epic' }]
                    }
                  },
                  fieldValues: {
                    nodes: [{
                      name: 'In Progress',
                      field: { name: 'Status' }
                    }]
                  }
                }]
              }
            }
          }
        });
      }
      // Check if this is the epic sub-issues query
      else if (query.includes('subIssues')) {
        return Promise.resolve({
          repository: {
            issue: {
              title: 'Epic: Completed Epic',
              body: 'Epic with no open issues',
              subIssues: {
                nodes: [] // No sub-issues
              }
            }
          }
        });
      }
      // Check if this is the search query
      else if (query.includes('search')) {
        return Promise.resolve({
          search: {
            nodes: [] // No sub-issues found in search either
          }
        });
      }
      // Default response
      return Promise.resolve({});
    });

    const result = await workflowNext();

    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'complete_epic',
      epicNumber: 100,
      epicTitle: 'Epic: Completed Epic',
      suggestion: 'All sub-issues for this epic are complete. Consider marking the epic as done.'
    });
  });

  it('should detect when current branch corresponds to in-progress issue without PR', async () => {
    const { workflowNext } = await import('../workflow-next.js');
    
    mockExecSync.mockImplementation((cmd: string) => {
      if (cmd === 'gh auth token') return 'mock-token\n';
      if (cmd === 'gh api user --jq .login') return 'testuser\n';
      if (cmd === 'gh repo view --json owner,name') {
        return JSON.stringify({ owner: { login: 'jwilger' }, name: 'event_modeler' });
      }
      if (cmd === 'git status --porcelain') return '';
      if (cmd === 'git branch --show-current') return 'feature/some-feature-78\n';
      if (cmd === 'git symbolic-ref refs/remotes/origin/HEAD') return 'refs/remotes/origin/main\n';
      if (cmd === 'git rev-list --count origin/main..HEAD') return '3\n'; // Has commits
      if (cmd.startsWith('gh pr list --head')) return '[]';
      return '';
    });
    
    mockGraphql
      .mockResolvedValueOnce({
        user: {
          projectV2: {
            items: {
              nodes: [{
                id: 'test-item-id',
                content: {
                  number: 78,
                  title: 'Test Issue #78',
                  body: '## Tasks\n- [ ] Todo 1\n- [ ] Todo 2',
                  state: 'OPEN',
                  assignees: {
                    nodes: [{ login: 'testuser' }]
                  },
                  labels: {
                    nodes: []
                  }
                },
                fieldValues: {
                  nodes: [{
                    name: 'In Progress',
                    field: { name: 'Status' }
                  }]
                }
              }]
            }
          }
        }
      })
      .mockResolvedValueOnce({
        repository: {
          pullRequests: {
            nodes: [] // No PR exists
          }
        }
      });

    const result = await workflowNext();

    expect(result.requestedData.nextSteps).toHaveLength(1);
    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'todos_complete',
      issueNumber: 78,
      title: 'Test Issue #78',
      suggestion: "You have commits for issue #78 on branch 'feature/some-feature-78'. Create a PR before moving to the next issue."
    });
    expect(result.issuesFound).toContain("Branch 'feature/some-feature-78' has commits but no PR");
    expect(result.suggestedActions).toContain('Create a PR for issue #78');
  });

  it('should not suggest PR creation if branch has no commits', async () => {
    const { workflowNext } = await import('../workflow-next.js');
    
    mockExecSync.mockImplementation((cmd: string) => {
      if (cmd === 'gh auth token') return 'mock-token\n';
      if (cmd === 'gh api user --jq .login') return 'testuser\n';
      if (cmd === 'gh repo view --json owner,name') {
        return JSON.stringify({ owner: { login: 'jwilger' }, name: 'event_modeler' });
      }
      if (cmd === 'git status --porcelain') return '';
      if (cmd === 'git branch --show-current') return 'feature/no-commits-78\n';
      if (cmd === 'git symbolic-ref refs/remotes/origin/HEAD') return 'refs/remotes/origin/main\n';
      if (cmd === 'git rev-list --count origin/main..HEAD') return '0\n'; // No commits
      if (cmd.startsWith('gh pr list --head')) return '[]';
      return '';
    });
    
    mockGraphql.mockResolvedValue({
      user: {
        projectV2: {
          items: {
            nodes: [{
              id: 'test-item-id',
              content: {
                number: 78,
                title: 'Test Issue #78',
                body: '## Tasks\n- [ ] Todo 1\n- [ ] Todo 2',
                state: 'OPEN',
                assignees: {
                  nodes: [{ login: 'testuser' }]
                },
                labels: {
                  nodes: []
                }
              },
              fieldValues: {
                nodes: [{
                  name: 'In Progress',
                  field: { name: 'Status' }
                }]
              }
            }]
          }
        }
      }
    });

    const result = await workflowNext();

    // Should proceed with normal todo workflow since there are no commits
    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'work_on_todo',
      issueNumber: 78,
      todoItem: 'Todo 1'
    });
  });

  it('should handle various branch name patterns when extracting issue number', async () => {
    const { workflowNext } = await import('../workflow-next.js');
    
    const testBranchNames = [
      'feature/add-new-feature-123',
      'fix/issue-456',
      'bugfix/fix-something-789',
      'issue-321',
      'feat-999-new-thing',
      'hotfix/urgent-555'
    ];
    
    for (const branchName of testBranchNames) {
      vi.clearAllMocks();
      
      const expectedIssueNumber = parseInt(branchName.match(/-(\d+)/)![1], 10);
      
      mockExecSync.mockImplementation((cmd: string) => {
        if (cmd === 'gh auth token') return 'mock-token\n';
        if (cmd === 'gh api user --jq .login') return 'testuser\n';
        if (cmd === 'gh repo view --json owner,name') {
          return JSON.stringify({ owner: { login: 'jwilger' }, name: 'event_modeler' });
        }
        if (cmd === 'git status --porcelain') return '';
        if (cmd === 'git branch --show-current') return `${branchName}\n`;
        if (cmd === 'git symbolic-ref refs/remotes/origin/HEAD') return 'refs/remotes/origin/main\n';
        if (cmd === 'git rev-list --count origin/main..HEAD') return '1\n';
        if (cmd.startsWith('gh pr list --head')) return '[]';
        return '';
      });
      
      mockGraphql
        .mockResolvedValueOnce({
          user: {
            projectV2: {
              items: {
                nodes: [{
                  id: 'test-item-id',
                  content: {
                    number: expectedIssueNumber,
                    title: `Test Issue #${expectedIssueNumber}`,
                    body: '## Tasks\n- [ ] Todo',
                    state: 'OPEN',
                    assignees: {
                      nodes: [{ login: 'testuser' }]
                    },
                    labels: {
                      nodes: []
                    }
                  },
                  fieldValues: {
                    nodes: [{
                      name: 'In Progress',
                      field: { name: 'Status' }
                    }]
                  }
                }]
              }
            }
          }
        })
        .mockResolvedValueOnce({
          repository: {
            pullRequests: {
              nodes: []
            }
          }
        });

      const result = await workflowNext();
      
      expect(result.automaticActions).toContain(
        `Detected issue #${expectedIssueNumber} from branch name: ${branchName}`
      );
    }
  });

  it('should ignore branch if issue number is not in progress', async () => {
    const { workflowNext } = await import('../workflow-next.js');
    
    mockExecSync.mockImplementation((cmd: string) => {
      if (cmd === 'gh auth token') return 'mock-token\n';
      if (cmd === 'gh api user --jq .login') return 'testuser\n';
      if (cmd === 'gh repo view --json owner,name') {
        return JSON.stringify({ owner: { login: 'jwilger' }, name: 'event_modeler' });
      }
      if (cmd === 'git status --porcelain') return '';
      if (cmd === 'git branch --show-current') return 'feature/not-in-progress-99\n';
      if (cmd === 'git symbolic-ref refs/remotes/origin/HEAD') return 'refs/remotes/origin/main\n';
      if (cmd === 'git rev-list --count origin/main..HEAD') return '1\n';
      if (cmd.startsWith('gh pr list --head')) return '[]';
      return '';
    });
    
    mockGraphql.mockResolvedValue({
      user: {
        projectV2: {
          items: {
            nodes: [] // No issues in progress
          }
        }
      }
    });

    const result = await workflowNext();

    // Should not suggest PR creation since issue is not in progress
    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'select_work',
      reason: 'No issues in progress. Visit project board to select next item.'
    });
  });

  it('should fallback to origin/master when default branch detection fails', async () => {
    const { workflowNext } = await import('../workflow-next.js');
    
    mockExecSync.mockImplementation((cmd: string) => {
      if (cmd === 'gh auth token') return 'mock-token\n';
      if (cmd === 'gh api user --jq .login') return 'testuser\n';
      if (cmd === 'gh repo view --json owner,name') {
        return JSON.stringify({ owner: { login: 'jwilger' }, name: 'event_modeler' });
      }
      if (cmd === 'git status --porcelain') return '';
      if (cmd === 'git branch --show-current') return 'feature/test-fallback-78\n';
      if (cmd === 'git symbolic-ref refs/remotes/origin/HEAD') {
        throw new Error('Command failed'); // Simulate failure
      }
      if (cmd === 'git rev-list --count origin/master..HEAD') return '2\n'; // Should use master
      if (cmd.startsWith('gh pr list --head')) return '[]';
      return '';
    });
    
    mockGraphql
      .mockResolvedValueOnce({
        user: {
          projectV2: {
            items: {
              nodes: [{
                id: 'test-item-id',
                content: {
                  number: 78,
                  title: 'Test Issue #78',
                  body: '## Tasks\n- [ ] Todo',
                  state: 'OPEN',
                  assignees: {
                    nodes: [{ login: 'testuser' }]
                  },
                  labels: {
                    nodes: []
                  }
                },
                fieldValues: {
                  nodes: [{
                    name: 'In Progress',
                    field: { name: 'Status' }
                  }]
                }
              }]
            }
          }
        }
      })
      .mockResolvedValueOnce({
        repository: {
          pullRequests: {
            nodes: []
          }
        }
      });

    const result = await workflowNext();

    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'todos_complete',
      issueNumber: 78,
      suggestion: expect.stringContaining('Create a PR')
    });
  });
});