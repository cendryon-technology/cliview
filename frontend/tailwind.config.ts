import type { Config } from "tailwindcss";

export default {
  content: ["./src/**/*.{html,svelte,ts}"],
  theme: {
    extend: {
      colors: {
        bg: "#0b0d10",
        panel: "#14171c",
        border: "#22262e",
        muted: "#8a8f98",
        accent: "#7cb7ff"
      },
      fontFamily: {
        mono: ["ui-monospace", "SFMono-Regular", "Menlo", "monospace"]
      }
    }
  },
  plugins: []
} satisfies Config;
