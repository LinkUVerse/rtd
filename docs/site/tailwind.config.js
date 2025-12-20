// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

const defaultTheme = require("tailwindcss/defaultTheme");

module.exports = {
  corePlugins: {
    preflight: false, // disable Tailwind's reset
  },
  content: ["./src/**/*.{js,jsx,ts,tsx}", "./docs/**/*.mdx"], // my markdown stuff is in ../docs, not /src
  safelist: ["text-rtd-success-dark"],
  darkMode: ["class", '[data-theme="dark"]'], // hooks into docusaurus' dark mode settings
  theme: {
    extend: {
      fontFamily: {
        sans: ["Inter", ...defaultTheme.fontFamily.sans],
        twkeverett: ["Twkeverett"],
      },
      colors: {
        "rtd-black": "var(--rtd-black)",
        "rtd-blue-primary": "rgb(var(--rtd-blue-primary)/<alpha-value>)",
        "rtd-blue": "var(--rtd-blue)",
        "rtd-blue-bright": "rgb(var(--rtd-blue-bright)/<alpha-value>)",
        "rtd-blue-light": "rgb(var(--rtd-blue-light)/<alpha-value>)",
        "rtd-blue-lighter": "var(--rtd-blue-lighter)",
        "rtd-blue-dark": "rgb(var(--rtd-blue-dark)/<alpha-value>)",
        "rtd-blue-darker": "var(--rtd-blue-darker)",
        "rtd-hero": "var(--rtd-hero)",
        "rtd-hero-dark": "var(--rtd-hero-dark)",
        "rtd-steel": "var(--rtd-steel)",
        "rtd-steel-dark": "var(--rtd-steel-dark)",
        "rtd-steel-darker": "var(--rtd-steel-darker)",
        "rtd-header-nav": "var(--rtd-header-nav)",
        "rtd-success": "var(--rtd-success)",
        "rtd-success-dark": "var(--rtd-success-dark)",
        "rtd-success-light": "var(--rtd-success-light)",
        "rtd-issue": "var(--rtd-issue)",
        "rtd-issue-dark": "var(--rtd-issue-dark)",
        "rtd-issue-light": "var(--rtd-issue-light)",
        "rtd-warning": "var(--rtd-warning)",
        "rtd-warning-dark": "var(--rtd-warning-dark)",
        "rtd-warning-light": "var(--rtd-warning-light)",
        "rtd-code": "var(--rtd-code)",
        "rtd-gray-3s": "rgb(var(--rtd-gray-3s)/<alpha-value>)",
        "rtd-gray-5s": "rgb(var(--rtd-gray-5s)/<alpha-value>)",
        "rtd-gray": {
          35: "rgb(var(--rtd-gray-35)/<alpha-value>)",
          40: "rgb(var(--rtd-gray-40)/<alpha-value>)",
          45: "rgb(var(--rtd-gray-45)/<alpha-value>)",
          50: "var(--rtd-gray-50)",
          55: "rgb(var(--rtd-gray-55)/<alpha-value>)",
          60: "var(--rtd-gray-60)",
          65: "var(--rtd-gray-65)",
          70: "var(--rtd-gray-70)",
          75: "var(--rtd-gray-75)",
          80: "var(--rtd-gray-80)",
          85: "var(--rtd-gray-85)",
          90: "var(--rtd-gray-90)",
          95: "var(--rtd-gray-95)",
          100: "var(--rtd-gray-100)",
        },
        "rtd-grey": {
          35: "rgb(var(--rtd-gray-35)/<alpha-value>)",
          40: "rgb(var(--rtd-gray-40)/<alpha-value>)",
          45: "rgb(var(--rtd-gray-45)/<alpha-value>)",
          50: "var(--rtd-gray-50)",
          55: "rgb(var(--rtd-gray-55)/<alpha-value>)",
          60: "var(--rtd-gray-60)",
          65: "var(--rtd-gray-65)",
          70: "var(--rtd-gray-70)",
          75: "var(--rtd-gray-75)",
          80: "var(--rtd-gray-80)",
          85: "var(--rtd-gray-85)",
          90: "var(--rtd-gray-90)",
          95: "var(--rtd-gray-95)",
          100: "var(--rtd-gray-100)",
        },
        "rtd-disabled": "rgb(var(--rtd-disabled)/<alpha-value>)",
        "rtd-link-color-dark": "var(--rtd-link-color-dark)",
        "rtd-link-color-light": "var(--rtd-link-color-light)",
        "rtd-ghost-white": "var(--rtd-ghost-white)",
        "rtd-ghost-dark": "var(--rtd-ghost-dark)",
        "ifm-background-color-dark": "var(--ifm-background-color-dark)",
        "rtd-white": "rgb(var(--rtd-white)/<alpha-value>)",
        "rtd-card-dark": "rgb(var(--rtd-card-dark)/<alpha-value>)",
        "rtd-card-darker": "rgb(var(--rtd-card-darker)/<alpha-value>)",
      },
      borderRadius: {
        rtd: "40px",
      },
      boxShadow: {
        rtd: "0px 0px 4px rgba(0, 0, 0, 0.02)",
        "rtd-button": "0px 1px 2px rgba(16, 24, 40, 0.05)",
        "rtd-notification": "0px 0px 20px rgba(29, 55, 87, 0.11)",
      },
      gradientColorStopPositions: {
        36: "36%",
      },
    },
  },
  plugins: [
    function ({ addUtilities }) {
      const arrowMask = `url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'><path d='M8.12 4.12a1 1 0 0 1 1.41 0l6.35 6.35a1 1 0 0 1 0 1.41l-6.35 6.35a1 1 0 1 1-1.41-1.41L13.59 12 8.12 6.53a1 1 0 0 1 0-1.41z'/></svg>") no-repeat center / contain`;

      addUtilities({
        ".mask-arrow": {
          transition: "transform 0.2s ease",
          background: "currentColor",
          WebkitMask: arrowMask,
          mask: arrowMask,
        },
      });
    },
  ],
};
