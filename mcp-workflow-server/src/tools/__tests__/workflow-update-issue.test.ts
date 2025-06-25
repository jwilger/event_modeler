import { describe, it, expect, vi, beforeEach } from 'vitest';
import { workflowUpdateIssue } from '../workflow-update-issue.js';
import * as githubUtils from '../../utils/github.js';
import * as authUtils from '../../utils/auth.js';
import * as config from '../../config.js';
import { Octokit } from '@octokit/rest';

vi.mock('../../utils/github.js');
vi.mock('../../utils/auth.js');
vi.mock('../../config.js');
vi.mock('@octokit/rest');

describe('workflowUpdateIssue', () => {
  const mockOctokit = {
    graphql: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(Octokit).mockImplementation(() => mockOctokit as unknown as Octokit);
    vi.mocked(authUtils.getGitHubToken).mockReturnValue('test-token');
    vi.mocked(githubUtils.getRepoInfo).mockReturnValue({
      owner: 'testowner',
      repo: 'testrepo',
    });
    vi.mocked(config.getProjectConfig).mockReturnValue({
      isComplete: true,
      config: {
        github: {
          projectNumber: 1,
          projectId: 'PVT_test',
          statusFieldId: 'PVTSSF_status',
          statusOptions: {
            todo: 'PVTSSF_todo',
            inProgress: 'PVTSSF_inprogress',
            done: 'PVTSSF_done',
          },
        },
      },
    });
  });

  it('should update issue status successfully', async () => {
    const mockProjectData = {
      user: {
        projectV2: {
          id: 'PVT_test',
          fields: {
            nodes: [
              {
                id: 'PVTSSF_status',
                name: 'Status',
                options: [
                  { id: 'PVTSSF_todo', name: 'Todo' },
                  { id: 'PVTSSF_inprogress', name: 'In Progress' },
                  { id: 'PVTSSF_done', name: 'Done' },
                ],
              },
            ],
          },
          items: {
            nodes: [
              {
                id: 'PVTI_item1',
                content: { number: 123 },
              },
            ],
          },
        },
      },
    };

    mockOctokit.graphql
      .mockResolvedValueOnce(mockProjectData) // For query
      .mockResolvedValueOnce({ updateProjectV2ItemFieldValue: { projectV2Item: { id: 'PVTI_item1' } } }); // For mutation

    const result = await workflowUpdateIssue({
      issueNumber: 123,
      status: 'in_progress',
    });

    expect(result.requestedData!).toEqual({
      issueNumber: 123,
      updatedFields: {
        status: 'In Progress',
      },
      projectItemId: 'PVTI_item1',
    });
    expect(result.automaticActions).toContain('Updated status to "In Progress"');
    expect(result.issuesFound).toHaveLength(0);
  });

  it('should handle status values with multiple underscores', async () => {
    const mockProjectData = {
      user: {
        projectV2: {
          id: 'PVT_test',
          fields: {
            nodes: [
              {
                id: 'PVTSSF_status',
                name: 'Status',
                options: [
                  { id: 'PVTSSF_waiting', name: 'Waiting For Review' },
                ],
              },
            ],
          },
          items: {
            nodes: [
              {
                id: 'PVTI_item1',
                content: { number: 123 },
              },
            ],
          },
        },
      },
    };

    mockOctokit.graphql
      .mockResolvedValueOnce(mockProjectData) // For query
      .mockResolvedValueOnce({ updateProjectV2ItemFieldValue: { projectV2Item: { id: 'PVTI_item1' } } }); // For mutation

    // Test with a status that has multiple underscores (not in the type definition)
    const result = await workflowUpdateIssue({
      issueNumber: 123,
      // @ts-expect-error - Testing with a status value not in the type definition
      status: 'waiting_for_review',
    });

    expect(result.requestedData!).toEqual({
      issueNumber: 123,
      updatedFields: {
        status: 'Waiting For Review',
      },
      projectItemId: 'PVTI_item1',
    });
    expect(result.automaticActions).toContain('Updated status to "Waiting For Review"');
    expect(result.issuesFound).toHaveLength(0);
  });

  it('should update multiple fields successfully', async () => {
    const mockProjectData = {
      user: {
        projectV2: {
          id: 'PVT_test',
          fields: {
            nodes: [
              {
                id: 'PVTSSF_status',
                name: 'Status',
                options: [
                  { id: 'PVTSSF_todo', name: 'Todo' },
                  { id: 'PVTSSF_inprogress', name: 'In Progress' },
                  { id: 'PVTSSF_done', name: 'Done' },
                ],
              },
              {
                id: 'PVTSSF_type',
                name: 'Type',
                options: [
                  { id: 'PVTSSF_bug', name: 'Bug' },
                  { id: 'PVTSSF_feature', name: 'Feature' },
                ],
              },
              {
                id: 'PVTSSF_priority',
                name: 'Priority',
                options: [
                  { id: 'PVTSSF_low', name: 'Low' },
                  { id: 'PVTSSF_high', name: 'High' },
                ],
              },
            ],
          },
          items: {
            nodes: [
              {
                id: 'PVTI_item1',
                content: { number: 123 },
              },
            ],
          },
        },
      },
    };

    mockOctokit.graphql
      .mockResolvedValueOnce(mockProjectData) // For query
      .mockResolvedValueOnce({ updateProjectV2ItemFieldValue: { projectV2Item: { id: 'PVTI_item1' } } }) // For status
      .mockResolvedValueOnce({ updateProjectV2ItemFieldValue: { projectV2Item: { id: 'PVTI_item1' } } }) // For type
      .mockResolvedValueOnce({ updateProjectV2ItemFieldValue: { projectV2Item: { id: 'PVTI_item1' } } }); // For priority

    const result = await workflowUpdateIssue({
      issueNumber: 123,
      status: 'done',
      type: 'bug',
      priority: 'high',
    });

    expect(result.requestedData!).toEqual({
      issueNumber: 123,
      updatedFields: {
        status: 'Done',
        type: 'Bug',
        priority: 'High',
      },
      projectItemId: 'PVTI_item1',
    });
    expect(result.automaticActions).toContain('Updated status to "Done"');
    expect(result.automaticActions).toContain('Updated type to "Bug"');
    expect(result.automaticActions).toContain('Updated priority to "High"');
    expect(result.issuesFound).toHaveLength(0);
  });

  it('should handle issue not found in project', async () => {
    const mockProjectData = {
      user: {
        projectV2: {
          id: 'PVT_test',
          fields: { nodes: [] },
          items: { nodes: [] },
        },
      },
    };

    mockOctokit.graphql.mockResolvedValueOnce(mockProjectData);

    const result = await workflowUpdateIssue({
      issueNumber: 999,
      status: 'done',
    });

    expect(result.requestedData!.error).toBe('Issue #999 not found in project');
    expect(result.issuesFound).toContain('Error: Issue #999 not found in project');
  });

  it('should handle invalid field value', async () => {
    const mockProjectData = {
      user: {
        projectV2: {
          id: 'PVT_test',
          fields: {
            nodes: [
              {
                id: 'PVTSSF_status',
                name: 'Status',
                options: [
                  { id: 'PVTSSF_todo', name: 'Todo' },
                  { id: 'PVTSSF_done', name: 'Done' },
                ],
              },
            ],
          },
          items: {
            nodes: [
              {
                id: 'PVTI_item1',
                content: { number: 123 },
              },
            ],
          },
        },
      },
    };

    mockOctokit.graphql.mockResolvedValueOnce(mockProjectData);

    const result = await workflowUpdateIssue({
      issueNumber: 123,
      status: 'in_progress', // This value doesn't exist in options
    });

    expect(result.issuesFound).toContain('Invalid status value: in_progress');
    expect(result.requestedData!.updatedFields).toEqual({});
  });

  it('should handle missing project configuration', async () => {
    vi.mocked(config.getProjectConfig).mockReturnValue({
      isComplete: false,
      config: {
        github: {},
      },
    });

    const result = await workflowUpdateIssue({
      issueNumber: 123,
      status: 'done',
    });

    expect(result.requestedData!.error).toBe('Project configuration not found. Run workflow_configure first.');
    expect(result.issuesFound).toContain('Error: Project configuration not found. Run workflow_configure first.');
  });

  it('should handle no fields to update', async () => {
    const mockProjectData = {
      user: {
        projectV2: {
          id: 'PVT_test',
          fields: { nodes: [] },
          items: {
            nodes: [
              {
                id: 'PVTI_item1',
                content: { number: 123 },
              },
            ],
          },
        },
      },
    };

    mockOctokit.graphql.mockResolvedValueOnce(mockProjectData);

    const result = await workflowUpdateIssue({
      issueNumber: 123,
    });

    expect(result.suggestedActions).toContain('No fields were updated. Specify status, type, or priority to update.');
    expect(result.requestedData!.updatedFields).toEqual({});
  });

  it('should handle field not found in project', async () => {
    const mockProjectData = {
      user: {
        projectV2: {
          id: 'PVT_test',
          fields: {
            nodes: [], // No fields defined
          },
          items: {
            nodes: [
              {
                id: 'PVTI_item1',
                content: { number: 123 },
              },
            ],
          },
        },
      },
    };

    mockOctokit.graphql.mockResolvedValueOnce(mockProjectData);

    const result = await workflowUpdateIssue({
      issueNumber: 123,
      status: 'done',
      type: 'bug',
      priority: 'high',
    });

    expect(result.issuesFound).toContain('Status field not found in project');
    expect(result.issuesFound).toContain('Type field not found in project');
    expect(result.issuesFound).toContain('Priority field not found in project');
    expect(result.requestedData!.updatedFields).toEqual({});
  });
});