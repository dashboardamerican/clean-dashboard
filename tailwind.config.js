/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Energy colors matching original design
        solar: '#fbbc05',
        wind: '#4285f4',
        battery: '#34a853',
        gas: '#ea4335',
        'clean-firm': '#FF7900',
        storage: '#673ab7',
        'gas-ccs': '#009688',
        dr: '#9AA0A6',
      },
      fontFamily: {
        sans: ['Google Sans', 'Roboto', 'sans-serif'],
      },
    },
  },
  plugins: [],
}
