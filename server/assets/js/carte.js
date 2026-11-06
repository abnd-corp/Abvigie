// @ts-check
/**
 * La carte : le point d'entrée de la page d'accueil. Elle assemble le monde, les personnages et la caméra, puis dessine
 * la ville image par image sur un canvas. Tout le contenu de la page est déjà dans le HTML : ce script ajoute la carte
 * et son interactivité, rien de plus.
 */
import { creerActeurs } from './acteurs.js';
import { emprise } from './batiments.js';
import { Camera, brancher } from './camera.js';
import { charger } from './donnees.js';
import { brancherInterface } from './interface.js';
import { contourSol, hash, proj, rgba } from './iso.js';
import { creerMonde } from './monde.js';

/** @typedef {import('./monde.js').Batiment} Batiment */

const POLICE = 'ui-monospace, SFMono-Regular, Menlo, Consolas, monospace';
const COULEURS_GRAVITE = { alerte: '#ff7d6e', attention: '#f4b860', info: '#5fcfb4' };

async function demarrer() {
  const element = document.getElementById('carte');
  if (!(element instanceof HTMLCanvasElement)) return;
  const canevas = element;
  const ctx = canevas.getContext('2d');
  if (!ctx) return;

  const donnees = await charger();
  const monde = creerMonde(donnees);
  const acteurs = creerActeurs(donnees, monde);
  const camera = new Camera();
  const mouvementReduit = matchMedia('(prefers-reduced-motion: reduce)').matches;

  /** L'état de l'affichage : réglages, survol, sélection. */
  const etat = {
    noms: true,
    alertes: true,
    personnages: true,
    /** @type {Batiment | null} */ survol: null,
    /** @type {ReturnType<typeof acteurs.trouver>} */ survolActeur: null,
    /** @type {string | null} */ selection: null,
    /** @type {Set<string>} */ surligne: new Set(),
    curseur: { x: 0, y: 0 },
    /** Vrai tant que le doigt ou la souris agit sur la carte. */
    curseurActif: false,
    t: 0,
  };

  let dpr = 1;
  let enCours = false;
  let dernier = 0;
  const animer = () => !mouvementReduit && !document.hidden;

  function dimensionner() {
    const r = canevas.getBoundingClientRect();
    dpr = Math.min(window.devicePixelRatio || 1, 2);
    const premiere = camera.largeur <= 1;
    canevas.width = Math.max(1, Math.round(r.width * dpr));
    canevas.height = Math.max(1, Math.round(r.height * dpr));
    camera.dimensionner(r.width, r.height);
    camera.limites = monde.boite;
    // tant que la personne n'a rien touché, on recadre la ville à chaque changement de taille
    if (premiere || !camera.manuel) camera.ajuster(monde.cadre);
    invalider();
  }

  /* ---------------------------------------------------------------- le dessin */

  /** @param {CanvasRenderingContext2D} c */
  function fond(c) {
    const g = c.createLinearGradient(0, 0, 0, camera.hauteur);
    g.addColorStop(0, '#0d1230');
    g.addColorStop(1, '#171e48');
    c.fillStyle = g;
    c.fillRect(0, 0, camera.largeur, camera.hauteur);
    for (let i = 0; i < 70; i++) {
      const h = hash(`etoile${i}`);
      const x = (h % 1000) / 1000;
      const y = ((h >>> 10) % 1000) / 1000;
      const scint = animer() ? 0.5 + 0.5 * Math.sin(etat.t * 1.3 + i) : 0.8;
      c.fillStyle = `rgba(200,210,255,${0.12 + 0.28 * scint * ((h >>> 20) % 3 === 0 ? 1 : 0.5)})`;
      c.fillRect(x * camera.largeur, y * camera.hauteur, 1.5, 1.5);
    }
  }

  /**
   * Après un bâtiment : son fanion d'alerte et, s'il dort, des « z » qui montent.
   * @param {CanvasRenderingContext2D} c
   * @param {Batiment} b
   */
  function apres(c, b) {
    const p = b.projet;
    const e = emprise(p.slug);
    const [ancreX] = b.ancre;
    if (etat.alertes && p.gravite) {
      const [px, py] = proj(b.ox + e.x + e.w + 0.1, b.oy + e.y + e.d * 0.2, 0);
      const haut = py - 46;
      c.strokeStyle = '#dfe4ff';
      c.lineWidth = 2;
      c.beginPath();
      c.moveTo(px, py);
      c.lineTo(px, haut);
      c.stroke();
      const v = animer() ? Math.sin(etat.t * 3 + px) * 2.5 : 0;
      c.fillStyle = COULEURS_GRAVITE[p.gravite];
      c.beginPath();
      c.moveTo(px, haut);
      c.quadraticCurveTo(px + 11, haut - 3 + v, px + 22, haut + 2 + v * 0.5);
      c.lineTo(px + 22, haut + 11);
      c.quadraticCurveTo(px + 11, haut + 8 + v, px, haut + 13);
      c.closePath();
      c.fill();
      if (p.alertes > 1) {
        c.fillStyle = '#10152e';
        c.font = `700 9px ${POLICE}`;
        c.textAlign = 'left';
        c.textBaseline = 'middle';
        c.fillText(String(p.alertes), px + 5, haut + 6.5);
      }
    }
    if (p.lumieres === 0 && animer()) {
      for (let i = 0; i < 3; i++) {
        const ph = (etat.t * 0.3 + i / 3) % 1;
        c.fillStyle = `rgba(190,200,255,${(1 - ph) * 0.7})`;
        c.font = `700 ${9 + ph * 7}px ${POLICE}`;
        c.textAlign = 'center';
        c.fillText('z', ancreX + 12 + ph * 10 + i * 3, b.ancre[1] - 6 - ph * 26);
      }
    }
  }

  function dessiner() {
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, camera.largeur, camera.hauteur);
    fond(ctx);

    ctx.save();
    ctx.translate(camera.largeur / 2, camera.hauteur / 2);
    ctx.scale(camera.zoom, camera.zoom);
    ctx.translate(-camera.x, -camera.y);
    const scene = { ox: 0, oy: 0, t: etat.t, anim: animer(), zoom: camera.zoom };
    monde.dessinerSol(ctx, scene, camera.zoneVisible(80));

    // au sol : pointillés des chantiers, anneaux de sélection et de quête
    for (const b of monde.batiments) {
      const e = emprise(b.projet.slug);
      const sol = { x: b.ox + e.x - 0.15, y: b.oy + e.y - 0.15, w: e.w + 0.3, d: e.d + 0.3 };
      if (b.projet.chantier && etat.alertes) contourSol(ctx, sol, 'rgba(230,236,255,0.85)', 2, [8, 6]);
      const pouls = animer() ? 0.5 + 0.5 * Math.sin(etat.t * 4) : 1;
      if (etat.surligne.has(b.projet.slug)) contourSol(ctx, sol, `rgba(255,200,95,${0.5 + 0.5 * pouls})`, 3.5);
      if (etat.selection === b.projet.slug) contourSol(ctx, sol, 'rgba(120,255,214,0.95)', 3);
    }

    const liste = monde.objets(scene, apres);
    if (etat.personnages) liste.push(...acteurs.objets(scene));
    liste.sort((a, b) => a.profondeur - b.profondeur);
    for (const o of liste) o.dessiner(ctx);

    if (etat.survol) {
      const b = etat.survol;
      ctx.beginPath();
      b.hull.forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)));
      ctx.closePath();
      ctx.fillStyle = 'rgba(255,255,255,0.07)';
      ctx.fill();
      ctx.strokeStyle = 'rgba(255,255,255,0.75)';
      ctx.lineWidth = 2;
      ctx.stroke();
    }
    ctx.restore();

    etiquettes(ctx);
    infobulle(ctx);
  }

  /** @param {CanvasRenderingContext2D} c */
  function etiquettes(c) {
    c.textBaseline = 'middle';
    c.textAlign = 'center';
    if (etat.noms) {
      for (const b of monde.batiments) {
        const survole = etat.survol === b || etat.selection === b.projet.slug;
        if (camera.zoom < 0.34 && !survole) continue;
        const [x, y] = camera.versEcran(b.ancre[0], b.ancre[1] - 6);
        etiquette(c, b.projet.nom, x, y, b.projet.accent, survole);
      }
    }
    if (etat.personnages && (etat.noms || etat.survolActeur)) {
      for (const p of acteurs.positions()) {
        const survole = etat.survolActeur?.login === p.login;
        if (camera.zoom < 0.85 && !survole) continue;
        const [x, y] = camera.versEcran(p.x, p.y - 4);
        etiquette(c, p.login, x, y, p.robot ? '#9aa6d6' : '#7ed39a', survole, true);
      }
    }
  }

  /**
   * Un nom dans un cadre, comme sur une enseigne.
   * @param {CanvasRenderingContext2D} c
   * @param {string} texte
   * @param {number} x
   * @param {number} y
   * @param {string} couleur
   * @param {boolean} vif
   * @param {boolean} [petit]
   */
  function etiquette(c, texte, x, y, couleur, vif, petit = false) {
    const taille = petit ? 9 : 11;
    c.font = `700 ${taille}px ${POLICE}`;
    const l = c.measureText(texte).width + 10;
    const h = taille + 7;
    c.fillStyle = vif ? 'rgba(14,19,48,0.98)' : 'rgba(14,19,48,0.86)';
    c.strokeStyle = couleur;
    c.lineWidth = vif ? 2 : 1.2;
    c.beginPath();
    c.roundRect(x - l / 2, y - h / 2, l, h, 3);
    c.fill();
    c.stroke();
    c.fillStyle = vif ? '#ffffff' : '#e6e9fb';
    c.fillText(texte, x, y + 0.5);
  }

  /** @param {CanvasRenderingContext2D} c */
  function infobulle(c) {
    const b = etat.survol;
    if (!b || etat.survolActeur) return;
    const p = b.projet;
    const etatTexte = p.lumieres === 0 ? `endormi (${p.jours_inactif} j)` : `actif, ${p.jours_inactif === 0 ? "aujourd'hui" : `il y a ${p.jours_inactif} j`}`;
    const lignes = [p.nom, etatTexte, `${p.issues} issue${p.issues > 1 ? 's' : ''} · ${p.prs} PR`];
    if (p.alertes) lignes.push(`${p.alertes} alerte${p.alertes > 1 ? 's' : ''}`);
    if (p.chantier) lignes.push('chantier à débloquer');
    c.font = `600 11px ${POLICE}`;
    const l = Math.max(...lignes.map((t) => c.measureText(t).width)) + 18;
    const h = lignes.length * 16 + 12;
    let x = etat.curseur.x + 16;
    let y = etat.curseur.y + 16;
    if (x + l > camera.largeur - 8) x = etat.curseur.x - l - 12;
    if (y + h > camera.hauteur - 8) y = etat.curseur.y - h - 12;
    c.fillStyle = 'rgba(12,16,42,0.96)';
    c.strokeStyle = p.accent;
    c.lineWidth = 1.5;
    c.beginPath();
    c.roundRect(x, y, l, h, 5);
    c.fill();
    c.stroke();
    c.textAlign = 'left';
    lignes.forEach((t, i) => {
      c.fillStyle = i === 0 ? '#ffffff' : i === lignes.length - 1 && (p.chantier || p.alertes) ? '#f4b860' : '#aab3d8';
      c.font = `${i === 0 ? 700 : 500} 11px ${POLICE}`;
      c.fillText(t, x + 9, y + 15 + i * 16);
    });
  }

  /* ---------------------------------------------------------------- la boucle */

  /** @param {number} ts */
  function boucle(ts) {
    // l'animation d'ambiance (lumières, fumée, personnages) se contente de 30 images par seconde : la carte reste
    // fluide et ne chauffe pas. Dès qu'on la déplace, on repasse à la cadence de l'écran.
    if (!camera.actif && !etat.curseurActif && ts - dernier < 30) {
      requestAnimationFrame(boucle);
      return;
    }
    const dt = Math.min((ts - dernier) / 1000, 0.05);
    dernier = ts;
    etat.t = ts / 1000;
    const bouge = camera.maj(dt);
    if (animer() && etat.personnages) acteurs.maj(dt);
    dessiner();
    if (animer() || bouge || camera.actif) requestAnimationFrame(boucle);
    else enCours = false;
  }

  function invalider() {
    if (enCours) return;
    enCours = true;
    dernier = performance.now();
    requestAnimationFrame(boucle);
  }

  /* ---------------------------------------------------------------- les entrées */

  /**
   * @param {number} px
   * @param {number} py
   */
  function survoler(px, py) {
    etat.curseur = { x: px, y: py };
    const [wx, wy] = camera.versMonde(px, py);
    etat.survolActeur = etat.personnages ? acteurs.trouver(wx, wy) : null;
    etat.survol = etat.survolActeur ? null : monde.trouver(wx, wy);
    canevas.classList.toggle('pointe', etat.survol !== null);
    invalider();
  }

  brancher(canevas, camera, {
    survol: survoler,
    clic(px, py) {
      const [wx, wy] = camera.versMonde(px, py);
      const b = monde.trouver(wx, wy);
      if (b) interfaceCarte.ouvrir(b.projet.slug);
    },
    sortie() {
      etat.survol = null;
      etat.survolActeur = null;
      invalider();
    },
    change: invalider,
    saisie(actif) {
      etat.curseurActif = actif;
    },
  });

  /** Ce que l'interface peut demander à la carte. */
  const interfaceCarte = brancherInterface({
    donnees,
    surligner(slugs) {
      etat.surligne = new Set(slugs);
      invalider();
    },
    selectionner(slug) {
      etat.selection = slug;
      if (slug) {
        const b = monde.batiments.find((x) => x.projet.slug === slug);
        if (b) camera.aller(b.ancre[0], b.ancre[1] + 30, Math.max(camera.zoom, 0.95));
      }
      invalider();
    },
    reglages(r) {
      Object.assign(etat, r);
      invalider();
    },
    zoomer(sens) {
      if (sens === 0) camera.ajuster(monde.cadre, true);
      else camera.zoomerAutour(sens > 0 ? 1.3 : 1 / 1.3, camera.largeur / 2, camera.hauteur / 2);
      invalider();
    },
  });

  new ResizeObserver(dimensionner).observe(canevas);
  document.addEventListener('visibilitychange', invalider);
  dimensionner();
  invalider();
  canevas.dataset.pret = 'oui';
  void rgba;
}

demarrer().catch((erreur) => {
  // sans carte, tout le reste de la page fonctionne : on le dit dans la console, sans bruit pour la personne
  console.warn('Abvigie : la carte n\'a pas pu démarrer.', erreur);
});
