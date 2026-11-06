// @ts-check
/**
 * Les bâtiments : chaque projet est décrit par quelques volumes (boîtes, cylindres, toits…) et quelques détails animés
 * qui renvoient à ce que fait le projet. Pour en ajouter un : une entrée dans `DESCRIPTIONS`. Voir `docs/ajouter-un-immeuble.md`.
 *
 * Les positions sont en tuiles, relatives à la parcelle du bâtiment (qui fait 3 × 3 tuiles), et les hauteurs en pixels.
 */
import { boite, couleursDe, cylindre, dome, hash, melange, nuance, ombre, pignon, polygone, proj, pyramide, rgba } from './iso.js';

/**
 * @typedef {{
 *   t: 'box' | 'cyl' | 'gable' | 'pyr' | 'dome',
 *   x: number, y: number, w: number, d?: number, h: number, z?: number,
 *   c?: string, crete?: 'x' | 'y', fen?: boolean
 * }} Volume
 */
/** @typedef {{ volumes: Volume[], deco?: string[] }} Description */
/**
 * Ce qu'il faut savoir d'un projet pour le dessiner.
 * @typedef {{ slug: string, accent: string, lumieres: number, archive?: boolean }} Etat
 */
/**
 * Le contexte d'un dessin : où, quand, et si l'on anime.
 * @typedef {{ ox: number, oy: number, t: number, anim: boolean, zoom: number }} Scene
 */

const AMBRE = '#ffc75f';

