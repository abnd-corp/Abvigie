// @ts-check
/**
 * Le moteur isométrique : la projection, les couleurs et les formes de base (boîte, cylindre, toits, dôme).
 *
 * Le monde est une grille de tuiles. Une position est `(gx, gy)` en tuiles et une hauteur `z` en pixels.
 * À l'écran, l'axe `gx` descend vers la droite et l'axe `gy` vers la gauche ; le point le plus proche de la
 * caméra est celui où `gx + gy` est le plus grand. Ce fichier ne connaît ni les projets ni GitHub.
 */

/** Largeur et hauteur d'une tuile à l'écran (un losange 2:1). */
export const TW = 64;
export const TH = 32;

/**
 * @param {number} gx
 * @param {number} gy
 * @param {number} [z]
 * @returns {[number, number]}
 */
export function proj(gx, gy, z = 0) {
  return [((gx - gy) * TW) / 2, ((gx + gy) * TH) / 2 - z];
}

/**
 * L'inverse de `proj` pour un point du sol (z = 0).
 * @param {number} x
 * @param {number} y
 * @returns {[number, number]}
 */
export function unproj(x, y) {
  const a = x / (TW / 2);
  const b = y / (TH / 2);
  return [(a + b) / 2, (b - a) / 2];
}

/* ------------------------------------------------------------------ couleurs */

/** @type {Map<string, [number, number, number]>} */
const cacheRgb = new Map();

/**
 * @param {string} hex  « #rrggbb »
 * @returns {[number, number, number]}
 */
export function rgb(hex) {
  let c = cacheRgb.get(hex);
  if (!c) {
    const n = parseInt(hex.slice(1), 16);
    c = [(n >> 16) & 255, (n >> 8) & 255, n & 255];
    cacheRgb.set(hex, c);
  }
  return c;
}

/** @param {number} n */
const octet = (n) => Math.max(0, Math.min(255, Math.round(n)));

/** @param {number[]} c */
function enHex(c) {
  return `#${((1 << 24) | (octet(c[0]) << 16) | (octet(c[1]) << 8) | octet(c[2])).toString(16).slice(1)}`;
}

/**
 * Mélange deux couleurs : `t = 0` donne `a`, `t = 1` donne `b`.
 * @param {string} a
 * @param {string} b
 * @param {number} t
 */
export function melange(a, b, t) {
  const [r1, g1, b1] = rgb(a);
  const [r2, g2, b2] = rgb(b);
  return enHex([r1 + (r2 - r1) * t, g1 + (g2 - g1) * t, b1 + (b2 - b1) * t]);
}

/**
 * Éclaircit (`k > 0`, vers le blanc) ou assombrit (`k < 0`, vers le noir).
 * @param {string} hex
 * @param {number} k  entre -1 et 1
 */
export function nuance(hex, k) {
  return k >= 0 ? melange(hex, '#ffffff', k) : melange(hex, '#000000', -k);
}

/**
 * @param {string} hex
 * @param {number} alpha
 */
export function rgba(hex, alpha) {
  const [r, g, b] = rgb(hex);
  return `rgba(${r},${g},${b},${alpha})`;
}

/**
 * Un entier stable à partir d'un texte (FNV-1a) : sert à varier les fenêtres, les arbres, les couleurs
 * sans rien tirer au hasard, donc sans que la ville change à chaque chargement.
 * @param {string} texte
 */
export function hash(texte) {
  let h = 0x811c9dc5;
  for (let i = 0; i < texte.length; i++) {
    h ^= texte.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return h >>> 0;
}

/* ------------------------------------------------------------------ formes */

/**
 * Les trois couleurs d'un volume éclairé par le haut et la gauche.
 * @typedef {{ haut: string, gauche: string, droite: string, trait: string }} Couleurs
 */

/**
 * @param {string} base
 * @returns {Couleurs}
 */
export function couleursDe(base) {
  return { haut: nuance(base, 0.2), gauche: base, droite: nuance(base, -0.3), trait: rgba('#05081c', 0.55) };
}

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {[number, number][]} points
 * @param {string | CanvasGradient | null} remplissage
 * @param {string | null} [trait]
 */
export function polygone(ctx, points, remplissage, trait = null) {
  ctx.beginPath();
  points.forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)));
  ctx.closePath();
  if (remplissage) {
    ctx.fillStyle = remplissage;
    ctx.fill();
  }
  if (trait) {
    ctx.strokeStyle = trait;
    ctx.lineWidth = 1;
    ctx.lineJoin = 'round';
    ctx.stroke();
  }
}

