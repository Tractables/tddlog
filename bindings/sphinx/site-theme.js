"use strict";

// The Python and C pages' theme, kept in step with the rest of the site's.
// The site's pages keep the reader's choice as `tddlog-theme` (light or
// dark; none for the system's), Furo as `theme` (light, dark or auto). Furo
// loads this script at the end of the body, after it has set the page's
// theme from its own choice, so the script sets both to the site's choice,
// and hands a choice made here back to the site.
(() => {
  let theme;
  try {
    const chosen = localStorage.getItem("tddlog-theme");
    theme = chosen === "light" || chosen === "dark" ? chosen : "auto";
    localStorage.setItem("theme", theme);
  } catch (failure) {
    return;
  }
  const follow = () => {
    document.body.dataset.theme = theme;
    new MutationObserver(() => {
      const chosen = document.body.dataset.theme;
      try {
        if (chosen === "light" || chosen === "dark") localStorage.setItem("tddlog-theme", chosen);
        else localStorage.removeItem("tddlog-theme");
      } catch (failure) {}
    }).observe(document.body, { attributes: true, attributeFilter: ["data-theme"] });
  };
  if (document.body === null) document.addEventListener("DOMContentLoaded", follow);
  else follow();
})();