/** @type {Record<string, Description>} */
export const DESCRIPTIONS = {
  // un atelier : toit en dents de scie, cheminée qui fume, engrenage sur la façade
  abatelier: {
    volumes: [
      { t: 'box', x: 0.2, y: 0.3, w: 2.6, d: 2.4, h: 26 },
      { t: 'gable', x: 0.2, y: 0.3, w: 0.87, d: 2.4, z: 26, h: 15, crete: 'y', c: 'toit' },
      { t: 'gable', x: 1.07, y: 0.3, w: 0.87, d: 2.4, z: 26, h: 15, crete: 'y', c: 'toit' },
      { t: 'gable', x: 1.94, y: 0.3, w: 0.86, d: 2.4, z: 26, h: 15, crete: 'y', c: 'toit' },
      { t: 'box', x: 2.3, y: 0.4, w: 0.34, d: 0.34, z: 26, h: 30, c: 'accent', fen: false },
    ],
    deco: ['fumee:2.47,0.57,56', 'engrenage:droite,1.2,13'],
  },
  // la carte de France : un toit en forme d'Hexagone, quadrillé en départements
  abcarte: {
    volumes: [
      { t: 'box', x: 0.3, y: 0.4, w: 2.4, d: 2.2, h: 22 },
      { t: 'box', x: 0.35, y: 0.45, w: 0.7, d: 0.7, z: 22, h: 14, c: 'clair' },
    ],
    deco: ['hexagone:1.5,1.5,22', 'epingle:2.1,0.9,36'],
  },
  // une messagerie chiffrée : une tour d'antenne dont les ondes rayonnent, un cadenas sur la façade
  abcom: {
    volumes: [
      { t: 'box', x: 0.3, y: 0.3, w: 2.4, d: 2.4, h: 18 },
      { t: 'box', x: 0.95, y: 0.95, w: 1.1, d: 1.1, z: 18, h: 66 },
    ],
    deco: ['antenne:1.5,1.5,84', 'cadenas:gauche,1.1,6'],
  },
  // un DNS chiffré : un dôme surmonté d'un globe, un bouclier sur la façade
  abdns: {
    volumes: [
      { t: 'box', x: 0.4, y: 0.4, w: 2.2, d: 2.2, h: 36 },
      { t: 'dome', x: 0.5, y: 0.5, w: 2, d: 2, z: 36, h: 24, c: 'accent', fen: false },
    ],
    deco: ['globe:1.5,1.5,70', 'bouclier:gauche,1.1,8'],
  },
  // un gestionnaire de fichiers sur Raspberry Pi : une maison basse, une rivière de fichiers qui coule devant
  abflow: {
    volumes: [
      { t: 'box', x: 0.2, y: 0.2, w: 2.6, d: 1.5, h: 24 },
      { t: 'gable', x: 0.2, y: 0.2, w: 2.6, d: 1.5, z: 24, h: 14, crete: 'x', c: 'toit' },
    ],
    deco: ['riviere:0,2.1,3,0.7', 'baies:1.5,0.5,38'],
  },
  // des loueurs, sur une carte : une épingle qui flotte au-dessus du toit
  abloue: {
    volumes: [
      { t: 'box', x: 0.5, y: 0.5, w: 2, d: 2, h: 46 },
      { t: 'box', x: 0.5, y: 0.5, w: 2, d: 2, z: 46, h: 5, c: 'accent', fen: false },
    ],
    deco: ['epingle:1.5,1.5,70', 'clim:0.7,0.7,51'],
  },
  // un catalogue de modules : des blocs de tailles différentes, empilés
  abnd: {
    volumes: [
      { t: 'box', x: 0.2, y: 0.2, w: 1.5, d: 1.5, h: 38 },
      { t: 'box', x: 0.9, y: 0.95, w: 1.9, d: 1.8, h: 24, c: 'accent' },
      { t: 'box', x: 0.35, y: 0.5, w: 1.0, d: 1.0, z: 38, h: 22, c: 'clair' },
      { t: 'box', x: 1.8, y: 0.2, w: 1.0, d: 1.0, h: 58 },
    ],
    deco: ['clim:2.2,0.9,58'],
  },
  // la santé : un bâtiment de soin, une croix lumineuse sur le toit
  absante: {
    volumes: [
      { t: 'box', x: 0.2, y: 0.4, w: 2.6, d: 2.2, h: 28 },
      { t: 'box', x: 0.9, y: 0.8, w: 1.2, d: 1.4, z: 28, h: 22 },
    ],
    deco: ['croix:1.5,1.5,50'],
  },
  // des tickets : un bâtiment en escalier qui monte, un ticket accroché
  abvance: {
    volumes: [
      { t: 'box', x: 0.2, y: 0.2, w: 2.6, d: 2.6, h: 20 },
      { t: 'box', x: 0.5, y: 0.5, w: 2, d: 2, z: 20, h: 20, c: 'clair' },
      { t: 'box', x: 0.8, y: 0.8, w: 1.4, d: 1.4, z: 40, h: 20, c: 'accent' },
      { t: 'box', x: 1.1, y: 1.1, w: 0.8, d: 0.8, z: 60, h: 20 },
    ],
    deco: ['fanion-haut:1.5,1.5,80'],
  },
  // la vigie : un phare dont le faisceau balaie la ville
  abvigie: {
    volumes: [
      { t: 'box', x: 0.4, y: 0.4, w: 2.2, d: 2.2, h: 8, c: 'sombre', fen: false },
      { t: 'cyl', x: 0.7, y: 0.7, w: 1.6, z: 8, h: 44, fen: false },
      { t: 'cyl', x: 0.9, y: 0.9, w: 1.2, z: 52, h: 30, c: 'clair', fen: false },
      { t: 'cyl', x: 1.05, y: 1.05, w: 0.9, z: 82, h: 14, c: 'lampe', fen: false },
      { t: 'pyr', x: 1.0, y: 1.0, w: 1.0, d: 1.0, z: 96, h: 14, c: 'accent' },
    ],
    deco: ['faisceau:1.5,1.5,90'],
  },
  // un audit d'images Docker : une tour de guet et son projecteur, des conteneurs au pied
  abwatch: {
    volumes: [
      { t: 'box', x: 0.7, y: 0.7, w: 0.18, d: 0.18, h: 46, c: 'sombre', fen: false },
      { t: 'box', x: 2.1, y: 0.7, w: 0.18, d: 0.18, h: 46, c: 'sombre', fen: false },
      { t: 'box', x: 0.7, y: 2.1, w: 0.18, d: 0.18, h: 46, c: 'sombre', fen: false },
      { t: 'box', x: 2.1, y: 2.1, w: 0.18, d: 0.18, h: 46, c: 'sombre', fen: false },
      { t: 'box', x: 0.55, y: 0.55, w: 1.9, d: 1.9, z: 46, h: 26 },
      { t: 'pyr', x: 0.45, y: 0.45, w: 2.1, d: 2.1, z: 72, h: 16, c: 'accent' },
      { t: 'box', x: 0.1, y: 2.3, w: 1.0, d: 0.6, h: 12, c: 'clair', fen: false },
      { t: 'box', x: 2.0, y: 0.1, w: 0.9, d: 0.6, h: 12, c: 'accent', fen: false },
    ],
    deco: ['projecteur:1.5,1.5,64'],
  },
  // un convertisseur de documents : deux blocs reliés par des flèches qui défilent
  abx2t: {
    volumes: [
      { t: 'box', x: 0.2, y: 0.3, w: 1.1, d: 2.4, h: 40, c: 'clair' },
      { t: 'box', x: 1.9, y: 0.3, w: 1.0, d: 2.4, h: 60 },
      { t: 'box', x: 1.9, y: 0.3, w: 1.0, d: 2.4, z: 60, h: 6, c: 'accent', fen: false },
    ],
    deco: ['convoyeur:1.3,1.5,1.9,1.5'],
  },
  // une tour qui s'enfonce dans l'eau, une pièce qui tourne au sommet
  abyss: {
    volumes: [{ t: 'box', x: 0.9, y: 0.9, w: 1.2, d: 1.2, z: -8, h: 104, c: 'sombre' }],
    deco: ['eau:0.2,0.2,2.6,2.6', 'piece:1.5,1.5,108'],
  },
  // la gouvernance : un palais à colonnes et son fronton
  gouvernance: {
    volumes: [
      { t: 'box', x: 0.2, y: 0.3, w: 2.6, d: 2.4, h: 5, c: 'clair', fen: false },
      { t: 'box', x: 0.4, y: 0.5, w: 2.2, d: 2, z: 5, h: 5, c: 'clair', fen: false },
      { t: 'box', x: 0.6, y: 0.7, w: 1.8, d: 1.6, z: 10, h: 34, c: 'clair', fen: false },
      { t: 'gable', x: 0.5, y: 0.6, w: 2, d: 1.8, z: 44, h: 18, crete: 'x', c: 'toit' },
    ],
    deco: ['colonnes:0.7,2.3,5,1.7,10,34', 'balance:1.5,1.5,66'],
  },
  // la configuration de l'orga : une tour de contrôle et son engrenage
  'org-config': {
    volumes: [
      { t: 'cyl', x: 1.1, y: 1.1, w: 0.8, z: 0, h: 70, fen: false },
      { t: 'box', x: 0.5, y: 0.5, w: 2, d: 2, z: 70, h: 20, c: 'clair' },
      { t: 'box', x: 0.3, y: 0.3, w: 2.4, d: 2.4, h: 8, c: 'sombre', fen: false },
    ],
    deco: ['engrenage-haut:1.5,1.5,100'],
  },
  // le modèle de projet : un bâtiment en chantier, sous échafaudage
  template: {
    volumes: [{ t: 'box', x: 0.8, y: 0.8, w: 1.4, d: 1.4, h: 50, c: 'sombre' }],
    deco: ['echafaudage:0.6,0.6,1.8,1.8,60', 'grue:0.4,0.4,60'],
  },
};

