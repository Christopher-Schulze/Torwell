import { writable } from 'svelte/store';

export const errorStore = writable<Error | null>(null);
