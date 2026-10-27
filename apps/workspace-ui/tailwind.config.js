/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  darkMode: ['class', '[data-theme="dark"]'],
  theme: {
    extend: {
      colors: {
        mercury: {
          primary: '#5266eb',
          'on-primary': '#ffffff',
          periwinkle: '#9cb4e8',
          canvas: '#fbfcfd',
          surface: '#ffffff',
          'surface-tint': '#fbfbff',
          lavender: '#edecfb',
          night: '#171721',
          'night-raised': '#1e1e2a',
          ink: '#28253b',
          'ink-secondary': '#554e7b',
          'on-dark': '#ededf3',
          fog: '#c3c3cc',
          slate: '#70707d',
          writer: '#3b5bdb',
          sheets: '#0ca678',
          slides: '#e8590c',
          pdf: '#e03131',
          ai: '#7048e8',
        },
      },
      borderRadius: {
        'mercury-xs': '4px',
        'mercury-sm': '6px',
        'mercury-md': '10px',
        'mercury-lg': '14px',
        'mercury-xl': '20px',
        'mercury-pill': '40px',
      },
      fontFamily: {
        sans: ['Inter', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        mono: ['JetBrains Mono', 'Fira Code', 'Menlo', 'monospace'],
      },
    },
  },
  plugins: [],
};
