/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './src/**/*.{html,js,svelte,ts}',
    './src/routes/**/*.{svelte,ts}',
    './src/lib/**/*.{svelte,ts}'
  ],
  theme: {
    extend: {
      colors: {
        void: '#050507',
        'void-deep': '#000000',
        'neon-green': '#00ff41',
        'neon-purple': '#bc13fe',
      },
      fontFamily: {
        mono: ['Fira Code', 'monospace'],
        sans: ['Inter', 'sans-serif'],
      },
      boxShadow: {
        'glow-green': '0 0 20px -5px rgba(0, 255, 65, 0.4)',
        'glow-purple': '0 0 20px -5px rgba(188, 19, 254, 0.4)',
      }
    },
  },
  plugins: [
    require('tailwindcss-glassmorphism'),
  ],
}
