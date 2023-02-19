// Performance measurement APIs
//
// This module provides an implementation of a subset of the W3C Web Performance APIs
// https://nodejs.org/api/perf_hooks.html#performance-measurement-apis

'use strict';

const perfHooks = process.binding('perf_hooks');

export const performance = perfHooks.performance;

export default perfHooks;