/**
 * Un dégradé vertical sur une face : plus sombre au pied, pour donner du volume.
 * @param {CanvasRenderingContext2D} ctx
 * @param {string} couleur
 * @param {number} yHaut
 * @param {number} yBas
 */
function degrade(ctx, couleur, yHaut, yBas) {
  const g = ctx.createLinearGradient(0, yHaut, 0, yBas);
  g.addColorStop(0, couleur);
  g.addColorStop(1, nuance(couleur, -0.22));
  return g;
}

/**
 * Une boîte : face gauche (côté `gy` maximal), face droite (côté `gx` maximal) et dessus.
 * @param {CanvasRenderingContext2D} ctx
 * @param {{ x: number, y: number, w: number, d: number, z: number, h: number }} b
 * @param {Couleurs} c
 */
export function boite(ctx, { x, y, w, d, z, h }, c) {
  const haut = [proj(x, y, z + h), proj(x + w, y, z + h), proj(x + w, y + d, z + h), proj(x, y + d, z + h)];
  const gauche = [proj(x, y + d, z), proj(x + w, y + d, z), proj(x + w, y + d, z + h), proj(x, y + d, z + h)];
  const droite = [proj(x + w, y, z), proj(x + w, y + d, z), proj(x + w, y + d, z + h), proj(x + w, y, z + h)];
  const yh = proj(x, y, z + h)[1];
  const yb = proj(x + w, y + d, z)[1];
  polygone(ctx, gauche, degrade(ctx, c.gauche, yh, yb), c.trait);
  polygone(ctx, droite, degrade(ctx, c.droite, yh, yb), c.trait);
  polygone(ctx, haut, c.haut, c.trait);
}

/**
 * Un toit à deux pentes. La crête court le long de `gx` (`crete = 'x'`) ou de `gy` (`crete = 'y'`).
 * @param {CanvasRenderingContext2D} ctx
 * @param {{ x: number, y: number, w: number, d: number, z: number, h: number, crete?: 'x' | 'y' }} t
 * @param {Couleurs} c
 */
export function pignon(ctx, { x, y, w, d, z, h, crete = 'x' }, c) {
  if (crete === 'x') {
    const my = y + d / 2;
    polygone(ctx, [proj(x, y + d, z), proj(x + w, y + d, z), proj(x + w, my, z + h), proj(x, my, z + h)], c.gauche, c.trait);
    polygone(ctx, [proj(x + w, y, z), proj(x + w, y + d, z), proj(x + w, my, z + h)], c.droite, c.trait);
    polygone(ctx, [proj(x, y, z), proj(x, my, z + h), proj(x + w, my, z + h), proj(x + w, y, z)], c.haut, c.trait);
  } else {
    const mx = x + w / 2;
    polygone(ctx, [proj(x, y + d, z), proj(x + w, y + d, z), proj(mx, y + d, z + h)], c.gauche, c.trait);
    polygone(ctx, [proj(x + w, y, z), proj(x + w, y + d, z), proj(mx, y + d, z + h), proj(mx, y, z + h)], c.droite, c.trait);
    polygone(ctx, [proj(x, y, z), proj(mx, y, z + h), proj(mx, y + d, z + h), proj(x, y + d, z)], c.haut, c.trait);
  }
}

/**
 * Une pyramide à base carrée.
 * @param {CanvasRenderingContext2D} ctx
 * @param {{ x: number, y: number, w: number, d: number, z: number, h: number }} p
 * @param {Couleurs} c
 */
export function pyramide(ctx, { x, y, w, d, z, h }, c) {
  const sommet = proj(x + w / 2, y + d / 2, z + h);
  polygone(ctx, [proj(x, y + d, z), proj(x + w, y + d, z), sommet], c.gauche, c.trait);
  polygone(ctx, [proj(x + w, y, z), proj(x + w, y + d, z), sommet], c.droite, c.trait);
}

/**
 * Un cylindre : un flanc dégradé et une ellipse pour le dessus. Le rayon est la moitié de `w`.
 * @param {CanvasRenderingContext2D} ctx
 * @param {{ x: number, y: number, w: number, z: number, h: number }} cy
 * @param {Couleurs} c
 */
