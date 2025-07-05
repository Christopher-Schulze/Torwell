/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './src/**/*.{html,js,svelte,ts}',
    './src/routes/**/*.{svelte,ts}',
    './src/lib/**/*.{svelte,ts}'
  ],
  theme: {
    extend: {},
  },
  plugins: [
    require('tailwindcss-glassmorphism'),
  ],
}
