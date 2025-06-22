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

export interface WorkflowResponse {
  requestedData: Record<string, unknown> | null;
  automaticActions: string[];
  issuesFound: string[];
  suggestedActions: string[];
  allPRStatus: PRStatus[];
}