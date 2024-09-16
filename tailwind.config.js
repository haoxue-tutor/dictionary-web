/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["*.html", "./src/**/*.rs",],
  theme: {
    extend: {
      spacing: {
        '128': '32rem',
      },
    },
  },
  plugins: [],
}

