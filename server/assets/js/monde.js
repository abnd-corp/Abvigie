// @ts-check
/**
 * Le monde : la mise en page de la ville (un quartier par domaine, une parcelle par projet), le sol et ses routes,
 * les arbres et les lampadaires, et de quoi retrouver le bâtiment sous la souris.
 *
 * La carte entière est une plaque. Les parcelles font 3 × 3 tuiles de terrain, séparées par des routes d'une tuile :
 * une parcelle occupe donc 4 × 4 tuiles avec sa route. Les positions des routes sont des multiples de 4.
 */
import { dessiner, emprise, grue } from './batiments.js';
import { TH, TW, hash, nuance, polygone, proj, rgba, unproj } from './iso.js';

/** Tuiles par parcelle, route comprise. */
export const LOT = 4;

/** @typedef {import('./donnees.js').Donnees} Donnees */
/** @typedef {import('./donnees.js').Projet} Projet */
/** @typedef {{ id: string, titre: string, x: number, y: number, w: number, h: number }} Quartier */
/** @typedef {import('./batiments.js').Scene} Scene */

/**
 * @typedef {{
 *   projet: Projet,
 *   ox: number, oy: number,
 *   hull: [number, number][],
 *   ancre: [number, number],
 *   pied: [number, number],
 *   profondeur: number,
 *   haut: number
 * }} Batiment
 */

const COULEURS_QUARTIER = /** @type {Record<string, string>} */ ({
  web: '#2b3d82',
  systemes: '#1f5a78',
  outillage: '#5a4282',
  autres: '#343b68',
});

/** La couleur d'accent de chaque quartier : son liseré et son nom. */
const ACCENTS_QUARTIER = /** @type {Record<string, string>} */ ({
  web: '#86a2ff',
  systemes: '#5fd0e6',
  outillage: '#b896ff',
  autres: '#9aa6d6',
});

/**
 * Les rangées de parcelles d'un quartier : deux lignes pour une petite poignée de projets, davantage ensuite.
 * @param {number} n
 */
function disposition(n) {
  const lignes = n <= 2 ? 1 : n <= 8 ? 2 : Math.ceil(n / 4);
  return { lignes, colonnes: Math.ceil(n / lignes) };
}

/**
 * @param {Donnees} donnees
 */
