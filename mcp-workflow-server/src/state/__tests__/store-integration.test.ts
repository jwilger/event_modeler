import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { StateStore } from '../store.js';
import { mkdirSync, rmSync, existsSync } from 'fs';
import { tmpdir } from 'os';
import { join } from 'path';

describe('StateStore Integration Tests', () => {
  const testDir = join(tmpdir(), 'event-modeler-test-' + Date.now());
  
  beforeEach(() => {
    // Create test directory
    mkdirSync(testDir, { recursive: true });
    // Override homedir for tests
    process.env.HOME = testDir;
  });

  afterEach(() => {
    // Clean up test directory
    if (existsSync(testDir)) {
      rmSync(testDir, { recursive: true, force: true });
    }
  });

  it('should persist and restore state', () => {
    // Create a store and update some state
    const store1 = new StateStore();
    store1.updatePRStatus(123, 2, 'success');
    store1.recordBranchCreation('feature/test');
    
    // Create a new store instance - should load persisted state
    const store2 = new StateStore();
    
    const prStatus = store2.getPRStatus(123);
    expect(prStatus).toBeDefined();
    expect(prStatus?.lastReviewCount).toBe(2);
    expect(prStatus?.lastCheckRunStatus).toBe('success');
    
    const branchDate = store2.getBranchCreationDate('feature/test');
    expect(branchDate).toBeDefined();
  });
  
  it('should handle PR status lifecycle', () => {
    const store = new StateStore();
    
    // Add PR status
    store.updatePRStatus(456, 0, 'pending');
    expect(store.getPRStatus(456)).toBeDefined();
    
    // Update PR status
    store.updatePRStatus(456, 1, 'failed');
    const status = store.getPRStatus(456);
    expect(status?.lastReviewCount).toBe(1);
    expect(status?.lastCheckRunStatus).toBe('failed');
    
    // Clear PR status
    store.clearPRStatus(456);
    expect(store.getPRStatus(456)).toBeUndefined();
  });
});