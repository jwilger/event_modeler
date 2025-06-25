import { execSync } from "child_process";

let cachedToken: string | null = null;

/**
 * Get GitHub token, using cached value if available.
 * First tries environment variables (GH_TOKEN, GITHUB_TOKEN),
 * then falls back to extracting from gh CLI.
 * The token is cached after first retrieval to avoid repeated execSync calls.
 */
export function getGitHubToken(): string {
  // Return cached token if available
  if (cachedToken) {
    return cachedToken;
  }

  // Try environment variables first
  const envToken = process.env.GH_TOKEN || process.env.GITHUB_TOKEN;
  if (envToken) {
    cachedToken = envToken;
    return cachedToken;
  }

  // Fall back to gh CLI
  try {
    const token = execSync("gh auth token", { encoding: "utf8" }).trim();
    if (!token) {
      throw new Error("gh auth token returned empty");
    }
    cachedToken = token;
    console.error("GitHub token extracted from gh CLI and cached");
    return cachedToken;
  } catch {
    throw new Error(
      "GitHub token not found. Please set GH_TOKEN environment variable or run 'gh auth login'"
    );
  }
}

/**
 * Initialize GitHub token at startup to fail fast if not available
 */
export function initializeAuth(): void {
  try {
    getGitHubToken();
    console.error("GitHub authentication initialized successfully");
  } catch (error) {
    console.error("WARNING: GitHub authentication not available:", error);
    console.error("Some tools may not function without authentication");
  }
}