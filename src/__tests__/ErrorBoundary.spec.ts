import { render } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import 'fake-indexeddb/auto';

vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));
vi.mock('@tauri-apps/api/tauri', () => ({ invoke: vi.fn() }));
import { get } from 'svelte/store';
import ErrorBoundaryTest from './ErrorBoundaryTest.svelte';
import { uiStore } from '../lib/stores/uiStore';

describe('Global error handling', () => {
  it('captures errors from child components', () => {
    expect(() => render(ErrorBoundaryTest)).toThrowError('Boom');
    expect(get(uiStore).error).toBe('Boom');
  });
});
