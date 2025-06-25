import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { execSync } from "child_process";

vi.mock("child_process");

describe("auth", () => {
  const originalEnv = process.env;
  const mockExecSync = vi.mocked(execSync);

  beforeEach(() => {
    vi.clearAllMocks();
    // Reset environment
    process.env = { ...originalEnv };
    // Reset module state by re-importing
    vi.resetModules();
  });

  afterEach(() => {
    process.env = originalEnv;
  });

  describe("getGitHubToken", () => {
    it("should return token from GH_TOKEN environment variable", async () => {
      process.env.GH_TOKEN = "test-token-from-env";
      
      const { getGitHubToken: getToken } = await import("../auth.js");
      const token = getToken();
      
      expect(token).toBe("test-token-from-env");
      expect(mockExecSync).not.toHaveBeenCalled();
    });

    it("should return token from GITHUB_TOKEN environment variable", async () => {
      delete process.env.GH_TOKEN;
      process.env.GITHUB_TOKEN = "test-token-from-github-env";
      
      const { getGitHubToken: getToken } = await import("../auth.js");
      const token = getToken();
      
      expect(token).toBe("test-token-from-github-env");
      expect(mockExecSync).not.toHaveBeenCalled();
    });

    it("should fall back to gh CLI when env vars not set", async () => {
      delete process.env.GH_TOKEN;
      delete process.env.GITHUB_TOKEN;
      mockExecSync.mockReturnValue("test-token-from-cli\n");
      
      const { getGitHubToken: getToken } = await import("../auth.js");
      const token = getToken();
      
      expect(token).toBe("test-token-from-cli");
      expect(mockExecSync).toHaveBeenCalledWith("gh auth token", { encoding: "utf8" });
    });

    it("should cache token after first retrieval", async () => {
      delete process.env.GH_TOKEN;
      delete process.env.GITHUB_TOKEN;
      mockExecSync.mockReturnValue("cached-token\n");
      
      const { getGitHubToken: getToken } = await import("../auth.js");
      
      // First call
      const token1 = getToken();
      expect(token1).toBe("cached-token");
      expect(mockExecSync).toHaveBeenCalledTimes(1);
      
      // Second call should use cache
      const token2 = getToken();
      expect(token2).toBe("cached-token");
      expect(mockExecSync).toHaveBeenCalledTimes(1); // Still only called once
    });

    it("should throw error when no token available", async () => {
      delete process.env.GH_TOKEN;
      delete process.env.GITHUB_TOKEN;
      mockExecSync.mockImplementation(() => {
        throw new Error("gh not found");
      });
      
      const { getGitHubToken: getToken } = await import("../auth.js");
      
      expect(() => getToken()).toThrow(
        "GitHub token not found. Please set GH_TOKEN environment variable or run 'gh auth login'"
      );
    });

    it("should throw error when gh auth token returns empty", async () => {
      delete process.env.GH_TOKEN;
      delete process.env.GITHUB_TOKEN;
      mockExecSync.mockReturnValue("");
      
      const { getGitHubToken: getToken } = await import("../auth.js");
      
      expect(() => getToken()).toThrow(
        "GitHub token not found. Please set GH_TOKEN environment variable or run 'gh auth login'"
      );
    });
  });

  describe("initializeAuth", () => {
    it("should initialize successfully with valid token", async () => {
      process.env.GH_TOKEN = "test-token";
      const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
      
      const { initializeAuth: init } = await import("../auth.js");
      
      expect(() => init()).not.toThrow();
      expect(consoleSpy).toHaveBeenCalledWith("GitHub authentication initialized successfully");
      
      consoleSpy.mockRestore();
    });

    it("should log warning when authentication fails", async () => {
      delete process.env.GH_TOKEN;
      delete process.env.GITHUB_TOKEN;
      mockExecSync.mockImplementation(() => {
        throw new Error("gh not found");
      });
      
      const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
      
      const { initializeAuth: init } = await import("../auth.js");
      
      expect(() => init()).not.toThrow();
      expect(consoleSpy).toHaveBeenCalledWith(
        "WARNING: GitHub authentication not available:",
        expect.any(Error)
      );
      expect(consoleSpy).toHaveBeenCalledWith(
        "Some tools may not function without authentication"
      );
      
      consoleSpy.mockRestore();
    });
  });
});