// @ts-check
/**
 * Les vignettes : le bâtiment d'un projet dessiné en petit, fixe, dans une carte de la liste ou en tête d'une fiche.
 * C'est le même dessin que sur la carte (`batiments.js`) : un projet a partout le même visage.
 *
 * Ce fichier est aussi le point d'entrée des pages de projet, qui n'ont pas de carte.
 */
import { description, dessiner, emprise } from './batiments.js';
import { proj } from './iso.js';

/**
 * Dessine toutes les vignettes d'une page ou d'un morceau de page.
 * @param {ParentNode} racine
 */
export function dessinerVignettes(racine) {
  racine.querySelectorAll('canvas[data-vignette]').forEach((c) => {
    if (c instanceof HTMLCanvasElement) vignette(c);
  });
}

/** @param {HTMLCanvasElement} c */
function vignette(c) {
  const ctx = c.getContext('2d');
  const slug = c.dataset.vignette;
  if (!ctx || !slug) return;
  description(slug);
  const lumieres = Number(c.dataset.lumieres ?? 3);
  const accent = getComputedStyle(c).getPropertyValue('--ac').trim() || '#9ba4c8';

  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const L = Number(c.getAttribute('width')) || 96;
  const H = Number(c.getAttribute('height')) || 112;
  c.width = L * dpr;
  c.height = H * dpr;
  c.style.width = `${L}px`;
  c.style.height = `${H}px`;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, L, H);

  // cadrage : on cherche la boîte de l'image du bâtiment, puis on la centre et on la réduit pour qu'elle tienne
  const e = emprise(slug);
  const points = [
    proj(e.x, e.y + e.d, 0), proj(e.x + e.w, e.y, 0), proj(e.x + e.w, e.y + e.d, 0), proj(e.x, e.y, e.h), proj(e.x + e.w, e.y, e.h), proj(e.x, e.y + e.d, e.h),
  ];
  const x0 = Math.min(...points.map((p) => p[0])) - 6;
  const x1 = Math.max(...points.map((p) => p[0])) + 6;
  const y0 = Math.min(...points.map((p) => p[1])) - 24;
  const y1 = Math.max(...points.map((p) => p[1])) + 8;
  const echelle = Math.min((L - 6) / (x1 - x0), (H - 6) / (y1 - y0), 1);
  ctx.translate(L / 2 - ((x0 + x1) / 2) * echelle, H / 2 - ((y0 + y1) / 2) * echelle);
  ctx.scale(echelle, echelle);
  dessiner(ctx, { slug, accent, lumieres }, { ox: 0, oy: 0, t: 0, anim: false, zoom: 1 });
}

// sur une page de projet, il n'y a pas de carte : ce module dessine les vignettes tout seul
if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', () => dessinerVignettes(document));
else dessinerVignettes(document);
