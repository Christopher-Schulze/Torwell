import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import { errorStore } from '../lib/components/AppErrorBoundary.svelte';
import Wrapper from './ErrorBoundaryWrapper.svelte';

describe('AppErrorBoundary', () => {
  it('captures errors from child components', () => {
    render(Wrapper);
    let err: Error | null = null;
    const unsub = errorStore.subscribe((e) => (err = e));
    expect(err).toBeInstanceOf(Error);
    unsub();
  });
});
