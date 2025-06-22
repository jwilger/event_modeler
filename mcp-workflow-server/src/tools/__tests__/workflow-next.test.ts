import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { execSync } from 'child_process';
import { Octokit } from '@octokit/rest';

// Mock child_process
vi.mock('child_process', () => ({
  execSync: vi.fn()
}));

// Mock Octokit
vi.mock('@octokit/rest');

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
    }) as any);
    
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
    
    mockGraphql.mockResolvedValue({
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
    
    mockGraphql.mockResolvedValue({
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

    const result = await workflowNext();

    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'epic_analysis',
      epicNumber: 100,
      epicTitle: 'Epic: Test Epic',
      suggestion: 'Work on sub-issue #10: Sub-issue 1'
    });
    expect(result.requestedData.nextSteps[0].subIssues).toHaveLength(2);
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
    
    mockGraphql.mockResolvedValue({
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

    const result = await workflowNext();

    expect(result.requestedData.nextSteps[0]).toMatchObject({
      action: 'complete_epic',
      epicNumber: 100,
      epicTitle: 'Epic: Completed Epic',
      suggestion: 'No open issues found for this epic. Consider marking it as done.'
    });
  });
});