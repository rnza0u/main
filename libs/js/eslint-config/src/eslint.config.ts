import eslint from '@eslint/js'
import tseslint from 'typescript-eslint'
import stylistic from '@stylistic/eslint-plugin'
import _globals from 'globals'

export default tseslint.config(
  {
    ...eslint.configs.recommended,
    languageOptions: {
      globals: {
        ..._globals.browser,
        ..._globals.node
      },
      parserOptions: {
        ecmaFeatures: {
          jsx: true
        }
      }
    }
  },
  ...tseslint.configs.recommended,
  ...tseslint.configs.strict,
  {
    rules: {
      '@typescript-eslint/no-var-requires': 'off'
    }
  },
  {

    plugins: {
      '@stylistic': stylistic as any
    },
    rules: {
      '@stylistic/indent': ['error', 4],
      '@stylistic/no-extra-semi': ['error'],
      '@stylistic/semi': ['error', 'never'],
      '@stylistic/quotes': ['error', 'single']
    }
  },
  {
    ignores: ['**/dist/', '**/lib/', '**/node_modules', 'build/', '.docusaurus/'],
  }
)