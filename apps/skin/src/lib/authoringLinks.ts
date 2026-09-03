const DEFAULT_DOCS_BASE = "https://terrarium.mintlify.app";

export const docsBase =
  (import.meta.env.VITE_DOCS_URL as string | undefined)?.replace(/\/$/, "") || DEFAULT_DOCS_BASE;

export const authorLinks = {
  docsHome: docsBase,
  deployGuide: `${docsBase}/getting-started/deploy-from-game`,
} as const;
