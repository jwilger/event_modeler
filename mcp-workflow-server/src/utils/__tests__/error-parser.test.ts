import { describe, it, expect } from 'vitest';
import {
  parseTypeScriptErrors,
  parseESLintErrors,
  parseCargoErrors,
  parseTestErrors,
  parsePreCommitOutput,
  formatParsedErrors,
} from '../error-parser.js';

describe('Error Parser', () => {
  describe('parseTypeScriptErrors', () => {
    it('should parse TypeScript compilation errors', () => {
      const output = `
src/tools/example.ts(42,5): error TS2339: Property 'foo' does not exist on type 'Bar'.
src/tools/example.ts(78,12): error TS2304: Cannot find name 'undefined_var'.
src/utils/helper.ts(10,3): warning TS6133: 'unused' is declared but its value is never read.
      `;

      const errors = parseTypeScriptErrors(output);

      expect(errors).toHaveLength(3);
      expect(errors[0]).toEqual({
        file: 'src/tools/example.ts',
        line: 42,
        column: 5,
        severity: 'error',
        rule: 'TS2339',
        message: "Property 'foo' does not exist on type 'Bar'.",
      });
      expect(errors[1]).toEqual({
        file: 'src/tools/example.ts',
        line: 78,
        column: 12,
        severity: 'error',
        rule: 'TS2304',
        message: "Cannot find name 'undefined_var'.",
      });
      expect(errors[2].severity).toBe('warning');
    });

    it('should return empty array for no errors', () => {
      const output = 'Build successful';
      const errors = parseTypeScriptErrors(output);
      expect(errors).toHaveLength(0);
    });
  });

  describe('parseESLintErrors', () => {
    it('should parse ESLint error output', () => {
      const output = `
/home/user/project/src/tools/example.ts
  10:5  error  'foo' is defined but never used  @typescript-eslint/no-unused-vars
  25:10  warning  Missing return type on function  @typescript-eslint/explicit-function-return-type

/home/user/project/src/utils/helper.ts
  5:1  error  'console' is not allowed  no-console
      `;

      const errors = parseESLintErrors(output);

      expect(errors).toHaveLength(3);
      expect(errors[0]).toEqual({
        file: '/home/user/project/src/tools/example.ts',
        line: 10,
        column: 5,
        severity: 'error',
        message: "'foo' is defined but never used",
        rule: '@typescript-eslint/no-unused-vars',
      });
      expect(errors[1].severity).toBe('warning');
      expect(errors[2].file).toBe('/home/user/project/src/utils/helper.ts');
    });

    it('should parse ESLint errors with Windows paths', () => {
      const output = `
C:\\Users\\project\\src\\tools\\example.ts
  10:5  error  'foo' is defined but never used  @typescript-eslint/no-unused-vars
  25:10  warning  Missing return type on function  @typescript-eslint/explicit-function-return-type

C:/Users/project/src/utils/helper.ts
  5:1  error  'console' is not allowed  no-console
      `;

      const errors = parseESLintErrors(output);

      expect(errors).toHaveLength(3);
      expect(errors[0]).toEqual({
        file: 'C:\\Users\\project\\src\\tools\\example.ts',
        line: 10,
        column: 5,
        severity: 'error',
        message: "'foo' is defined but never used",
        rule: '@typescript-eslint/no-unused-vars',
      });
      expect(errors[1].severity).toBe('warning');
      expect(errors[2].file).toBe('C:/Users/project/src/utils/helper.ts');
    });
  });

  describe('parseCargoErrors', () => {
    it('should parse Rust compilation errors', () => {
      const output = `
error[E0425]: cannot find value \`undefined_var\` in this scope
 --> src/main.rs:10:5
  |
10 |     undefined_var;
  |     ^^^^^^^^^^^^^ not found in this scope

warning: unused variable: \`x\`
 --> src/lib.rs:5:9
  |
5 |     let x = 42;
  |         ^ help: if this is intentional, prefix it with an underscore: \`_x\`
      `;

      const errors = parseCargoErrors(output);

      expect(errors).toHaveLength(2);
      expect(errors[0]).toEqual({
        file: 'src/main.rs',
        line: 10,
        column: 5,
        severity: 'error',
        rule: 'E0425',
        message: 'cannot find value `undefined_var` in this scope',
      });
      expect(errors[1]).toEqual({
        file: 'src/lib.rs',
        line: 5,
        column: 9,
        severity: 'warning',
        rule: undefined,
        message: "unused variable: `x`",
      });
    });
  });

  describe('parseTestErrors', () => {
    it('should parse test failure output', () => {
      const output = `
 FAIL  src/tools/__tests__/example.test.ts > Example test suite > should work
 FAIL  src/utils/__tests__/helper.test.js
 ✓ src/other.test.ts (5 tests)
      `;

      const errors = parseTestErrors(output);

      expect(errors).toHaveLength(2);
      expect(errors[0]).toEqual({
        file: 'src/tools/__tests__/example.test.ts',
        message: 'Test failed',
        severity: 'error',
      });
      expect(errors[1]).toEqual({
        file: 'src/utils/__tests__/helper.test.js',
        message: 'Test failed',
        severity: 'error',
      });
    });
  });

  describe('parsePreCommitOutput', () => {
    it('should parse mixed pre-commit output', () => {
      const output = `
MCP Server lint..........................................................Failed
- hook id: mcp-server-lint
- exit code: 1

/home/user/project/src/tools/example.ts
  42:5  error  Property 'foo' does not exist on type 'Bar'  @typescript-eslint/no-unused-vars

MCP Server build.........................................................Failed
- hook id: mcp-server-build
- exit code: 1

src/tools/example.ts(78,12): error TS2304: Cannot find name 'undefined_var'.
      `;

      const toolErrors = parsePreCommitOutput(output);

      expect(toolErrors).toHaveLength(2);
      
      // TypeScript errors (detected first)
      expect(toolErrors[0].tool).toBe('TypeScript');
      expect(toolErrors[0].errors).toHaveLength(1);
      expect(toolErrors[0].summary).toBe('TypeScript check failed with 1 error');
      expect(toolErrors[0].fixSuggestions).toContain('Fix TypeScript errors in the affected files');

      // ESLint errors (detected second)
      expect(toolErrors[1].tool).toBe('ESLint');
      expect(toolErrors[1].errors).toHaveLength(1);
      expect(toolErrors[1].summary).toBe('ESLint found 1 issue');
      expect(toolErrors[1].fixSuggestions).toContain('Fix ESLint issues in the affected files');
    });

    it('should handle cargo fmt errors', () => {
      const output = `
cargo fmt............................................Failed
- hook id: cargo-fmt
- exit code: 1

error: file not formatted according to style guide
 --> src/main.rs:10:5
      `;

      const toolErrors = parsePreCommitOutput(output);

      expect(toolErrors).toHaveLength(1);
      expect(toolErrors[0].tool).toBe('cargo fmt');
      expect(toolErrors[0].summary).toBe('Rust formatting check failed');
      expect(toolErrors[0].fixSuggestions).toContain('Run `cargo fmt` to fix formatting issues');
    });
  });

  describe('formatParsedErrors', () => {
    it('should format parsed errors for display', () => {
      const toolErrors = [
        {
          tool: 'TypeScript',
          errors: [
            {
              file: 'src/example.ts',
              line: 42,
              column: 5,
              message: "Property 'foo' does not exist",
              severity: 'error' as const,
              rule: 'TS2339',
            },
          ],
          summary: 'TypeScript check failed with 1 error',
          fixSuggestions: ['Fix TypeScript errors'],
        },
        {
          tool: 'ESLint',
          errors: [
            {
              file: 'src/helper.ts',
              line: 10,
              message: 'Unused variable',
              severity: 'warning' as const,
            },
          ],
          summary: 'ESLint found 1 issue',
          fixSuggestions: ['Fix ESLint issues'],
        },
      ];

      const formatted = formatParsedErrors(toolErrors);

      expect(formatted).toContain('TypeScript check failed with 1 error:');
      expect(formatted).toContain('  src/example.ts:42:5 [error] - Property \'foo\' does not exist (TS2339)');
      expect(formatted).toContain('ESLint found 1 issue:');
      expect(formatted).toContain('  src/helper.ts:10 [warning] - Unused variable');
    });

    it('should handle errors without file location', () => {
      const toolErrors = [
        {
          tool: 'Tests',
          errors: [
            {
              message: 'Test suite failed',
              severity: 'error' as const,
            },
          ],
          summary: '1 test failed',
          fixSuggestions: ['Fix failing tests'],
        },
      ];

      const formatted = formatParsedErrors(toolErrors);

      expect(formatted).toContain('1 test failed:');
      expect(formatted).toContain('   [error] - Test suite failed');
    });

    it('should preserve empty lines between tools but remove trailing empty lines', () => {
      const toolErrors = [
        {
          tool: 'TypeScript',
          errors: [
            {
              file: 'src/example.ts',
              line: 42,
              message: 'Error 1',
              severity: 'error' as const,
            },
          ],
          summary: 'TypeScript errors',
          fixSuggestions: [],
        },
        {
          tool: 'ESLint',
          errors: [
            {
              file: 'src/example.ts',
              line: 10,
              message: 'Error 2',
              severity: 'error' as const,
            },
          ],
          summary: 'ESLint errors',
          fixSuggestions: [],
        },
      ];

      const formatted = formatParsedErrors(toolErrors);
      
      // Should have empty line between tools
      const typeScriptIndex = formatted.findIndex(line => line.includes('TypeScript errors'));
      const eslintIndex = formatted.findIndex(line => line.includes('ESLint errors'));
      const emptyLineBetween = formatted.slice(typeScriptIndex + 1, eslintIndex).some(line => line === '');
      expect(emptyLineBetween).toBe(true);
      
      // Should not have trailing empty lines
      expect(formatted[formatted.length - 1]).not.toBe('');
    });
  });
});