import { WorkflowResponse, PRStatus } from '../types.js';
import { getRepoInfo } from '../utils/github.js';
import { Octokit } from '@octokit/rest';
import { getGitHubToken } from '../utils/auth.js';

interface WorkflowManagePRParams {
  action: 'analyze' | 'rebase' | 'update_chains';
  prNumber?: number;
  targetBranch?: string;
  force?: boolean;
}

interface PRAction {
  prNumber: number;
  action: 'rebased' | 'updated_base' | 'skipped' | 'conflict_detected';
  success: boolean;
  details: string;
  conflictingFiles?: string[];
}

interface ManualIntervention {
  prNumber: number;
  issue: 'merge_conflict' | 'complex_rebase' | 'dependency_issue' | 'force_push_risk';
  conflictingFiles?: string[];
  suggestion: string;
}

interface PRChainInfo {
  prNumber: number;
  title: string;
  baseBranch: string;
  headBranch: string;
  dependsOn?: number[];
  dependents?: number[];
  status: 'merged' | 'ready' | 'blocked_conflict' | 'pending' | 'needs_rebase';
  needsRebase: boolean;
  isMergeable: boolean;
}

export async function workflowManagePR(params: WorkflowManagePRParams): Promise<WorkflowResponse> {
  const automaticActions: string[] = [];
  const issuesFound: string[] = [];
  const suggestedActions: string[] = [];
  const allPRStatus: PRStatus[] = [];

  try {
    const { action, prNumber, targetBranch = 'main' } = params;

    const token = getGitHubToken();
    const octokit = new Octokit({ auth: token });
    const { owner, repo } = getRepoInfo();

    automaticActions.push(`Starting PR management action: ${action}`);

    // Get all open PRs
    const { data: pulls } = await octokit.pulls.list({
      owner,
      repo,
      state: 'open',
      per_page: 100,
    });

    automaticActions.push(`Found ${pulls.length} open PRs`);

    const actionsPerformed: PRAction[] = [];
    const manualInterventionNeeded: ManualIntervention[] = [];
    const prChains: PRChainInfo[] = [];

    // Analyze PR chains and dependencies
    for (const pr of pulls) {
      const prInfo = await analyzePR(octokit, owner, repo, pr);
      prChains.push(prInfo);
    }

    // Build dependency graph
    buildDependencyGraph(prChains);

    switch (action) {
      case 'analyze':
        automaticActions.push('Performed analysis of PR chains and dependencies');
        break;

      case 'rebase': {
        if (prNumber) {
          const result = await rebaseSinglePR(octokit, owner, repo, prNumber);
          actionsPerformed.push(result.action);
          if (result.intervention) {
            manualInterventionNeeded.push(result.intervention);
          }
        } else {
          issuesFound.push('PR number required for rebase action');
        }
        break;
      }

      case 'update_chains': {
        const chainResults = await updatePRChains(octokit, owner, repo, prChains, targetBranch);
        actionsPerformed.push(...chainResults.actions);
        manualInterventionNeeded.push(...chainResults.interventions);
        break;
      }

      default:
        throw new Error(`Unknown action: ${action}`);
    }

    // Generate suggestions based on analysis
    generateSuggestions(prChains, manualInterventionNeeded, suggestedActions);

    const summary = {
      totalPRsAnalyzed: prChains.length,
      automaticRebases: actionsPerformed.filter((a) => a.action === 'rebased' && a.success).length,
      conflictsDetected: actionsPerformed.filter((a) => a.action === 'conflict_detected').length,
      chainsUpdated: actionsPerformed.filter((a) => a.action === 'updated_base' && a.success)
        .length,
    };

    return {
      requestedData: {
        actionsPerformed,
        manualInterventionNeeded,
        prChains,
        summary,
      },
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus,
    };
  } catch (error) {
    issuesFound.push(`Error: ${error instanceof Error ? error.message : String(error)}`);
    return {
      requestedData: {
        error: error instanceof Error ? error.message : String(error),
      },
      automaticActions,
      issuesFound,
      suggestedActions,
      allPRStatus,
    };
  }
}

interface PullRequestData {
  number: number;
  title: string;
  body?: string | null;
  head: { ref: string };
  base: { ref: string };
}

