"use strict";

// The theme of every page of the site: the one chosen on a page before, else
// the system's. Loaded in the head, so that it is set before the page is
// drawn and does not flash the other one; the button with id `theme`, where
// a page has one, switches between light and dark and remembers the choice.
(() => {
  const root = document.documentElement;
  try {
    const theme = localStorage.getItem("tddlog-theme");
    if (theme === "light" || theme === "dark") root.dataset.theme = theme;
  } catch (failure) {}

  const system = window.matchMedia("(prefers-color-scheme: dark)");
  const dark = () => (root.dataset.theme ?? (system.matches ? "dark" : "light")) === "dark";
  document.addEventListener("DOMContentLoaded", () => {
    const button = document.getElementById("theme");
    if (button === null) return;
    const show = () => {
      const other = dark() ? "Light" : "Dark";
      button.textContent = other;
      button.setAttribute("aria-label", `Switch to the ${other.toLowerCase()} theme`);
    };
    button.addEventListener("click", () => {
      root.dataset.theme = dark() ? "light" : "dark";
      try {
        localStorage.setItem("tddlog-theme", root.dataset.theme);
      } catch (failure) {}
      show();
    });
    system.addEventListener("change", show);
    show();
  });
})();
