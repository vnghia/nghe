/** @type {import('tailwindcss').Config} */
const themes = JSON.parse(
  require("node:fs").readFileSync("./daisy-themes.json", "utf8")
);

module.exports = {
  mode: "all",
  content: ["./src/**/*.{rs,html,css}", "./dist/**/*.html"],
  theme: {
    extend: {},
  },
  plugins: [require("daisyui")],
  daisyui: {
    themes,
  },
};