/**
 * Le bâtiment d'un projet qui n'a pas encore le sien : une forme simple, toujours la même pour un même nom.
 * @param {string} slug
 * @returns {Description}
 */
function generique(slug) {
  const h = hash(slug);
  const w = 1.6 + (h % 8) / 10;
  const d = 1.6 + ((h >>> 3) % 8) / 10;
  const hauteur = 34 + ((h >>> 6) % 40);
  /** @type {Volume[]} */
  const volumes = [{ t: 'box', x: (3 - w) / 2, y: (3 - d) / 2, w, d, h: hauteur }];
  const toit = (h >>> 9) % 3;
  if (toit === 1) volumes.push({ t: 'gable', x: (3 - w) / 2, y: (3 - d) / 2, w, d, z: hauteur, h: 14, crete: 'x', c: 'toit' });
  if (toit === 2) volumes.push({ t: 'pyr', x: (3 - w) / 2, y: (3 - d) / 2, w, d, z: hauteur, h: 16, c: 'toit' });
  return { volumes, deco: toit === 0 ? [`clim:${(3 - w) / 2 + 0.3},${(3 - d) / 2 + 0.3},${hauteur}`] : [] };
}

/** @param {string} slug */
export const description = (slug) => DESCRIPTIONS[slug] ?? generique(slug);

/**
 * L'emprise d'un bâtiment (en tuiles, relative à sa parcelle) et sa hauteur totale. Sert à l'ombre, aux pointillés
 * des chantiers, à l'étiquette et au survol.
 * @param {string} slug
 */
export function emprise(slug) {
  const vs = description(slug).volumes;
  let x0 = 9;
  let y0 = 9;
  let x1 = 0;
  let y1 = 0;
  let haut = 0;
  for (const v of vs) {
    x0 = Math.min(x0, v.x);
    y0 = Math.min(y0, v.y);
    x1 = Math.max(x1, v.x + v.w);
    y1 = Math.max(y1, v.y + (v.d ?? v.w));
    haut = Math.max(haut, (v.z ?? 0) + v.h);
  }
  return { x: x0, y: y0, w: x1 - x0, d: y1 - y0, h: haut };
}

/* ------------------------------------------------------------------ dessin */

/** Les parts de fenêtres allumées pour chaque niveau d'activité, de 0 à 4. */
const PART_ALLUMEE = [0, 0.22, 0.45, 0.7, 0.92];

/**
 * @param {Etat} etat
 * @param {string} cle
 * @returns {string}
 */
function couleurVolume(etat, cle) {
  const dormant = etat.lumieres === 0;
  const base = dormant ? melange('#161c38', etat.accent, 0.05) : melange('#3a4580', etat.accent, 0.4);
  switch (cle) {
    case 'accent':
      return dormant ? melange(base, etat.accent, 0.12) : melange(base, etat.accent, 0.75);
    case 'toit':
      return dormant ? base : melange(base, etat.accent, 0.55);
    case 'clair':
      return dormant ? nuance(base, 0.08) : nuance(base, 0.22);
    case 'sombre':
      return nuance(base, -0.35);
    case 'lampe':
      return dormant ? nuance(base, 0.1) : AMBRE;
    default:
      return base;
  }
}

/**
 * Dessine un bâtiment : ombre, volumes dans l'ordre de la profondeur, fenêtres, puis ses détails animés.
 * @param {CanvasRenderingContext2D} ctx
 * @param {Etat} etat
 * @param {Scene} s
 */
export function dessiner(ctx, etat, s) {
  const desc = description(etat.slug);
  const e = emprise(etat.slug);
  ombre(ctx, { x: s.ox + e.x, y: s.oy + e.y, w: e.w, d: e.d }, e.h, etat.lumieres === 0 ? 0.4 : 0.3);

  const ordre = [...desc.volumes].sort((a, b) => a.x + a.w + (a.y + (a.d ?? a.w)) + (a.z ?? 0) / 100 - (b.x + b.w + (b.y + (b.d ?? b.w)) + (b.z ?? 0) / 100));
  ordre.forEach((v, i) => {
    const c = couleursDe(couleurVolume(etat, v.c ?? 'corps'));
    const g = { x: s.ox + v.x, y: s.oy + v.y, w: v.w, d: v.d ?? v.w, z: v.z ?? 0, h: v.h };
    if (v.t === 'box') {
      boite(ctx, g, c);
      if (v.fen !== false) fenetres(ctx, g, etat, `${i}`, s);
      if (i === 0 && g.z === 0 && g.w >= 1.2 && v.fen !== false) porte(ctx, g, etat);
    } else if (v.t === 'cyl') {
      cylindre(ctx, g, c);
    } else if (v.t === 'gable') {
      pignon(ctx, { ...g, crete: v.crete }, c);
    } else if (v.t === 'pyr') {
      pyramide(ctx, g, c);
    } else if (v.t === 'dome') {
      dome(ctx, g, c);
    }
  });
  for (const nom of desc.deco ?? []) detail(ctx, nom, etat, s);
}

