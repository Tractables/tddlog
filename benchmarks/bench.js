"use strict";
// The benchmark pages: a file too large for the page is loaded into its
// box when its tab is first shown; without a script its link opens it.
(() => {
  const fill = async (pre) => {
    if (pre.dataset.done) return;
    pre.dataset.done = "1";
    try {
      const response = await fetch(pre.dataset.src);
      if (!response.ok) throw new Error(String(response.status));
      pre.textContent = await response.text();
    } catch (failure) {
      pre.remove();
    }
  };
  const show = () => {
    for (const input of document.querySelectorAll(".ptabs > input:checked")) {
      const panel = document.getElementById("p" + input.id.slice(1));
      if (panel) for (const pre of panel.querySelectorAll("pre.lazy")) fill(pre);
    }
  };
  document.addEventListener("change", (event) => { if (event.target.matches(".ptabs > input")) show(); });
  document.addEventListener("DOMContentLoaded", show);
})();
