import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import LogsModal from '../lib/components/LogsModal.svelte';

describe('Modal focus trapping', () => {
  it('keeps focus inside LogsModal', async () => {
    const { getByLabelText, container } = render(LogsModal, { props: { show: true } });
    const modal = container.querySelector('[role="dialog"]') as HTMLElement;
    const first = getByLabelText('Close logs');
    const last = getByLabelText('Clear logs');

    last.focus();
    await fireEvent.keyDown(modal, { key: 'Tab' });
    expect(document.activeElement).toBe(first);

    first.focus();
    await fireEvent.keyDown(modal, { key: 'Tab', shiftKey: true });
    expect(document.activeElement).toBe(last);
  });

  it('focuses the close button when opened', async () => {
    const { getByLabelText } = render(LogsModal, { props: { show: true } });
    await Promise.resolve();
    const close = getByLabelText('Close logs');
    expect(document.activeElement).toBe(close);
  });
});