async function analyzePR(
  octokit: Octokit,
  owner: string,
  repo: string,
  pr: PullRequestData
): Promise<PRChainInfo> {
  // Get detailed PR information
  const { data: prDetail } = await octokit.pulls.get({
    owner,
    repo,
    pull_number: pr.number,
  });

  const needsRebase = prDetail.mergeable_state === 'behind' || prDetail.mergeable_state === 'dirty';
  const isMergeable = prDetail.mergeable === true;

  // Analyze dependencies based on PR body
  const dependsOn = extractDependencies(pr.body || '');

  return {
    prNumber: pr.number,
    title: pr.title,
    baseBranch: pr.base.ref,
    headBranch: pr.head.ref,
    dependsOn,
    dependents: [], // Will be filled in buildDependencyGraph
    status: determineStatus(needsRebase, isMergeable),
    needsRebase,
    isMergeable,
  };
}

function extractDependencies(prBody: string): number[] {
  const dependencies: number[] = [];

  // Look for "Depends on #123" patterns in PR body
  const dependsOnMatches = prBody.match(/depends on #(\d+)/gi);
  if (dependsOnMatches) {
    dependencies.push(...dependsOnMatches.map((match) => parseInt(match.match(/#(\d+)/)![1])));
  }

  // Look for "Closes #123" patterns (these are dependencies too)
  const closesMatches = prBody.match(/closes #(\d+)/gi);
  if (closesMatches) {
    dependencies.push(...closesMatches.map((match) => parseInt(match.match(/#(\d+)/)![1])));
  }

  return dependencies;
}

function buildDependencyGraph(prChains: PRChainInfo[]): void {
  // Build reverse dependency relationships
  for (const pr of prChains) {
    if (pr.dependsOn) {
      for (const depPrNumber of pr.dependsOn) {
        const depPr = prChains.find((p) => p.prNumber === depPrNumber);
        if (depPr) {
          if (!depPr.dependents) depPr.dependents = [];
          depPr.dependents.push(pr.prNumber);
        }
      }
    }
  }
}

function determineStatus(needsRebase: boolean, isMergeable: boolean): PRChainInfo['status'] {
  if (needsRebase) return 'needs_rebase';
  if (!isMergeable) return 'blocked_conflict';
  return 'ready';
}

async function rebaseSinglePR(
  _octokit: Octokit,
  _owner: string,
  _repo: string,
  prNumber: number
): Promise<{ action: PRAction; intervention?: ManualIntervention }> {
  // This is a placeholder for the actual rebase implementation
  // In a real implementation, this would:
  // 1. Check out the PR branch
  // 2. Fetch latest changes
  // 3. Attempt rebase
  // 4. Handle conflicts
  // 5. Force push if successful

  return {
    action: {
      prNumber,
      action: 'skipped',
      success: false,
      details: 'Rebase implementation pending',
    },
    intervention: {
      prNumber,
      issue: 'complex_rebase',
      suggestion: 'Manual rebase required - implementation pending',
    },
  };
}

async function updatePRChains(
  _octokit: Octokit,
  _owner: string,
  _repo: string,
  _prChains: PRChainInfo[],
  _targetBranch: string
): Promise<{ actions: PRAction[]; interventions: ManualIntervention[] }> {
  const actions: PRAction[] = [];
  const interventions: ManualIntervention[] = [];

  // This is a placeholder for chain update logic
  // Would implement topological sort and update PRs in dependency order

  return { actions, interventions };
}

function generateSuggestions(
  prChains: PRChainInfo[],
  interventions: ManualIntervention[],
  suggestions: string[]
): void {
  const needsRebase = prChains.filter((pr) => pr.needsRebase).length;
  const conflicts = prChains.filter((pr) => pr.status === 'blocked_conflict').length;

  if (needsRebase > 0) {
    suggestions.push(
      `${needsRebase} PR${needsRebase === 1 ? '' : 's'} need${needsRebase === 1 ? 's' : ''} rebasing`
    );
  }

  if (conflicts > 0) {
    suggestions.push(
      `${conflicts} PR${conflicts === 1 ? '' : 's'} ${conflicts === 1 ? 'has' : 'have'} merge conflicts that need manual resolution`
    );
  }

  if (interventions.length > 0) {
    suggestions.push(
      `${interventions.length} PR${interventions.length === 1 ? '' : 's'} require${interventions.length === 1 ? 's' : ''} manual intervention`
    );
  }

  // Suggest next actions based on PR chain analysis
  const readyPRs = prChains.filter(
    (pr) => pr.status === 'ready' && (!pr.dependsOn || pr.dependsOn.length === 0)
  );
  if (readyPRs.length > 0) {
    suggestions.push(
      `${readyPRs.length} PR${readyPRs.length === 1 ? '' : 's'} ${readyPRs.length === 1 ? 'is' : 'are'} ready to merge (no dependencies)`
    );
  }
}
