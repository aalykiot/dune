import js from '@eslint/js';
import { defineConfig, globalIgnores } from 'eslint/config';
import prettierConfig from 'eslint-plugin-prettier/recommended';
import globals from 'globals';

export default defineConfig([
  globalIgnores([
    'dist/',
    'temp/',
    'node_modules/',
    'src/js/assert.js',
    'src/js/events.js',
    'src/js/text-encoding.js',
    'src/js/structured-clone.js',
  ]),
  js.configs.recommended,
  prettierConfig,
  {
    files: ['src/js/**/*.js', 'examples/**/*.js', 'tests/**/*.js'],
    languageOptions: {
      globals: {
        ...globals.es2026,
        ...globals['shared-node-browser'],
        global: true,
        globalThis: true,
        process: true,
        prompt: true,
        setImmediate: true,
        clearImmediate: true,
      },
    },
  },
]);
