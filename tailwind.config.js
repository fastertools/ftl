/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./ui/templates/**/*.{templ,go}",
    "./ui/gen/**/*.go",
    "./internal/handlers/**/*.go",
    "./internal/html/**/*.go",
  ],
  theme: {
    extend: {
      colors: {
        // Custom colors from CUE can be added here if needed
        'ftl-primary': '#1a1a1a',
        'ftl-secondary': '#2a2a2a',
      }
    },
  },
  plugins: [],
}