export function cylindre(ctx, { x, y, w, z, h }, c) {
  const r = w / 2;
  const [cx, cyy] = proj(x + r, y + r, z);
  const rx = r * TW * Math.SQRT1_2;
  const ry = r * TH * Math.SQRT1_2;
  const flanc = ctx.createLinearGradient(cx - rx, 0, cx + rx, 0);
  flanc.addColorStop(0, c.gauche);
  flanc.addColorStop(0.55, c.gauche);
  flanc.addColorStop(1, c.droite);
  ctx.beginPath();
  ctx.moveTo(cx - rx, cyy);
  ctx.lineTo(cx - rx, cyy - h);
  ctx.ellipse(cx, cyy - h, rx, ry, 0, Math.PI, 0, false);
  ctx.lineTo(cx + rx, cyy);
  ctx.ellipse(cx, cyy, rx, ry, 0, 0, Math.PI, false);
  ctx.closePath();
  ctx.fillStyle = flanc;
  ctx.fill();
  ctx.strokeStyle = c.trait;
  ctx.lineWidth = 1;
  ctx.stroke();
  ctx.beginPath();
  ctx.ellipse(cx, cyy - h, rx, ry, 0, 0, Math.PI * 2);
  ctx.fillStyle = c.haut;
  ctx.fill();
  ctx.stroke();
}

/**
 * Un dôme : une demi-sphère posée sur sa base.
 * @param {CanvasRenderingContext2D} ctx
 * @param {{ x: number, y: number, w: number, z: number, h: number }} d
 * @param {Couleurs} c
 */
export function dome(ctx, { x, y, w, z, h }, c) {
  const r = w / 2;
  const [cx, cy] = proj(x + r, y + r, z);
  const rx = r * TW * Math.SQRT1_2;
  const ry = r * TH * Math.SQRT1_2;
  const g = ctx.createRadialGradient(cx - rx * 0.35, cy - h * 0.7, 2, cx, cy - h * 0.3, Math.max(rx, h) * 1.05);
  g.addColorStop(0, nuance(c.haut, 0.25));
  g.addColorStop(0.5, c.gauche);
  g.addColorStop(1, c.droite);
  ctx.beginPath();
  ctx.ellipse(cx, cy, rx, ry, 0, 0, Math.PI, false);
  ctx.ellipse(cx, cy, rx, h + ry * 0.2, 0, Math.PI, 0, false);
  ctx.closePath();
  ctx.fillStyle = g;
  ctx.fill();
  ctx.strokeStyle = c.trait;
  ctx.lineWidth = 1;
  ctx.stroke();
}

/**
 * Une ombre portée douce, vers le bas et la droite (la lumière vient du haut gauche).
 * @param {CanvasRenderingContext2D} ctx
 * @param {{ x: number, y: number, w: number, d: number }} e  l'emprise au sol
 * @param {number} h  la hauteur de l'objet
 * @param {number} [alpha]
 */
export function ombre(ctx, { x, y, w, d }, h, alpha = 0.3) {
  const dx = h * 0.55;
  const dy = h * 0.22;
  const pts = [proj(x, y + d), proj(x + w, y + d), proj(x + w, y), proj(x, y)];
  const decale = pts.map(([px, py]) => /** @type {[number, number]} */ ([px + dx, py + dy]));
  const toutes = [pts[0], pts[1], decale[1], decale[2], decale[3], decale[0]];
  ctx.beginPath();
  toutes.forEach(([px, py], i) => (i ? ctx.lineTo(px, py) : ctx.moveTo(px, py)));
  ctx.closePath();
  ctx.fillStyle = `rgba(2,4,16,${alpha})`;
  ctx.fill();
}

/**
 * Le contour au sol d'une emprise, éventuellement en pointillés.
 * @param {CanvasRenderingContext2D} ctx
 * @param {{ x: number, y: number, w: number, d: number }} e
 * @param {string} couleur
 * @param {number} [largeur]
 * @param {number[]} [tirets]
 */
export function contourSol(ctx, { x, y, w, d }, couleur, largeur = 2, tirets = []) {
  ctx.beginPath();
  [proj(x, y), proj(x + w, y), proj(x + w, y + d), proj(x, y + d)].forEach(([px, py], i) => (i ? ctx.lineTo(px, py) : ctx.moveTo(px, py)));
  ctx.closePath();
  ctx.setLineDash(tirets);
  ctx.strokeStyle = couleur;
  ctx.lineWidth = largeur;
  ctx.stroke();
  ctx.setLineDash([]);
}
