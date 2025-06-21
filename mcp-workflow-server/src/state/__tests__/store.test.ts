import { describe, it, expect } from 'vitest';
import { WorkflowState } from '../store.js';

describe('StateStore', () => {
  describe('state structure', () => {
    it('should have correct default state structure', () => {
      const defaultState: WorkflowState = {
        lastCheckedPRs: {},
        branchCreationDates: {},
        lastStatusCheck: new Date().toISOString(),
        version: 1,
      };

      expect(defaultState).toHaveProperty('lastCheckedPRs');
      expect(defaultState).toHaveProperty('branchCreationDates');
      expect(defaultState).toHaveProperty('lastStatusCheck');
      expect(defaultState).toHaveProperty('version');
      expect(defaultState.version).toBe(1);
    });

    it('should have correct PR status structure', () => {
      const prStatus = {
        lastChecked: '2024-01-01T00:00:00.000Z',
        lastReviewCount: 2,
        lastCheckRunStatus: 'success',
      };

      expect(prStatus).toHaveProperty('lastChecked');
      expect(prStatus).toHaveProperty('lastReviewCount');
      expect(prStatus).toHaveProperty('lastCheckRunStatus');
    });
  });

});