export function creerMonde(donnees) {
  /** @type {Batiment[]} */
  const batiments = [];
  /** @type {{ id: string, titre: string, x: number, y: number, w: number, h: number }[]} */
  const quartiers = [];
  /** @type {{ cx: number, cy: number }[]} */
  const vides = [];

  // --- les blocs de quartiers, rangés de gauche à droite sur des étagères de 8 parcelles de large
  const LARGEUR_MAX = 8;
  let cx = 0;
  let cy = 0;
  let hautEtagere = 0;
  let largeurTotale = 0;
  for (const q of donnees.quartiers) {
    const projets = donnees.projets.filter((p) => p.quartier === q.id);
    if (!projets.length) continue;
    const { lignes, colonnes } = disposition(projets.length);
    if (cx > 0 && cx + colonnes > LARGEUR_MAX) {
      cx = 0;
      cy += hautEtagere + 1;
      hautEtagere = 0;
    }
    quartiers.push({ id: q.id, titre: q.titre, x: cx, y: cy, w: colonnes, h: lignes });
    projets.forEach((projet, i) => {
      const lx = cx + (i % colonnes);
      const ly = cy + Math.floor(i / colonnes);
      const ox = lx * LOT + 1;
      const oy = ly * LOT + 1;
      const e = emprise(projet.slug);
      /** @type {[number, number][]} */
      const coins = [];
      for (const z of [0, e.h]) {
        for (const [gx, gy] of [[e.x, e.y], [e.x + e.w, e.y], [e.x + e.w, e.y + e.d], [e.x, e.y + e.d]]) coins.push(proj(ox + gx, oy + gy, z));
      }
      batiments.push({
        projet,
        ox,
        oy,
        hull: enveloppe(coins),
        ancre: proj(ox + e.x + e.w / 2, oy + e.y + e.d / 2, e.h),
        pied: proj(ox + e.x + e.w, oy + e.y + e.d, 0),
        profondeur: ox + e.x + e.w + (oy + e.y + e.d),
        haut: e.h,
      });
    });
    for (let k = projets.length; k < lignes * colonnes; k++) vides.push({ cx: cx + (k % colonnes), cy: cy + Math.floor(k / colonnes) });
    cx += colonnes + 1;
    largeurTotale = Math.max(largeurTotale, cx - 1);
    hautEtagere = Math.max(hautEtagere, lignes);
  }
  const hauteurTotale = cy + hautEtagere;

  // l'emplacement du premier client : la charte demande de s'y préparer, il reste donc libre
  const client = { cx: Math.min(cx, LARGEUR_MAX - 1), cy };
  const lotsLargeur = Math.max(largeurTotale, client.cx + 1);
  /** La plaque : une marge d'une demi-parcelle autour de tout. */
  const marge = 2;
  const W = lotsLargeur * LOT + 1 + marge * 2;
  const H = hauteurTotale * LOT + 1 + marge * 2;
  const decalage = marge; // les parcelles commencent après la marge
  for (const b of batiments) {
    b.ox += decalage;
    b.oy += decalage;
    b.hull = b.hull.map(([x, y]) => /** @type {[number, number]} */ ([x + proj(decalage, decalage)[0], y + proj(decalage, decalage)[1]]));
    b.ancre = [b.ancre[0] + proj(decalage, decalage)[0], b.ancre[1] + proj(decalage, decalage)[1]];
    b.pied = [b.pied[0] + proj(decalage, decalage)[0], b.pied[1] + proj(decalage, decalage)[1]];
    b.profondeur += decalage * 2;
  }
  for (const q of quartiers) {
    q.x = q.x * LOT + decalage;
    q.y = q.y * LOT + decalage;
    q.w *= LOT;
    q.h *= LOT;
  }
  const clientTuile = { x: client.cx * LOT + decalage + 1, y: client.cy * LOT + decalage + 1 };

  // les parcelles libres deviennent des parcs ; la plus proche du centre accueille un étang
  const lotsLarg = lotsLargeur;
  const lotsHaut = hauteurTotale;
  const occupe = new Set([`${client.cx},${client.cy}`]);
  for (const q of quartiers) {
    // les quartiers sont maintenant en tuiles : on revient aux parcelles
    const lx = (q.x - decalage) / LOT;
    const ly = (q.y - decalage) / LOT;
    for (let ix = 0; ix < q.w / LOT; ix++) for (let iy = 0; iy < q.h / LOT; iy++) occupe.add(`${lx + ix},${ly + iy}`);
  }
  /** @type {{ cx: number, cy: number }[]} */
  const parcs = [];
  for (let ix = 0; ix < lotsLarg; ix++) for (let iy = 0; iy < lotsHaut; iy++) if (!occupe.has(`${ix},${iy}`)) parcs.push({ cx: ix, cy: iy });
  const milieu = { x: (lotsLarg - 1) / 2, y: (lotsHaut - 1) / 2 };
  parcs.sort((a, b) => Math.hypot(a.cx - milieu.x, a.cy - milieu.y) - Math.hypot(b.cx - milieu.x, b.cy - milieu.y));
  const etang = parcs.length ? parcs[0] : null;
  const estParc = new Set(parcs.map((p) => `${p.cx},${p.cy}`));

  // --- le décor : arbres dans les parcelles vides et les coins, lampadaires aux carrefours
  /** @type {{ gx: number, gy: number, taille: number, graine: number }[]} */
  const arbres = [];
  for (const p of parcs) {
    for (let k = 0; k < (p === etang ? 4 : 7); k++) {
      const h = hash(`parc${p.cx},${p.cy},${k}`);
      const bord = p === etang;
      arbres.push({
        gx: p.cx * LOT + decalage + (bord ? (k % 2 ? 0.7 : 2.5) : 0.6 + (h % 24) / 10),
        gy: p.cy * LOT + decalage + (bord ? 0.7 + (h % 18) / 10 : 0.6 + ((h >>> 5) % 24) / 10),
        taille: 0.8 + (h % 6) / 10,
        graine: h,
      });
    }
  }
  for (const v of vides) {
    for (let k = 0; k < 3; k++) {
      const h = hash(`vide${v.cx},${v.cy},${k}`);
      arbres.push({ gx: v.cx * LOT + decalage + 0.8 + (h % 20) / 10, gy: v.cy * LOT + decalage + 0.8 + ((h >>> 5) % 20) / 10, taille: 0.8 + (h % 5) / 10, graine: h });
    }
  }
  for (const b of batiments) {
    const h = hash(b.projet.slug + 'arbre');
    const e = emprise(b.projet.slug);
    if (e.x > 0.55) arbres.push({ gx: b.ox + 0.3, gy: b.oy + 2.7, taille: 0.7, graine: h });
    if (e.y + e.d < 2.45) arbres.push({ gx: b.ox + 2.7, gy: b.oy + 2.75, taille: 0.65, graine: h >>> 4 });
  }
  /** @type {{ gx: number, gy: number }[]} */
  const lampadaires = [];
  const noeuds = [];
  for (let i = 0; i * LOT <= W - marge * 2; i++) {
    for (let j = 0; j * LOT <= H - marge * 2; j++) {
      const gx = i * LOT + decalage + 0.5;
      const gy = j * LOT + decalage + 0.5;
      noeuds.push({ gx, gy, i, j });
      if ((i + j) % 2 === 0) lampadaires.push({ gx, gy });
    }
  }

  // le cadrage initial : la ville elle-même, avec de la place au-dessus pour les grandes tours et les fanions
  const ville = [proj(decalage, decalage), proj(W - marge, decalage), proj(W - marge, H - marge), proj(decalage, H - marge)];
  const cadre = {
    x0: Math.min(...ville.map((c) => c[0])) - 30,
    x1: Math.max(...ville.map((c) => c[0])) + 30,
    y0: Math.min(...ville.map((c) => c[1])) - 120,
    y1: Math.max(...ville.map((c) => c[1])) + 40,
  };

  // les bords de la plaque, en pixels du monde
  const coins = [proj(0, 0), proj(W, 0), proj(W, H), proj(0, H)];
  const boite = {
    x0: Math.min(...coins.map((c) => c[0])) - 40,
    x1: Math.max(...coins.map((c) => c[0])) + 40,
    y0: Math.min(...coins.map((c) => c[1])) - 130,
    y1: Math.max(...coins.map((c) => c[1])) + 60,
  };

  /* ---------------------------------------------------------------- le sol */

  /**
   * @param {CanvasRenderingContext2D} ctx
   * @param {Scene} s
   * @param {{ x0: number, y0: number, x1: number, y1: number }} vue  la zone visible, en pixels du monde
   */
  function dessinerSol(ctx, s, vue) {
    const EPAISSEUR = 22;
    // l'épaisseur de la plaque : deux flancs sombres sous les bords visibles
    polygone(ctx, [proj(0, H), proj(W, H), proj(W, H, -EPAISSEUR), proj(0, H, -EPAISSEUR)], '#0f1432');
    polygone(ctx, [proj(W, 0), proj(W, H), proj(W, H, -EPAISSEUR), proj(W, 0, -EPAISSEUR)], '#0a0e26');
    ctx.fillStyle = 'rgba(2,4,14,0.45)';
    ctx.beginPath();
    [proj(0, H, -EPAISSEUR), proj(W, H, -EPAISSEUR), proj(W, 0, -EPAISSEUR), proj(W + 3, 3, -EPAISSEUR - 22), proj(W + 3, H + 3, -EPAISSEUR - 22), proj(3, H + 3, -EPAISSEUR - 22)].forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)));
    ctx.closePath();
    ctx.fill();

    // toutes les tuiles d'une même couleur dans un seul tracé : pas de fines coutures entre elles
    /** @type {Map<string, [number, number][][]>} */
    const lots = new Map();
    /** @param {string} couleur @param {[number, number][]} poly */
    const ranger = (couleur, poly) => {
      const liste = lots.get(couleur);
      if (liste) liste.push(poly);
      else lots.set(couleur, [poly]);
    };
    const ROUTE = '#202a5a';
    for (let gx = 0; gx < W; gx++) {
      for (let gy = 0; gy < H; gy++) {
        const [x, y] = proj(gx, gy);
        if (x + TW / 2 < vue.x0 || x - TW / 2 > vue.x1 || y + TH < vue.y0 || y > vue.y1) continue;
        const lx = (gx - decalage) / LOT;
        const ly = (gy - decalage) / LOT;
        const route = gx < decalage || gy < decalage || (gx - decalage) % LOT === 0 || (gy - decalage) % LOT === 0 || gx >= W - marge || gy >= H - marge;
        let couleur = ROUTE;
        if (!route) {
          const q = quartiers.find((qq) => gx >= qq.x && gx < qq.x + qq.w && gy >= qq.y && gy < qq.y + qq.h);
          const lotX = Math.floor((gx - decalage) / LOT);
          const lotY = Math.floor((gy - decalage) / LOT);
          couleur = q ? COULEURS_QUARTIER[q.id] ?? COULEURS_QUARTIER.autres : estParc.has(`${lotX},${lotY}`) ? '#215a50' : '#262f63';
          if ((Math.floor(lx * LOT) + Math.floor(ly * LOT)) % 2 === 0) couleur = nuance(couleur, 0.045);
        }
        ranger(couleur, [proj(gx, gy), proj(gx + 1, gy), proj(gx + 1, gy + 1), proj(gx, gy + 1)]);
      }
    }
    for (const [couleur, polys] of lots) {
      ctx.beginPath();
      for (const poly of polys) poly.forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)));
      ctx.fillStyle = couleur;
      ctx.fill();
    }

    // le trait de la plaque et les pointillés au milieu des routes
    ctx.strokeStyle = 'rgba(110,130,230,0.35)';
    ctx.lineWidth = 2;
    ctx.beginPath();
    [proj(0, 0), proj(W, 0), proj(W, H), proj(0, H)].forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)));
    ctx.closePath();
    ctx.stroke();
    ctx.strokeStyle = 'rgba(170,185,255,0.22)';
    ctx.lineWidth = 1.5;
    ctx.setLineDash([10, 12]);
    ctx.beginPath();
    for (let i = 0; i * LOT + decalage <= W - marge; i++) {
      const gx = i * LOT + decalage + 0.5;
      const [ax, ay] = proj(gx, decalage + 0.5);
      const [bx, by] = proj(gx, H - marge - 0.5);
      ctx.moveTo(ax, ay);
      ctx.lineTo(bx, by);
    }
    for (let j = 0; j * LOT + decalage <= H - marge; j++) {
      const gy = j * LOT + decalage + 0.5;
      const [ax, ay] = proj(decalage + 0.5, gy);
      const [bx, by] = proj(W - marge - 0.5, gy);
      ctx.moveTo(ax, ay);
      ctx.lineTo(bx, by);
    }
    ctx.stroke();
    ctx.setLineDash([]);

    // une bande à la couleur de chaque quartier sur la rue de son bord avant ; son nom y est écrit par `dessinerNoms`
    for (const q of quartiers) {
      const vif = s.quartier === q.id;
      const couleur = accentQuartier(q.id);
      const x0 = q.x + 1;
      const y0 = q.y + q.h;
      const longueur = q.w - 1;
      polygone(ctx, [proj(x0, y0 + 0.08), proj(x0 + longueur, y0 + 0.08), proj(x0 + longueur, y0 + 0.92), proj(x0, y0 + 0.92)], rgba(couleur, vif ? 0.4 : 0.26), rgba(couleur, vif ? 0.95 : 0.6));
    }

    // l'étang du parc : de l'eau qui ondule doucement
    if (etang) {
      const ex = etang.cx * LOT + decalage + 1;
      const ey = etang.cy * LOT + decalage + 1;
      polygone(ctx, [proj(ex + 0.4, ey + 0.4), proj(ex + 2.6, ey + 0.4), proj(ex + 2.6, ey + 2.6), proj(ex + 0.4, ey + 2.6)], 'rgba(64,120,225,0.62)', 'rgba(150,190,255,0.6)');
      ctx.strokeStyle = 'rgba(210,228,255,0.55)';
      ctx.lineWidth = 1;
      for (let k = 0; k < 4; k++) {
        ctx.beginPath();
        for (let i = 0; i <= 10; i++) {
          const dz = s.anim ? Math.sin(s.t * 1.5 + i * 0.9 + k * 1.7) * 0.9 : 0;
          const [px, py] = proj(ex + 0.7 + (i / 10) * 1.6, ey + 0.8 + k * 0.5, dz);
          if (i) ctx.lineTo(px, py);
          else ctx.moveTo(px, py);
        }
        ctx.stroke();
      }
    }

    // l'emplacement du premier client
    const c = clientTuile;
    ctx.fillStyle = 'rgba(60,190,170,0.13)';
    ctx.beginPath();
    [proj(c.x, c.y), proj(c.x + 3, c.y), proj(c.x + 3, c.y + 3), proj(c.x, c.y + 3)].forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)));
    ctx.closePath();
    ctx.fill();
    ctx.setLineDash([8, 6]);
    ctx.strokeStyle = 'rgba(95,207,180,0.8)';
    ctx.lineWidth = 2;
    ctx.stroke();
    ctx.setLineDash([]);
    ctx.save();
    const [px, py] = proj(c.x + 0.3, c.y + 1.3);
    ctx.translate(px, py);
    ctx.transform(1, 0.5, -1, 0.5, 0, 0);
    ctx.font = '700 11px ui-monospace, SFMono-Regular, Menlo, monospace';
    ctx.fillStyle = 'rgba(95,207,180,0.95)';
    ctx.fillText('PREMIER CLIENT', 6, 0);
    ctx.font = '9px ui-monospace, SFMono-Regular, Menlo, monospace';
    ctx.fillStyle = 'rgba(95,207,180,0.7)';
    ctx.fillText('emplacement libre', 6, 14);
    ctx.restore();
  }

  /* ---------------------------------------------------------------- les objets, du plus loin au plus proche */

  /**
   * @param {CanvasRenderingContext2D} ctx
   * @param {{ gx: number, gy: number, taille: number, graine: number }} a
   * @param {Scene} s
   */
  function dessinerArbre(ctx, a, s) {
    const [x, y] = proj(a.gx, a.gy);
    const bal = s.anim ? Math.sin(s.t * 0.9 + a.graine) * 1.2 : 0;
    ctx.fillStyle = 'rgba(2,4,16,0.3)';
    ctx.beginPath();
    ctx.ellipse(x + 5, y + 2, 11 * a.taille, 4.5 * a.taille, 0, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = '#4b3a2e';
    ctx.fillRect(x - 1.5, y - 10 * a.taille, 3, 10 * a.taille);
    const verts = ['#2f7d62', '#35906f', '#2b6f63'];
    for (let i = 0; i < 3; i++) {
      ctx.fillStyle = verts[(a.graine + i) % 3];
      ctx.beginPath();
      ctx.arc(x + bal * (i + 1) * 0.3 + (i - 1) * 3, y - (14 + i * 4) * a.taille, (8 - i * 1.4) * a.taille, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  /**
   * @param {CanvasRenderingContext2D} ctx
   * @param {{ gx: number, gy: number }} l
   * @param {Scene} s
   */
  function dessinerLampadaire(ctx, l, s) {
    const [x, y] = proj(l.gx, l.gy);
    ctx.strokeStyle = '#7480b8';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(x, y);
    ctx.lineTo(x, y - 26);
    ctx.stroke();
    const halo = ctx.createRadialGradient(x, y - 27, 1, x, y - 27, 26);
    const scint = s.anim ? 0.85 + 0.15 * Math.sin(s.t * 2 + l.gx) : 1;
    halo.addColorStop(0, `rgba(255,214,130,${0.55 * scint})`);
    halo.addColorStop(1, 'rgba(255,214,130,0)');
    ctx.fillStyle = halo;
    ctx.beginPath();
    ctx.arc(x, y - 27, 26, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = '#ffe3a0';
    ctx.beginPath();
    ctx.arc(x, y - 27, 2.5, 0, Math.PI * 2);
    ctx.fill();
  }

  /**
   * Tout ce qui se dresse sur le sol (la liste n'est pas triée : l'appelant y ajoute d'autres objets, puis trie).
   * @param {Scene} s
   * @param {(ctx: CanvasRenderingContext2D, b: Batiment) => void} apres  appelé après chaque bâtiment (fanions, sélection)
   */
  function objets(s, apres) {
    /** @type {{ profondeur: number, dessiner: (ctx: CanvasRenderingContext2D) => void }[]} */
    const liste = [];
    for (const a of arbres) liste.push({ profondeur: a.gx + a.gy, dessiner: (ctx) => dessinerArbre(ctx, a, s) });
    for (const l of lampadaires) liste.push({ profondeur: l.gx + l.gy, dessiner: (ctx) => dessinerLampadaire(ctx, l, s) });
    for (const b of batiments) {
      liste.push({
        profondeur: b.profondeur,
        dessiner: (ctx) => {
          const p = b.projet;
          // la grue d'un chantier se dresse au sol, derrière le bâtiment, et dépasse un peu son toit
          if (p.chantier) grue(ctx, b.ox + 2.75, b.oy + 0.25, 0, s, Math.max(38, b.haut * 0.8));
          dessiner(ctx, { slug: p.slug, accent: p.accent, lumieres: p.lumieres, archive: p.archive }, { ...s, ox: b.ox, oy: b.oy });
          apres(ctx, b);
        },
      });
    }
    return liste;
  }

  /**
   * Le nom de chaque quartier, à plat sur sa bande de rue, comme « premier client ». Dessiné par-dessus les bâtiments,
   * pour que rien ne le cache.
   * @param {CanvasRenderingContext2D} ctx
   * @param {Scene} s
   */
  function dessinerNoms(ctx, s) {
    for (const q of quartiers) {
      const vif = s.quartier === q.id;
      const x0 = q.x + 1;
      const y0 = q.y + q.h;
      const longueur = q.w - 1;
      ctx.save();
      const [x, y] = proj(x0, y0);
      ctx.translate(x, y);
      ctx.transform(1, 0.5, -1, 0.5, 0, 0);
      const texte = q.titre.toUpperCase();
      const police = (/** @type {number} */ t) => `800 ${t}px ui-monospace, SFMono-Regular, Menlo, monospace`;
      let taille = 21;
      ctx.font = police(taille);
      if ('letterSpacing' in ctx) ctx.letterSpacing = '3px';
      const dispo = longueur * 32 - 20;
      const mesure = ctx.measureText(texte).width;
      if (mesure > dispo) {
        taille = Math.max(8, Math.floor((taille * dispo) / mesure));
        ctx.font = police(taille);
      }
      const largeur = Math.min(ctx.measureText(texte).width, dispo);
      ctx.textBaseline = 'middle';
      ctx.lineJoin = 'round';
      ctx.lineWidth = 4;
      ctx.strokeStyle = 'rgba(10,14,38,0.85)';
      const gauche = 10 + (dispo - largeur) / 2;
      ctx.strokeText(texte, gauche, 16);
      ctx.fillStyle = vif ? '#ffffff' : accentQuartier(q.id);
      ctx.fillText(texte, gauche, 16);
      ctx.restore();
    }
  }

  /** @param {string} id */
  const accentQuartier = (id) => ACCENTS_QUARTIER[id] ?? ACCENTS_QUARTIER.autres;

  /**
   * Le liseré d'un quartier, sur l'axe des routes qui le bordent.
   * @param {CanvasRenderingContext2D} ctx
   * @param {{ x: number, y: number, w: number, h: number, id: string }} q
   * @param {boolean} vif
   */
  function contourQuartier(ctx, q, vif) {
    const coins = [proj(q.x + 0.5, q.y + 0.5), proj(q.x + q.w + 0.5, q.y + 0.5), proj(q.x + q.w + 0.5, q.y + q.h + 0.95), proj(q.x + 0.5, q.y + q.h + 0.95)];
    ctx.beginPath();
    coins.forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)));
    ctx.closePath();
    if (vif) {
      ctx.fillStyle = rgba(accentQuartier(q.id), 0.12);
      ctx.fill();
    }
    ctx.strokeStyle = rgba(accentQuartier(q.id), vif ? 0.95 : 0.5);
    ctx.lineWidth = vif ? 3 : 2;
    ctx.stroke();
  }

  /**
   * Le quartier sous un point du monde, s'il y en a un.
   * @param {number} x
   * @param {number} y
   */
  function quartierSous(x, y) {
    const [gx, gy] = unproj(x, y);
    // le quartier comprend la rue de son bord avant, où est écrit son nom
    return quartiers.find((q) => gx >= q.x + 0.5 && gx < q.x + q.w + 0.5 && gy >= q.y + 0.5 && gy < q.y + q.h + 1) ?? null;
  }

  /**
   * Le bâtiment sous un point du monde, le plus proche de la caméra en cas de recouvrement.
   * @param {number} x
   * @param {number} y
   */
  function trouver(x, y) {
    /** @type {Batiment | null} */
    let trouve = null;
    for (const b of batiments) {
      if (dansPolygone(x, y, b.hull) && (!trouve || b.profondeur > trouve.profondeur)) trouve = b;
    }
    return trouve;
  }

  return { batiments, quartiers, W, H, decalage, marge, noeuds, boite, cadre, clientTuile, dessinerSol, dessinerNoms, objets, trouver, unproj, accentQuartier, contourQuartier, quartierSous };
}

/**
 * L'enveloppe convexe d'un nuage de points (chaîne monotone d'Andrew).
 * @param {[number, number][]} pts
 * @returns {[number, number][]}
 */
function enveloppe(pts) {
  const p = [...pts].sort((a, b) => a[0] - b[0] || a[1] - b[1]);
  /** @param {[number, number]} o @param {[number, number]} a @param {[number, number]} b */
  const croix = (o, a, b) => (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
  /** @type {[number, number][]} */
  const bas = [];
  for (const q of p) {
    while (bas.length >= 2 && croix(bas[bas.length - 2], bas[bas.length - 1], q) <= 0) bas.pop();
    bas.push(q);
  }
  /** @type {[number, number][]} */
  const haut = [];
  for (const q of [...p].reverse()) {
    while (haut.length >= 2 && croix(haut[haut.length - 2], haut[haut.length - 1], q) <= 0) haut.pop();
    haut.push(q);
  }
  bas.pop();
  haut.pop();
  return [...bas, ...haut];
}

/**
 * @param {number} x
 * @param {number} y
 * @param {[number, number][]} poly
 */
export function dansPolygone(x, y, poly) {
  let dedans = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const [xi, yi] = poly[i];
    const [xj, yj] = poly[j];
    if (yi > y !== yj > y && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi) dedans = !dedans;
  }
  return dedans;
}

