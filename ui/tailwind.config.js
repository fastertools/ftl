/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./templates/**/*.{templ,go}",
    "./gen/**/*.go",
    "../internal/handlers/**/*.go",
    "../internal/html/**/*.go",
  ],
  // Custom colors managed via CUE schema system in ui/schema/styles.cue
  // and generated as Go constants in ui/gen/styles_gen.go
}