/**
 * Une porte au pied de la façade de gauche : un cadre sombre, et une fente de lumière si le projet est actif.
 * @param {CanvasRenderingContext2D} ctx
 * @param {{ x: number, y: number, w: number, d: number, z: number, h: number }} g
 * @param {Etat} etat
 */
function porte(ctx, g, etat) {
  const u = g.w * 0.5 - 0.17;
  const P = (/** @type {number} */ a, /** @type {number} */ v) => proj(g.x + a, g.y + g.d, g.z + v);
  polygone(ctx, [P(u - 0.04, 0), P(u + 0.38, 0), P(u + 0.38, 12), P(u - 0.04, 12)], '#0a0d20');
  if (etat.lumieres > 0) polygone(ctx, [P(u + 0.05, 0), P(u + 0.29, 0), P(u + 0.29, 9), P(u + 0.05, 9)], 'rgba(255,199,95,0.55)');
}

/**
 * Les fenêtres des deux façades visibles d'une boîte. Une fenêtre est allumée ou non selon un nombre tiré du nom du projet :
 * un projet plus actif en allume davantage, et quelques-unes vacillent doucement.
 * @param {CanvasRenderingContext2D} ctx
 * @param {{ x: number, y: number, w: number, d: number, z: number, h: number }} g
 * @param {Etat} etat
 * @param {string} cle
 * @param {Scene} s
 */
function fenetres(ctx, g, etat, cle, s) {
  const lignes = Math.floor((g.h - 8) / 12);
  if (lignes < 1) return;
  const part = PART_ALLUMEE[etat.lumieres] ?? 0;
  for (const face of /** @type {const} */ (['g', 'd'])) {
    const longueur = face === 'g' ? g.w : g.d;
    const colonnes = Math.floor(longueur / 0.52);
    if (colonnes < 1) continue;
    const pas = longueur / colonnes;
    const pasV = (g.h - 10) / lignes;
    for (let r = 0; r < lignes; r++) {
      for (let c = 0; c < colonnes; c++) {
        const n = hash(`${etat.slug}${cle}${face}${c},${r}`);
        let allumee = (n % 1000) / 1000 < part;
        // quelques fenêtres vacillent : elles s'éteignent un instant
        if (allumee && s.anim && n % 9 === 0 && Math.sin(s.t * 0.6 + (n % 628) / 100) > 0.93) allumee = false;
        const u0 = c * pas + pas * 0.25;
        const du = pas * 0.5;
        const v0 = 6 + r * pasV;
        const dv = 7;
        const P = face === 'g' ? (/** @type {number} */ u, /** @type {number} */ v) => proj(g.x + u, g.y + g.d, g.z + v) : (/** @type {number} */ u, /** @type {number} */ v) => proj(g.x + g.w, g.y + u, g.z + v);
        const coins = [P(u0, v0), P(u0 + du, v0), P(u0 + du, v0 + dv), P(u0, v0 + dv)];
        const teinte = n % 5 === 0 ? '#ffe39a' : AMBRE;
        polygone(ctx, coins, allumee ? teinte : etat.lumieres === 0 ? '#070a1a' : '#1a2252');
        if (allumee && s.zoom > 0.7) {
          const cx = (coins[0][0] + coins[2][0]) / 2;
          const cy = (coins[0][1] + coins[2][1]) / 2;
          ctx.fillStyle = rgba(teinte, 0.16);
          ctx.beginPath();
          ctx.arc(cx, cy, 8, 0, Math.PI * 2);
          ctx.fill();
        }
      }
    }
  }
}

/* ------------------------------------------------------------------ détails animés */

/**
 * @param {string} spec  « nom:argument,argument… »
 * @returns {[string, string[]]}
 */
function lire(spec) {
  const i = spec.indexOf(':');
  return i < 0 ? [spec, []] : [spec.slice(0, i), spec.slice(i + 1).split(',')];
}

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {string} spec
 * @param {Etat} etat
 * @param {Scene} s
 */
function detail(ctx, spec, etat, s) {
  const [nom, a] = lire(spec);
  const n = a.map(Number);
  const vif = etat.lumieres > 0;
  const accent = vif ? etat.accent : melange(etat.accent, '#101530', 0.7);
  const f = DETAILS[nom];
  if (f) f(ctx, a, n, { s, accent, vif, etat });
}

/**
 * @typedef {{ s: Scene, accent: string, vif: boolean, etat: Etat }} Rendu
 * @type {Record<string, (ctx: CanvasRenderingContext2D, a: string[], n: number[], r: Rendu) => void>}
 */
