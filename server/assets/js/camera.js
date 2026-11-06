// @ts-check
/**
 * La caméra : le point du monde au centre de l'écran et le niveau de zoom, plus tout ce qui la pilote : glisser pour se
 * déplacer (avec un peu d'inertie), molette ou pincement pour zoomer, clavier, et un déplacement en douceur vers un point.
 */

const ZOOM_MIN = 0.3;
const ZOOM_MAX = 2.6;
/** Au-dessous de cette distance (en pixels), un appui est un clic et non un glissement. */
const SEUIL_CLIC = 6;

export class Camera {
  constructor() {
    this.x = 0;
    this.y = 0;
    this.zoom = 1;
    this.largeur = 1;
    this.hauteur = 1;
    /** @type {{ x0: number, y0: number, x1: number, y1: number }} */
    this.limites = { x0: -1e4, y0: -1e4, x1: 1e4, y1: 1e4 };
    this.vx = 0;
    this.vy = 0;
    /** @type {{ x0: number, y0: number, z0: number, x1: number, y1: number, z1: number, t: number, duree: number } | null} */
    this.transition = null;
    /** Vrai tant que quelque chose bouge : la carte doit se redessiner. */
    this.actif = false;
    /** Vrai dès que la personne a déplacé ou zoomé la carte : on ne recadre plus toute seule. */
    this.manuel = false;
  }

  /** @param {number} largeur @param {number} hauteur */
  dimensionner(largeur, hauteur) {
    this.largeur = largeur;
    this.hauteur = hauteur;
  }

  /**
   * Cadre toute la ville avec un peu d'air.
   * @param {{ x0: number, y0: number, x1: number, y1: number }} boite
   * @param {boolean} [doucement]
   */
  ajuster(boite, doucement = false) {
    const z = Math.min(this.largeur / (boite.x1 - boite.x0), this.hauteur / (boite.y1 - boite.y0));
    this.aller((boite.x0 + boite.x1) / 2, (boite.y0 + boite.y1) / 2, Math.min(Math.max(z * 1.0, ZOOM_MIN), 1.3), doucement);
  }

  /**
   * @param {number} x
   * @param {number} y
   * @param {number} zoom
   * @param {boolean} [doucement]
   */
  aller(x, y, zoom, doucement = true) {
    zoom = Math.min(Math.max(zoom, ZOOM_MIN), ZOOM_MAX);
    if (!doucement) {
      this.x = x;
      this.y = y;
      this.zoom = zoom;
      this.vx = this.vy = 0;
      this.transition = null;
      this.actif = true;
      return;
    }
    this.transition = { x0: this.x, y0: this.y, z0: this.zoom, x1: x, y1: y, z1: zoom, t: 0, duree: 0.65 };
    this.actif = true;
  }

  /**
   * Un pixel de l'écran (relatif au canevas) vers un point du monde.
   * @param {number} px
   * @param {number} py
   * @returns {[number, number]}
   */
  versMonde(px, py) {
    return [(px - this.largeur / 2) / this.zoom + this.x, (py - this.hauteur / 2) / this.zoom + this.y];
  }

  /**
   * Un point du monde vers un pixel de l'écran.
   * @param {number} wx
   * @param {number} wy
   * @returns {[number, number]}
   */
  versEcran(wx, wy) {
    return [(wx - this.x) * this.zoom + this.largeur / 2, (wy - this.y) * this.zoom + this.hauteur / 2];
  }

  /** La zone du monde visible, avec une marge. */
  zoneVisible(marge = 60) {
    const [x0, y0] = this.versMonde(0, 0);
    const [x1, y1] = this.versMonde(this.largeur, this.hauteur);
    return { x0: x0 - marge, y0: y0 - marge, x1: x1 + marge, y1: y1 + marge };
  }

  /**
   * Zoome en gardant le point sous le curseur à sa place.
   * @param {number} facteur
   * @param {number} px
   * @param {number} py
   */
  zoomerAutour(facteur, px, py) {
    this.transition = null;
    this.manuel = true;
    const [wx, wy] = this.versMonde(px, py);
    this.zoom = Math.min(Math.max(this.zoom * facteur, ZOOM_MIN), ZOOM_MAX);
    this.x = wx - (px - this.largeur / 2) / this.zoom;
    this.y = wy - (py - this.hauteur / 2) / this.zoom;
    this.contraindre();
    this.actif = true;
  }

  /** Garde la ville à l'écran : le centre ne sort pas de la plaque. */
  contraindre() {
    const l = this.limites;
    this.x = Math.min(Math.max(this.x, l.x0), l.x1);
    this.y = Math.min(Math.max(this.y, l.y0), l.y1);
  }

  /**
   * Fait avancer l'inertie et les transitions. Renvoie vrai tant que la caméra bouge.
   * @param {number} dt  secondes
   */
  maj(dt) {
    let bouge = false;
    if (this.transition) {
      const tr = this.transition;
      tr.t = Math.min(1, tr.t + dt / tr.duree);
      const e = tr.t < 0.5 ? 2 * tr.t * tr.t : 1 - Math.pow(-2 * tr.t + 2, 2) / 2;
      this.x = tr.x0 + (tr.x1 - tr.x0) * e;
      this.y = tr.y0 + (tr.y1 - tr.y0) * e;
      this.zoom = tr.z0 + (tr.z1 - tr.z0) * e;
      if (tr.t >= 1) this.transition = null;
      bouge = true;
    } else if (Math.abs(this.vx) > 4 || Math.abs(this.vy) > 4) {
      this.x += (this.vx * dt) / this.zoom;
      this.y += (this.vy * dt) / this.zoom;
      const amorti = Math.exp(-5 * dt);
      this.vx *= amorti;
      this.vy *= amorti;
      this.contraindre();
      bouge = true;
    }
    this.actif = bouge;
    return bouge;
  }
}

