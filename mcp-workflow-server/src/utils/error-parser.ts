/**
 * Error parser utilities for extracting structured information from tool outputs
 */

export interface ParsedError {
  file?: string;
  line?: number;
  column?: number;
  message: string;
  severity?: 'error' | 'warning';
  rule?: string;
}

export interface ToolError {
  tool: string;
  errors: ParsedError[];
  summary?: string;
  fixSuggestions: string[];
}

/**
 * Parse TypeScript/TSC errors
 * Format: src/file.ts(10,5): error TS2304: Cannot find name 'foo'.
 */
export function parseTypeScriptErrors(output: string): ParsedError[] {
  const errors: ParsedError[] = [];
  const tsErrorRegex = /^(.+?)\((\d+),(\d+)\):\s*(error|warning)\s+TS(\d+):\s*(.+)$/gm;

  let match;
  while ((match = tsErrorRegex.exec(output)) !== null) {
    errors.push({
      file: match[1],
      line: parseInt(match[2]),
      column: parseInt(match[3]),
      severity: match[4] as 'error' | 'warning',
      rule: `TS${match[5]}`,
      message: match[6].trim(),
    });
  }

  return errors;
}

/**
 * Parse ESLint errors
 * Format: /path/to/file.ts
 *   10:5  error  'foo' is defined but never used  @typescript-eslint/no-unused-vars
 */
export function parseESLintErrors(output: string): ParsedError[] {
  const errors: ParsedError[] = [];
  const lines = output.split('\n');
  let currentFile: string | undefined;

  for (const line of lines) {
    // File path line - support both Unix and Windows paths
    // Unix: /path/to/file.ts
    // Windows: C:\path\to\file.ts or C:/path/to/file.ts
    const isWindowsPath = /^[A-Za-z]:[\\/]/.test(line);
    const isUnixPath = line.startsWith('/');
    const hasNoLocationInfo =
      !line.includes(':') || (isWindowsPath && line.split(':').length === 2);

    if ((isUnixPath || isWindowsPath) && hasNoLocationInfo) {
      currentFile = line.trim();
      continue;
    }

    // Error line format: "  10:5  error  Message  rule-name"
    const errorMatch = line.match(/^\s*(\d+):(\d+)\s+(error|warning)\s+(.+?)\s\s+(.+)$/);
    if (errorMatch && currentFile) {
      errors.push({
        file: currentFile,
        line: parseInt(errorMatch[1]),
        column: parseInt(errorMatch[2]),
        severity: errorMatch[3] as 'error' | 'warning',
        message: errorMatch[4].trim(),
        rule: errorMatch[5].trim(),
      });
    }
  }

  return errors;
}

/**
 * Parse Cargo/Rust errors
 * Format: error[E0425]: cannot find value `foo` in this scope
 *  --> src/main.rs:10:5
 */
export function parseCargoErrors(output: string): ParsedError[] {
  const errors: ParsedError[] = [];
  const lines = output.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const errorLine = lines[i];
    const errorMatch = errorLine.match(/^(error|warning)(?:\[([A-Z]\d+)\])?: (.+)$/);

    if (errorMatch) {
      // Look for the file location in the next few lines
      for (let j = i + 1; j < Math.min(i + 5, lines.length); j++) {
        const locationMatch = lines[j].match(/^\s*-->\s*(.+?):(\d+):(\d+)$/);
        if (locationMatch) {
          errors.push({
            file: locationMatch[1],
            line: parseInt(locationMatch[2]),
            column: parseInt(locationMatch[3]),
            severity: errorMatch[1] as 'error' | 'warning',
            rule: errorMatch[2],
            message: errorMatch[3].trim(),
          });
          break;
        }
      }
    }
  }

  return errors;
}

/**
 * Parse Jest/Vitest test failures
 * Format: FAIL  src/test.ts > Test suite > test case
 */
