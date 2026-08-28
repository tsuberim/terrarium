/** Resolve cross-app URLs from optional <meta name="terrarium-*"> tags. */
export function terrariumUrls() {
  const meta = (name, fallback) =>
    document.querySelector(`meta[name="${name}"]`)?.getAttribute("content")?.trim() || fallback;

  return {
    home: meta("terrarium-home", "/"),
    about: meta("terrarium-about", "/about.html"),
    play: meta("terrarium-play", "/"),
    console: meta("terrarium-console", "/"),
  };
}

export function injectShellNav(containerId, current) {
  const urls = terrariumUrls();
  const el = document.getElementById(containerId);
  if (!el) return;

  const items = [
    { key: "home", href: urls.home, label: "home" },
    { key: "about", href: urls.about, label: "about" },
    { key: "play", href: urls.play, label: "play" },
    { key: "console", href: urls.console, label: "console" },
  ];

  el.innerHTML = `
    <a class="brand" href="${urls.home}">terrarium</a>
    ${items
      .filter((i) => i.key !== "home")
      .map(
        (i) =>
          `<a href="${i.href}"${current === i.key ? ' aria-current="page"' : ""}>${i.label}</a>`,
      )
      .join("")}
  `;
}
