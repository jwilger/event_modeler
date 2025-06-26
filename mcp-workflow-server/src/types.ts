export interface CheckRunDetail {
  name: string;
  status: 'queued' | 'in_progress' | 'completed';
  conclusion: 'success' | 'failure' | 'neutral' | 'cancelled' | 'skipped' | 'timed_out' | 'action_required' | null;
  url?: string;
  output?: {
    title?: string;
    summary?: string;
  };
}

export interface PRStatus {
  number: number;
  title: string;
  branch: string;
  baseRef: string;
  state: 'open' | 'closed' | 'merged';
  isDraft: boolean;
  url: string;
  checks: {
    total: number;
    passed: number;
    failed: number;
    pending: number;
    details: CheckRunDetail[];
  };
  hasUnresolvedReviews: boolean;
  needsRebase: boolean;
  isMergeable: boolean;
}

export interface GitStatus {
  currentBranch: string;
  isClean: boolean;
  uncommittedFiles: string[];
  untrackedFiles: string[];
  aheadBehind: {
    ahead: number;
    behind: number;
  };
  lastCommit: {
    hash: string;
    message: string;
    date: string;
  };
}

export interface Issue {
  severity: 'error' | 'warning' | 'info';
  message: string;
  context?: Record<string, unknown>;
}

export interface SuggestedAction {
  priority: 'urgent' | 'high' | 'medium' | 'low';
  description: string;
  command?: string;
}

// Standardized next step action structure for workflow guidance
export interface NextStepAction {
  action: string;
  description: string;
  tool?: string;
  parameters?: Record<string, unknown>;
  priority: 'urgent' | 'high' | 'medium' | 'low';
  category: 'immediate' | 'next_logical' | 'optional';
  condition?: string;
  // Additional context fields (optional, tool-specific)
  [key: string]: unknown;
}

export interface WorkflowResponse {
  requestedData: Record<string, unknown> | null;
  automaticActions: string[];
  issuesFound: string[];
  suggestedActions: string[];
  nextSteps?: NextStepAction[];
  allPRStatus: PRStatus[];
}