/**
 * Branche la souris, le toucher et le clavier d'un canevas sur une caméra.
 * @param {HTMLCanvasElement} canevas
 * @param {Camera} camera
 * @param {{
 *   survol: (px: number, py: number) => void,
 *   clic: (px: number, py: number) => void,
 *   sortie: () => void,
 *   change: () => void,
 *   saisie?: (actif: boolean) => void
 * }} rappels
 */
export function brancher(canevas, camera, rappels) {
  /** @type {Map<number, { x: number, y: number }>} */
  const doigts = new Map();
  let deplacement = 0;
  let distance = 0;
  let dernierT = 0;

  /** @param {PointerEvent | WheelEvent} e @returns {[number, number]} */
  const local = (e) => {
    const r = canevas.getBoundingClientRect();
    return [e.clientX - r.left, e.clientY - r.top];
  };

  canevas.addEventListener('pointerdown', (e) => {
    canevas.setPointerCapture(e.pointerId);
    const [x, y] = local(e);
    doigts.set(e.pointerId, { x, y });
    deplacement = 0;
    camera.vx = camera.vy = 0;
    camera.transition = null;
    if (doigts.size === 2) {
      const [a, b] = [...doigts.values()];
      distance = Math.hypot(a.x - b.x, a.y - b.y);
    }
    canevas.classList.add('saisie');
    camera.manuel = true;
    rappels.saisie?.(true);
    dernierT = performance.now();
  });

  canevas.addEventListener('pointermove', (e) => {
    const [x, y] = local(e);
    const d = doigts.get(e.pointerId);
    if (!d) {
      if (e.pointerType === 'mouse') rappels.survol(x, y);
      return;
    }
    const dx = x - d.x;
    const dy = y - d.y;
    d.x = x;
    d.y = y;
    deplacement += Math.abs(dx) + Math.abs(dy);
    if (doigts.size === 1) {
      if (deplacement > SEUIL_CLIC) {
        camera.x -= dx / camera.zoom;
        camera.y -= dy / camera.zoom;
        const maintenant = performance.now();
        const dt = Math.max((maintenant - dernierT) / 1000, 0.001);
        dernierT = maintenant;
        camera.vx = camera.vx * 0.6 + (-dx / dt) * 0.4;
        camera.vy = camera.vy * 0.6 + (-dy / dt) * 0.4;
        camera.contraindre();
        rappels.change();
      }
    } else if (doigts.size === 2) {
      const [a, b] = [...doigts.values()];
      const nouvelle = Math.hypot(a.x - b.x, a.y - b.y);
      if (distance > 0) camera.zoomerAutour(nouvelle / distance, (a.x + b.x) / 2, (a.y + b.y) / 2);
      distance = nouvelle;
      rappels.change();
    }
  });

  /** @param {PointerEvent} e */
  const fin = (e) => {
    const d = doigts.get(e.pointerId);
    if (!d) return;
    doigts.delete(e.pointerId);
    canevas.classList.remove('saisie');
    if (doigts.size === 0) {
      rappels.saisie?.(false);
      if (deplacement <= SEUIL_CLIC) {
        camera.vx = camera.vy = 0;
        rappels.clic(d.x, d.y);
      } else if (performance.now() - dernierT > 90) {
        camera.vx = camera.vy = 0;
      }
      rappels.change();
    }
  };
  canevas.addEventListener('pointerup', fin);
  canevas.addEventListener('pointercancel', fin);
  canevas.addEventListener('pointerleave', (e) => {
    if (!doigts.has(e.pointerId)) rappels.sortie();
  });

  canevas.addEventListener(
    'wheel',
    (e) => {
      e.preventDefault();
      const [x, y] = local(e);
      camera.zoomerAutour(Math.exp(-e.deltaY * (e.ctrlKey ? 0.01 : 0.0016)), x, y);
      rappels.change();
    },
    { passive: false },
  );

  canevas.addEventListener('keydown', (e) => {
    const pas = 70 / camera.zoom;
    const touches = /** @type {Record<string, () => void>} */ ({
      ArrowLeft: () => (camera.x -= pas),
      ArrowRight: () => (camera.x += pas),
      ArrowUp: () => (camera.y -= pas),
      ArrowDown: () => (camera.y += pas),
      '+': () => camera.zoomerAutour(1.25, camera.largeur / 2, camera.hauteur / 2),
      '=': () => camera.zoomerAutour(1.25, camera.largeur / 2, camera.hauteur / 2),
      '-': () => camera.zoomerAutour(0.8, camera.largeur / 2, camera.hauteur / 2),
    });
    const f = touches[e.key];
    if (!f) return;
    e.preventDefault();
    f();
    camera.contraindre();
    camera.actif = true;
    rappels.change();
  });
}
