import type { Config } from "tailwindcss";

export default {
  content: ["./src/**/*.{html,js,svelte,ts}", "./index.html"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        accent: {
          DEFAULT: "#6366f1",
          light: "#818cf8",
          dark: "#4f46e5",
        },
      },
      fontFamily: {
        sans: ["-apple-system", "BlinkMacSystemFont", "SF Pro Display", "sans-serif"],
        mono: ["SF Mono", "Menlo", "Monaco", "Cascadia Code", "monospace"],
      },
      borderRadius: {
        "2xl": "16px",
        "3xl": "20px",
      },
      backdropBlur: {
        xl: "20px",
      },
      animation: {
        "spring-in": "spring-in 180ms cubic-bezier(0.34, 1.56, 0.64, 1) forwards",
        "fade-up": "fade-up 200ms ease-out forwards",
        "copy-flash": "copy-flash 150ms ease-out",
        "slide-in": "slide-in 200ms ease-out forwards",
      },
      keyframes: {
        "spring-in": {
          "0%": { opacity: "0", transform: "scale(0.96)" },
          "100%": { opacity: "1", transform: "scale(1)" },
        },
        "fade-up": {
          "0%": { opacity: "0", transform: "translateY(8px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        "copy-flash": {
          "0%": { backgroundColor: "transparent" },
          "50%": { backgroundColor: "rgba(99, 102, 241, 0.2)" },
          "100%": { backgroundColor: "transparent" },
        },
        "slide-in": {
          "0%": { opacity: "0", transform: "translateX(-8px)" },
          "100%": { opacity: "1", transform: "translateX(0)" },
        },
      },
    },
  },
  plugins: [],
} satisfies Config;
