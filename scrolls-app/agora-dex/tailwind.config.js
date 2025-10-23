/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'caesar-gold': '#FFD700',
        'caesar-red': '#DC143C',
        'caesar-dark': '#1a1a1a',
        'caesar-gray': '#2a2a2a',
      },
      fontFamily: {
        'caesar': ['Inter', 'sans-serif'],
      },
    },
  },
  plugins: [],
}