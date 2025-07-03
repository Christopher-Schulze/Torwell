import { render } from '@testing-library/svelte';
import IdlePanel from '../lib/components/IdlePanel.svelte';

describe('IdlePanel', () => {
  it('updates progressbar value', () => {
    const { getByRole } = render(IdlePanel, {
      props: {
        connectionProgress: 30,
        currentStatus: 'CONNECTING'
      }
    });

    const bar = getByRole('progressbar');
    expect(bar).toHaveAttribute('aria-valuenow', '30');
  });

  it('shows bootstrap message and retry info', () => {
    const { getByText } = render(IdlePanel, {
      props: {
        connectionProgress: 50,
        currentStatus: 'RETRYING',
        retryCount: 2,
        retryDelay: 5,
        bootstrapMessage: 'Starting'
      }
    });

    expect(getByText('Starting')).toBeInTheDocument();
    expect(getByText(/retry 2 in 5s/i)).toBeInTheDocument();
  });
});