export function parseTestErrors(output: string): ParsedError[] {
  const errors: ParsedError[] = [];
  const failRegex = /FAIL\s+(.+\.(?:test|spec)\.(?:ts|js|tsx|jsx))/g;

  let match;
  while ((match = failRegex.exec(output)) !== null) {
    errors.push({
      file: match[1],
      message: 'Test failed',
      severity: 'error',
    });
  }

  return errors;
}

/**
 * Parse pre-commit hook output and extract structured errors
 */
export function parsePreCommitOutput(output: string): ToolError[] {
  const toolErrors: ToolError[] = [];

  // Try to identify TypeScript errors
  const tsErrors = parseTypeScriptErrors(output);
  if (tsErrors.length > 0) {
    toolErrors.push({
      tool: 'TypeScript',
      errors: tsErrors,
      summary: `TypeScript check failed with ${tsErrors.length} error${tsErrors.length > 1 ? 's' : ''}`,
      fixSuggestions: [
        'Fix TypeScript errors in the affected files',
        'Run `npm run build` to see full error details',
      ],
    });
  }

  // Try to identify ESLint errors
  const eslintErrors = parseESLintErrors(output);
  if (eslintErrors.length > 0) {
    toolErrors.push({
      tool: 'ESLint',
      errors: eslintErrors,
      summary: `ESLint found ${eslintErrors.length} issue${eslintErrors.length > 1 ? 's' : ''}`,
      fixSuggestions: [
        'Fix ESLint issues in the affected files',
        'Run `npm run lint` to see all issues',
        'Run `npm run lint -- --fix` to auto-fix some issues',
      ],
    });
  }

  // Try to identify Cargo/Rust errors
  const cargoErrors = parseCargoErrors(output);
  if (cargoErrors.length > 0) {
    const hasFormatting = output.includes('cargo fmt');
    const hasClippy = output.includes('cargo clippy');

    toolErrors.push({
      tool: hasFormatting ? 'cargo fmt' : hasClippy ? 'cargo clippy' : 'Rust',
      errors: cargoErrors,
      summary: hasFormatting
        ? 'Rust formatting check failed'
        : hasClippy
          ? `Clippy found ${cargoErrors.length} issue${cargoErrors.length > 1 ? 's' : ''}`
          : `Rust compilation failed with ${cargoErrors.length} error${cargoErrors.length > 1 ? 's' : ''}`,
      fixSuggestions: hasFormatting
        ? ['Run `cargo fmt` to fix formatting issues']
        : hasClippy
          ? ['Fix Clippy warnings in the affected files']
          : ['Fix compilation errors in the affected files'],
    });
  }

  // Try to identify test failures
  const testErrors = parseTestErrors(output);
  if (testErrors.length > 0) {
    toolErrors.push({
      tool: 'Tests',
      errors: testErrors,
      summary: `${testErrors.length} test file${testErrors.length > 1 ? 's' : ''} failed`,
      fixSuggestions: ['Fix failing tests', 'Run `npm test` to see detailed test output'],
    });
  }

  return toolErrors;
}

/**
 * Format parsed errors for display in automaticActions
 */
export function formatParsedErrors(toolErrors: ToolError[]): string[] {
  const formatted: string[] = [];

  for (const toolError of toolErrors) {
    if (toolError.summary) {
      formatted.push(toolError.summary + ':');
    }

    for (const error of toolError.errors) {
      const location = error.file
        ? error.line
          ? `  ${error.file}:${error.line}${error.column ? `:${error.column}` : ''}`
          : `  ${error.file}`
        : '  ';

      const severity = error.severity ? ` [${error.severity}]` : '';
      const rule = error.rule ? ` (${error.rule})` : '';

      formatted.push(`${location}${severity} - ${error.message}${rule}`);
    }

    if (toolError.errors.length > 0) {
      formatted.push(''); // Empty line between tools
    }
  }

  // Remove trailing empty lines but preserve empty lines between tools
  return formatted.filter((line, index, arr) => {
    // Keep non-empty lines
    if (line !== '') return true;
    // Keep empty lines that are not at the end
    if (index < arr.length - 1) return true;
    // Remove trailing empty lines
    return false;
  });
}
