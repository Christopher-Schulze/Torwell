import { browser } from '$app/environment';
import { readable } from 'svelte/store';

export const reducedMotion = readable(false, (set) => {
  if (!browser) {
    return () => undefined;
  }
  const query = window.matchMedia('(prefers-reduced-motion: reduce)');
  set(query.matches);
  const handler = (event: MediaQueryListEvent) => set(event.matches);
  query.addEventListener('change', handler);
  return () => query.removeEventListener('change', handler);
});
