import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';

// Mock tauri event listener so store initialization doesn't fail
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));

// Provide a minimal mock for the uiStore used by TorChain
const setExitCountry = vi.fn();
vi.mock('$lib/stores/uiStore', () => {
  const { writable } = require('svelte/store');
  const store = writable({ settings: { exitCountry: null } });
  return {
    uiStore: {
      subscribe: store.subscribe,
      actions: {
        setExitCountry: (country: string | null) => {
          setExitCountry(country);
          store.set({ settings: { exitCountry: country } });
        },
      },
    },
  };
});

import TorChain from '../lib/components/TorChain.svelte';

const nodeData = [
  { nickname: 'entry', ip_address: '1.1.1.1', country: 'DE' },
  { nickname: 'middle', ip_address: '2.2.2.2', country: 'FR' },
  { nickname: 'exit', ip_address: '3.3.3.3', country: 'US' },
];

describe('TorChain', () => {
  it('renders node card data only when connected', () => {
    const { queryByText: queryDisconnected } = render(TorChain, {
      props: { isConnected: false, nodeData },
    });
    expect(queryDisconnected('1.1.1.1')).not.toBeInTheDocument();

    const { getByText } = render(TorChain, {
      props: { isConnected: true, nodeData },
    });
    expect(getByText('1.1.1.1')).toBeInTheDocument();
    expect(getByText('entry')).toBeInTheDocument();
  });

  it('calls setExitCountry when exit dropdown changes', async () => {
    const { getByLabelText } = render(TorChain, {
      props: { isConnected: true, nodeData },
    });

    const select = getByLabelText('Preferred exit country') as HTMLSelectElement;
    await fireEvent.change(select, { target: { value: 'US' } });

    expect(setExitCountry).toHaveBeenCalledWith('US');
  });

  it('displays isolated circuits list', () => {
    const isolatedCircuits = [
      {
        domain: 'example.com',
        nodes: [
          { nickname: 'n1', ip_address: '1.1.1.1', country: 'DE' },
          { nickname: 'n2', ip_address: '2.2.2.2', country: 'FR' },
        ],
      },
    ];

    const { getByText } = render(TorChain, {
      props: { isConnected: true, nodeData, isolatedCircuits },
    });

    expect(getByText('Isolated Circuits')).toBeInTheDocument();
    expect(getByText(/example.com/)).toBeInTheDocument();
  });
});
