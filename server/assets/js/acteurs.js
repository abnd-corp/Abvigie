// @ts-check
/**
 * Les personnages : les personnes et les robots qui travaillent dans l'organisation, vus depuis leurs issues et pull
 * requests ouvertes (des identifiants GitHub publics, rien d'autre). Ils se promènent de carrefour en carrefour.
 */
import { boite, couleursDe, hash, nuance, proj } from './iso.js';
import { LOT } from './monde.js';

/** @typedef {import('./donnees.js').Donnees} Donnees */
/** @typedef {import('./batiments.js').Scene} Scene */

const PEAUX = ['#f2c9a0', '#e0ac82', '#c68a62', '#8d5a3b', '#f6d7b8'];
const HABITS = ['#5fcfb4', '#9a8cff', '#f08fc0', '#6aa7ff', '#f4b860', '#ff7d6e', '#7ed39a', '#55c7e0'];
const ROBOTS = ['#5fcfb4', '#9a8cff', '#f4b860', '#6aa7ff'];

/**
 * @typedef {{
 *   login: string, robot: boolean, couleur: string, peau: string, graine: number,
 *   i: number, j: number, ni: number, nj: number, p: number, vitesse: number, decal: number,
 *   gx: number, gy: number
 * }} Marcheur
 */

/**
 * @param {Donnees} donnees
 * @param {{ decalage: number, W: number, H: number, marge: number }} monde
 */
export function creerActeurs(donnees, monde) {
  const I = Math.floor((monde.W - monde.marge * 2) / LOT);
  const J = Math.floor((monde.H - monde.marge * 2) / LOT);
  /** @param {number} i @param {number} j @returns {[number, number]} */
  const centre = (i, j) => [i * LOT + monde.decalage + 0.5, j * LOT + monde.decalage + 0.5];

  /** @type {Marcheur[]} */
  const marcheurs = donnees.acteurs.map((a, k) => {
    const h = hash(a.login);
    const i = h % (I + 1);
    const j = (h >>> 8) % (J + 1);
    const m = {
      login: a.login,
      robot: a.robot,
      couleur: a.robot ? ROBOTS[h % ROBOTS.length] : HABITS[h % HABITS.length],
      peau: PEAUX[(h >>> 4) % PEAUX.length],
      graine: k * 1.7 + (h % 100) / 10,
      i,
      j,
      ni: i,
      nj: j,
      p: 1,
      vitesse: a.robot ? 0.55 : 0.85,
      decal: ((h >>> 12) % 7) / 20 - 0.15,
      gx: 0,
      gy: 0,
    };
    [m.gx, m.gy] = centre(i, j);
    return m;
  });

  /** @param {Marcheur} m */
  function choisir(m) {
    /** @type {[number, number][]} */
    const voisins = [];
    for (const [di, dj] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
      const ni = m.i + di;
      const nj = m.j + dj;
      if (ni < 0 || nj < 0 || ni > I || nj > J) continue;
      voisins.push([ni, nj]);
    }
    const [ni, nj] = voisins[Math.floor(Math.random() * voisins.length)];
    m.ni = ni;
    m.nj = nj;
    m.p = 0;
  }

  /** @param {number} dt */
  function maj(dt) {
    for (const m of marcheurs) {
      if (m.p >= 1) {
        m.i = m.ni;
        m.j = m.nj;
        choisir(m);
      }
      m.p = Math.min(1, m.p + (dt * m.vitesse) / LOT);
      const [ax, ay] = centre(m.i, m.j);
      const [bx, by] = centre(m.ni, m.nj);
      // un léger décalage latéral évite que deux personnages se superposent
      const sens = m.ni !== m.i ? [0, m.decal] : [m.decal, 0];
      m.gx = ax + (bx - ax) * m.p + sens[0];
      m.gy = ay + (by - ay) * m.p + sens[1];
    }
  }

  /** Positions pour les étiquettes et le survol, en pixels du monde. */
  function positions() {
    return marcheurs.map((m) => {
      const [x, y] = proj(m.gx, m.gy);
      return { x, y: y - (m.robot ? 18 : 24), login: m.login, robot: m.robot };
    });
  }

  /**
   * Les personnages sous un point du monde.
   * @param {number} wx
   * @param {number} wy
   */
  function trouver(wx, wy) {
    return positions().find((p) => Math.hypot(p.x - wx, p.y + 8 - wy) < 11) ?? null;
  }

  /**
   * @param {Scene} s
   * @returns {{ profondeur: number, dessiner: (ctx: CanvasRenderingContext2D) => void }[]}
   */
  function objets(s) {
    return marcheurs.map((m) => ({ profondeur: m.gx + m.gy, dessiner: (ctx) => (m.robot ? dessinerRobot(ctx, m, s) : dessinerPersonne(ctx, m, s)) }));
  }

  return { marcheurs, maj, positions, trouver, objets };
}

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {Marcheur} m
 * @param {Scene} s
 */
function dessinerPersonne(ctx, m, s) {
  const [x, y] = proj(m.gx, m.gy);
  const pas = s.anim ? Math.sin(s.t * 9 + m.graine) : 0;
  const saut = s.anim ? Math.abs(pas) * 1.4 : 0;
  ctx.fillStyle = 'rgba(2,4,16,0.35)';
  ctx.beginPath();
  ctx.ellipse(x + 2, y + 1, 6, 2.6, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.strokeStyle = '#1a2040';
  ctx.lineWidth = 2.4;
  ctx.lineCap = 'round';
  ctx.beginPath();
  ctx.moveTo(x - 1.6, y - 7 - saut);
  ctx.lineTo(x - 1.6 - pas * 2, y);
  ctx.moveTo(x + 1.6, y - 7 - saut);
  ctx.lineTo(x + 1.6 + pas * 2, y);
  ctx.stroke();
  ctx.fillStyle = m.couleur;
  ctx.beginPath();
  ctx.roundRect(x - 4.5, y - 17 - saut, 9, 11, 3);
  ctx.fill();
  ctx.fillStyle = nuance(m.couleur, -0.25);
  ctx.fillRect(x - 4.5, y - 9 - saut, 9, 2);
  ctx.fillStyle = m.peau;
  ctx.beginPath();
  ctx.arc(x, y - 21 - saut, 4.2, 0, Math.PI * 2);
  ctx.fill();
  ctx.fillStyle = '#2a2230';
  ctx.beginPath();
  ctx.arc(x, y - 22.4 - saut, 4.2, Math.PI, 0);
  ctx.fill();
}

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {Marcheur} m
 * @param {Scene} s
 */
function dessinerRobot(ctx, m, s) {
  const [x, y] = proj(m.gx, m.gy);
  ctx.fillStyle = 'rgba(2,4,16,0.35)';
  ctx.beginPath();
  ctx.ellipse(x + 2, y + 1, 7, 3, 0, 0, Math.PI * 2);
  ctx.fill();
  boite(ctx, { x: m.gx - 0.17, y: m.gy - 0.17, w: 0.34, d: 0.34, z: 2, h: 9 }, couleursDe(m.couleur));
  const [ax, ay] = proj(m.gx, m.gy, 11);
  ctx.strokeStyle = '#9aa6d6';
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  ctx.moveTo(ax, ay);
  ctx.lineTo(ax, ay - 8);
  ctx.stroke();
  const clignote = !s.anim || Math.sin(s.t * 5 + m.graine) > 0;
  ctx.fillStyle = clignote ? '#ff7d6e' : '#5a2a30';
  ctx.beginPath();
  ctx.arc(ax, ay - 9, 2, 0, Math.PI * 2);
  ctx.fill();
}
