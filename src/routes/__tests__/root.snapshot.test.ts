import { render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import Page from '../+page.svelte';

describe('Root dashboard snapshot', () => {
  it('matches the expected markup', () => {
    const { container } = render(Page);
    expect(container).toMatchSnapshot();
  });
});
