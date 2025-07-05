import { render } from '@testing-library/svelte';
import { vi, describe, it, expect } from 'vitest';
import { tick } from 'svelte';

var warningCallback: (event: any) => void = () => {};
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((_event: string, cb: any) => {
    if (_event === 'security-warning') warningCallback = cb;
  })
}));

import SecurityBanner from '../lib/components/SecurityBanner.svelte';

describe('SecurityBanner', () => {
  it('renders message when event emitted', async () => {
    const { queryByRole, getByRole } = render(SecurityBanner);
    expect(queryByRole('alert')).toBeNull();

    warningCallback({ payload: 'warning msg' });
    await tick();

    expect(getByRole('alert')).toHaveTextContent('warning msg');
  });
});