const DETAILS = {
  // de la fumée qui monte d'une cheminée : sans état, calculée à partir de l'heure
  fumee(ctx, a, n, { s, vif }) {
    if (!vif) return;
    const [x, y] = proj(s.ox + n[0], s.oy + n[1], n[2]);
    for (let i = 0; i < 5; i++) {
      const phase = s.anim ? (s.t * 0.35 + i / 5) % 1 : i / 5;
      ctx.fillStyle = `rgba(190,200,230,${(1 - phase) * 0.38})`;
      ctx.beginPath();
      ctx.arc(x + Math.sin(phase * 5 + i) * 4 + phase * 10, y - phase * 40, 3 + phase * 7, 0, Math.PI * 2);
      ctx.fill();
    }
  },

  // un engrenage qui tourne, posé sur une façade
  engrenage(ctx, a, n, { s, accent, vif }) {
    const droite = a[0] === 'droite';
    const [x, y] = droite ? proj(s.ox + 2.8, s.oy + n[1], n[2]) : proj(s.ox + n[1], s.oy + 2.7, n[2]);
    ctx.save();
    ctx.translate(x, y);
    ctx.transform(1, droite ? -0.5 : 0.5, 0, 1, 0, 0);
    roue(ctx, 6, accent, s.anim && vif ? s.t * 0.8 : 0);
    ctx.restore();
  },
  'engrenage-haut'(ctx, a, n, { s, accent, vif }) {
    const [x, y] = proj(s.ox + n[0], s.oy + n[1], n[2]);
    ctx.save();
    ctx.translate(x, y);
    ctx.scale(1, 0.5);
    roue(ctx, 15, accent, s.anim && vif ? -s.t * 0.6 : 0);
    ctx.restore();
  },

  // la France en toit : un hexagone quadrillé, posé à plat sur le dessus
  hexagone(ctx, a, n, { s, accent }) {
    const pts = [[0, -1], [0.55, -0.7], [0.95, -0.1], [0.6, 0.55], [0.15, 0.95], [-0.4, 0.75], [-0.85, 0.2], [-0.8, -0.5], [-0.3, -0.95]];
    /** @type {[number, number][]} */
    const poly = pts.map(([u, v]) => proj(s.ox + n[0] + u * 0.85, s.oy + n[1] + v * 0.85, n[2] + 0.5));
    polygone(ctx, poly, rgba(accent, 0.85), 'rgba(5,8,28,0.7)');
    ctx.strokeStyle = 'rgba(5,8,28,0.45)';
    ctx.lineWidth = 1;
    for (const [u0, v0, u1, v1] of [[-0.6, 0, 0.7, 0.1], [-0.4, -0.5, 0.5, -0.4], [0, -0.9, 0.05, 0.9], [-0.3, 0.4, 0.5, 0.5]]) {
      const p0 = proj(s.ox + n[0] + u0, s.oy + n[1] + v0, n[2] + 0.6);
      const p1 = proj(s.ox + n[0] + u1, s.oy + n[1] + v1, n[2] + 0.6);
      ctx.beginPath();
      ctx.moveTo(p0[0], p0[1]);
      ctx.lineTo(p1[0], p1[1]);
      ctx.stroke();
    }
  },

  // une épingle de carte qui flotte
  epingle(ctx, a, n, { s, accent }) {
    const bob = s.anim ? Math.sin(s.t * 2) * 3 : 0;
    const [x, y] = proj(s.ox + n[0], s.oy + n[1], n[2]);
    ctx.fillStyle = 'rgba(0,0,0,0.25)';
    ctx.beginPath();
    ctx.ellipse(x, y + 18, 8 - bob * 0.3, 3, 0, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = accent;
    ctx.beginPath();
    ctx.moveTo(x, y + 14 + bob);
    ctx.bezierCurveTo(x - 12, y - 2 + bob, x - 10, y - 16 + bob, x, y - 16 + bob);
    ctx.bezierCurveTo(x + 10, y - 16 + bob, x + 12, y - 2 + bob, x, y + 14 + bob);
    ctx.fill();
    ctx.fillStyle = '#10152e';
    ctx.beginPath();
    ctx.arc(x, y - 6 + bob, 3.5, 0, Math.PI * 2);
    ctx.fill();
  },

  // une antenne : des ondes concentriques qui rayonnent
  antenne(ctx, a, n, { s, accent, vif }) {
    const [x, y] = proj(s.ox + n[0], s.oy + n[1], n[2]);
    ctx.strokeStyle = '#9aa6d6';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(x, y);
    ctx.lineTo(x, y - 34);
    ctx.stroke();
    ctx.fillStyle = vif ? accent : '#394066';
    ctx.beginPath();
    ctx.arc(x, y - 36, 3.5, 0, Math.PI * 2);
    ctx.fill();
    if (!vif) return;
    for (let i = 0; i < 3; i++) {
      const phase = s.anim ? (s.t * 0.7 + i / 3) % 1 : i / 3;
      ctx.strokeStyle = rgba(accent, (1 - phase) * 0.7);
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.ellipse(x, y - 36, 6 + phase * 30, 3 + phase * 15, 0, 0, Math.PI * 2);
      ctx.stroke();
    }
  },

  // un cadenas sur une façade (le chiffrement)
  cadenas(ctx, a, n, { s, accent }) {
    const gauche = a[0] === 'gauche';
    const [x, y] = gauche ? proj(s.ox + n[1], s.oy + 2.7, n[2]) : proj(s.ox + 2.7, s.oy + n[1], n[2]);
    ctx.save();
    ctx.translate(x, y);
    ctx.transform(1, gauche ? 0.5 : -0.5, 0, 1, 0, 0);
    ctx.strokeStyle = accent;
    ctx.lineWidth = 2.5;
    ctx.beginPath();
    ctx.arc(0, -10, 5, Math.PI, 0);
    ctx.stroke();
    ctx.fillStyle = accent;
    ctx.fillRect(-7, -10, 14, 11);
    ctx.fillStyle = '#10152e';
    ctx.fillRect(-1.2, -6, 2.4, 5);
    ctx.restore();
  },

  // un bouclier sur une façade (le DNS protégé)
  bouclier(ctx, a, n, { s, accent }) {
    const [x, y] = proj(s.ox + n[1], s.oy + 2.6, n[2] + 12);
    ctx.save();
    ctx.translate(x, y);
    ctx.transform(1, 0.5, 0, 1, 0, 0);
    ctx.fillStyle = rgba(accent, 0.95);
    ctx.beginPath();
    ctx.moveTo(-9, -10);
    ctx.lineTo(9, -10);
    ctx.lineTo(9, 2);
    ctx.quadraticCurveTo(9, 12, 0, 15);
    ctx.quadraticCurveTo(-9, 12, -9, 2);
    ctx.closePath();
    ctx.fill();
    ctx.restore();
  },

  // un globe qui tourne : des méridiens dont l'ouverture varie
  globe(ctx, a, n, { s, accent, vif }) {
    const [x, y] = proj(s.ox + n[0], s.oy + n[1], n[2]);
    const R = 11;
    const g = ctx.createRadialGradient(x - 4, y - 5, 1, x, y, R);
    g.addColorStop(0, nuance(accent, 0.4));
    g.addColorStop(1, melange(accent, '#10152e', 0.55));
    ctx.fillStyle = g;
    ctx.beginPath();
    ctx.arc(x, y - R, R, 0, Math.PI * 2);
    ctx.fill();
    ctx.strokeStyle = 'rgba(255,255,255,0.45)';
    ctx.lineWidth = 1;
    const tour = s.anim && vif ? s.t * 0.9 : 0;
    for (let k = 0; k < 3; k++) {
      const rx = Math.abs(Math.cos(tour + (k * Math.PI) / 3)) * R;
      ctx.beginPath();
      ctx.ellipse(x, y - R, Math.max(rx, 0.5), R, 0, 0, Math.PI * 2);
      ctx.stroke();
    }
    ctx.beginPath();
    ctx.ellipse(x, y - R, R, R * 0.3, 0, 0, Math.PI * 2);
    ctx.stroke();
  },

  // une croix lumineuse qui bat doucement
  croix(ctx, a, n, { s, accent, vif }) {
    const [x, y] = proj(s.ox + n[0], s.oy + n[1], n[2]);
    const pouls = s.anim && vif ? 0.55 + 0.45 * Math.sin(s.t * 2.2) : 0.7;
    if (vif) {
      const halo = ctx.createRadialGradient(x, y - 14, 2, x, y - 14, 34);
      halo.addColorStop(0, rgba(accent, 0.45 * pouls));
      halo.addColorStop(1, rgba(accent, 0));
      ctx.fillStyle = halo;
      ctx.beginPath();
      ctx.arc(x, y - 14, 34, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.fillStyle = vif ? nuance(accent, 0.15 + 0.2 * pouls) : melange(accent, '#10152e', 0.6);
    ctx.fillRect(x - 5, y - 30, 10, 32);
    ctx.fillRect(x - 15, y - 20, 30, 10);
  },

  // un petit fanion planté au sommet d'un escalier
  'fanion-haut'(ctx, a, n, { s, accent }) {
    const [x, y] = proj(s.ox + n[0], s.oy + n[1], n[2]);
    ctx.strokeStyle = '#cfd6ff';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(x, y);
    ctx.lineTo(x, y - 26);
    ctx.stroke();
    const v = s.anim ? Math.sin(s.t * 3) * 2 : 0;
    ctx.fillStyle = accent;
    ctx.beginPath();
    ctx.moveTo(x, y - 26);
    ctx.lineTo(x + 16 + v, y - 22);
    ctx.lineTo(x, y - 17);
    ctx.fill();
  },

  // le faisceau d'un phare : il balaie la ville
  faisceau(ctx, a, n, { s, vif }) {
    if (!vif) return;
    const ang = s.anim ? s.t * 0.7 : 0.6;
    const [x, y] = proj(s.ox + n[0], s.oy + n[1], n[2]);
    const L = 5.5;
    const p1 = proj(s.ox + n[0] + Math.cos(ang - 0.16) * L, s.oy + n[1] + Math.sin(ang - 0.16) * L, n[2] - 80);
    const p2 = proj(s.ox + n[0] + Math.cos(ang + 0.16) * L, s.oy + n[1] + Math.sin(ang + 0.16) * L, n[2] - 80);
    const g = ctx.createLinearGradient(x, y, (p1[0] + p2[0]) / 2, (p1[1] + p2[1]) / 2);
    g.addColorStop(0, 'rgba(255,225,150,0.5)');
    g.addColorStop(1, 'rgba(255,225,150,0)');
    ctx.fillStyle = g;
    ctx.beginPath();
    ctx.moveTo(x, y);
    ctx.lineTo(p1[0], p1[1]);
    ctx.lineTo(p2[0], p2[1]);
    ctx.closePath();
    ctx.fill();
  },

  // un projecteur de tour de guet : un faisceau étroit qui tourne
  projecteur(ctx, a, n, { s, accent, vif }) {
    if (!vif) return;
    const ang = s.anim ? s.t * 1.1 + 1 : 1;
    const [x, y] = proj(s.ox + n[0], s.oy + n[1], n[2]);
    const L = 3.6;
    const p1 = proj(s.ox + n[0] + Math.cos(ang - 0.1) * L, s.oy + n[1] + Math.sin(ang - 0.1) * L, 0);
    const p2 = proj(s.ox + n[0] + Math.cos(ang + 0.1) * L, s.oy + n[1] + Math.sin(ang + 0.1) * L, 0);
    const g = ctx.createLinearGradient(x, y, (p1[0] + p2[0]) / 2, (p1[1] + p2[1]) / 2);
    g.addColorStop(0, rgba(accent, 0.55));
    g.addColorStop(1, rgba(accent, 0));
    ctx.fillStyle = g;
    ctx.beginPath();
    ctx.moveTo(x, y);
    ctx.lineTo(p1[0], p1[1]);
    ctx.lineTo(p2[0], p2[1]);
    ctx.closePath();
    ctx.fill();
  },

  // des flèches qui défilent d'un bloc à l'autre : un format qui devient un autre
  convoyeur(ctx, a, n, { s, accent, vif }) {
    const [x0, y0, x1, y1] = n;
    for (let i = 0; i < 4; i++) {
      const p = s.anim && vif ? (s.t * 0.45 + i / 4) % 1 : i / 4;
      const [x, y] = proj(s.ox + x0 + (x1 - x0) * p, s.oy + y0 + (y1 - y0) * p, 1);
      ctx.fillStyle = rgba(accent, vif ? 0.55 + 0.4 * Math.sin(p * Math.PI) : 0.25);
      ctx.beginPath();
      ctx.moveTo(x + 9, y);
      ctx.lineTo(x - 4, y - 5);
      ctx.lineTo(x - 4, y + 5);
      ctx.closePath();
      ctx.fill();
    }
  },

  // de l'eau autour d'une tour : des reflets qui ondulent
  eau(ctx, a, n, { s, accent }) {
    const [x, y, w, d] = n;
    /** @type {[number, number][]} */
    const rive = [proj(s.ox + x, s.oy + y, 5), proj(s.ox + x + w, s.oy + y, 5), proj(s.ox + x + w, s.oy + y + d, 5), proj(s.ox + x, s.oy + y + d, 5)];
    polygone(ctx, rive, 'rgba(60,90,200,0.38)', rgba(accent, 0.5));
    ctx.strokeStyle = 'rgba(190,210,255,0.5)';
    ctx.lineWidth = 1;
    for (let k = 0; k < 4; k++) {
      const gy = y + 0.4 + k * (d / 4.2);
      ctx.beginPath();
      for (let i = 0; i <= 12; i++) {
        const gx = x + 0.2 + (i / 12) * (w - 0.4);
        const dz = s.anim ? Math.sin(s.t * 1.6 + i * 0.8 + k) * 1.5 : 0;
        const [px, py] = proj(s.ox + gx, s.oy + gy, 5 + dz);
        if (i) ctx.lineTo(px, py);
        else ctx.moveTo(px, py);
      }
      ctx.stroke();
    }
  },

  // une pièce qui tourne (le suivi des dépenses)
  piece(ctx, a, n, { s, accent, vif }) {
    const [x, y] = proj(s.ox + n[0], s.oy + n[1], n[2]);
    const largeur = s.anim && vif ? Math.abs(Math.cos(s.t * 2.2)) : 0.8;
    ctx.fillStyle = vif ? '#ffd26a' : '#4a4630';
    ctx.strokeStyle = vif ? '#b8862a' : '#2a2a20';
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.ellipse(x, y - 10, Math.max(largeur * 8, 1), 8, 0, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
    if (largeur > 0.4) {
      ctx.fillStyle = vif ? '#b8862a' : '#2a2a20';
      ctx.fillRect(x - 1, y - 14, 2, 8);
    }
    void accent;
  },

  // une rivière de fichiers : des petits carrés qui descendent le courant
  riviere(ctx, a, n, { s, accent, vif }) {
    const [x, y, w, d] = n;
    /** @type {[number, number][]} */
    const lit = [proj(s.ox + x, s.oy + y, 1), proj(s.ox + x + w, s.oy + y, 1), proj(s.ox + x + w, s.oy + y + d, 1), proj(s.ox + x, s.oy + y + d, 1)];
    polygone(ctx, lit, 'rgba(70,130,220,0.32)', rgba(accent, 0.45));
    for (let i = 0; i < 6; i++) {
      const p = s.anim && vif ? (s.t * 0.22 + i / 6) % 1 : i / 6;
      const [px, py] = proj(s.ox + x + 0.1 + p * (w - 0.2), s.oy + y + d / 2 + Math.sin(i * 2 + (s.anim ? s.t : 0)) * 0.12, 5);
      ctx.fillStyle = i % 2 ? '#ffe9a8' : accent;
      ctx.fillRect(px - 3.5, py - 3.5, 7, 7);
    }
  },

  // des baies rondes de serveur : trois petits voyants (la framboise)
  baies(ctx, a, n, { s, accent, vif }) {
    const [x, y] = proj(s.ox + n[0], s.oy + n[1], n[2]);
    for (let i = -1; i <= 1; i++) {
      ctx.fillStyle = vif && (!s.anim || Math.sin(s.t * 3 + i * 2) > -0.3) ? accent : '#2a3260';
      ctx.beginPath();
      ctx.arc(x + i * 8, y + i * 4, 2.6, 0, Math.PI * 2);
      ctx.fill();
    }
  },

  // des colonnes devant une façade
  colonnes(ctx, a, n, { s }) {
    const [x0, y, base, largeur, z0, h] = n;
    const nombre = 5;
    for (let i = 0; i < nombre; i++) {
      const x = x0 + (i * largeur) / (nombre - 1);
      boite(ctx, { x: s.ox + x, y: s.oy + y - 0.1, w: 0.16, d: 0.16, z: z0, h }, couleursDe('#c9d0ee'));
    }
    void base;
  },

  // une balance, la justice : un fléau qui penche doucement
  balance(ctx, a, n, { s, accent }) {
    const [x, y] = proj(s.ox + n[0], s.oy + n[1], n[2]);
    const pente = s.anim ? Math.sin(s.t * 1.2) * 3 : 0;
    ctx.strokeStyle = accent;
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(x, y);
    ctx.lineTo(x, y - 14);
    ctx.moveTo(x - 13, y - 14 + pente);
    ctx.lineTo(x + 13, y - 14 - pente);
    ctx.stroke();
    for (const sens of [-1, 1]) {
      const px = x + sens * 13;
      const py = y - 14 + sens * pente;
      ctx.beginPath();
      ctx.moveTo(px, py);
      ctx.lineTo(px - 5, py + 8);
      ctx.lineTo(px + 5, py + 8);
      ctx.closePath();
      ctx.stroke();
    }
  },

  // un échafaudage : des poteaux et des planchers autour d'un bâtiment en chantier
  echafaudage(ctx, a, n, { s }) {
    const [x, y, w, d, h] = n;
    ctx.strokeStyle = 'rgba(207,214,255,0.85)';
    ctx.lineWidth = 1.5;
    const coins = [[x, y + d], [x + w, y + d], [x + w, y]];
    for (const [cx, cy] of coins) {
      const [bx, by] = proj(s.ox + cx, s.oy + cy, 0);
      ctx.beginPath();
      ctx.moveTo(bx, by);
      ctx.lineTo(bx, by - h);
      ctx.stroke();
    }
    for (const z of [18, 36, h]) {
      ctx.beginPath();
      [[x, y + d], [x + w, y + d], [x + w, y]].forEach(([cx, cy], i) => {
        const [px, py] = proj(s.ox + cx, s.oy + cy, z);
        if (i) ctx.lineTo(px, py);
        else ctx.moveTo(px, py);
      });
      ctx.stroke();
    }
    ctx.beginPath();
    const [a0x, a0y] = proj(s.ox + x, s.oy + y + d, 0);
    const [a1x, a1y] = proj(s.ox + x + w, s.oy + y + d, 36);
    ctx.moveTo(a0x, a0y);
    ctx.lineTo(a1x, a1y);
    ctx.stroke();
  },

  grue(ctx, a, n, { s }) {
    grue(ctx, s.ox + n[0], s.oy + n[1], n[2], s);
  },

  // un groupe de climatisation sur un toit
  clim(ctx, a, n, { s }) {
    boite(ctx, { x: s.ox + n[0], y: s.oy + n[1], w: 0.38, d: 0.3, z: n[2], h: 6 }, couleursDe('#8d98c4'));
    boite(ctx, { x: s.ox + n[0] + 0.5, y: s.oy + n[1] + 0.1, w: 0.26, d: 0.26, z: n[2], h: 4 }, couleursDe('#7a85b3'));
  },
};

/**
 * Une roue dentée centrée sur l'origine.
 * @param {CanvasRenderingContext2D} ctx
 * @param {number} r
 * @param {string} couleur
 * @param {number} angle
 */
function roue(ctx, r, couleur, angle) {
  ctx.rotate(angle);
  ctx.fillStyle = couleur;
  ctx.beginPath();
  const dents = 8;
  for (let i = 0; i < dents * 2; i++) {
    const a = (i * Math.PI) / dents;
    const rr = i % 2 ? r * 0.78 : r;
    ctx.lineTo(Math.cos(a) * rr, Math.sin(a) * rr);
  }
  ctx.closePath();
  ctx.fill();
  ctx.fillStyle = '#10152e';
  ctx.beginPath();
  ctx.arc(0, 0, r * 0.32, 0, Math.PI * 2);
  ctx.fill();
}

/**
 * Une grue de chantier : un mât, une flèche qui oscille et un crochet qui se balance. Aussi dessinée au-dessus
 * de tout projet en chantier.
 * @param {CanvasRenderingContext2D} ctx
 * @param {number} gx
 * @param {number} gy
 * @param {number} z  la hauteur du pied de la grue
 * @param {Scene} s
 */
export function grue(ctx, gx, gy, z, s) {
  const [x, y] = proj(gx, gy, z);
  const haut = 38;
  const ang = s.anim ? Math.sin(s.t * 0.5) * 0.5 : 0.2;
  ctx.strokeStyle = '#f4b860';
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(x, y);
  ctx.lineTo(x, y - haut);
  ctx.stroke();
  const bx = x + Math.cos(ang) * 30;
  const by = y - haut + Math.sin(ang) * 9;
  ctx.beginPath();
  ctx.moveTo(x - 10, y - haut + 3);
  ctx.lineTo(bx, by);
  ctx.stroke();
  const pend = s.anim ? Math.sin(s.t * 1.4) * 3 : 0;
  ctx.strokeStyle = '#cfd6ff';
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(bx - 4, by + 1);
  ctx.lineTo(bx - 4 + pend, by + 17);
  ctx.stroke();
  ctx.fillStyle = '#f4b860';
  ctx.fillRect(bx - 7 + pend, by + 17, 6, 4);
}

