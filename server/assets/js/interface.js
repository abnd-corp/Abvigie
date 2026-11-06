// @ts-check
/**
 * L'interface autour de la carte : la fiche d'un projet dans le panneau, les quêtes qui éclairent leurs bâtiments, les
 * réglages (noms, alertes, personnages) et les boutons de zoom. Les onglets, eux, n'ont pas besoin de JavaScript.
 *
 * Sans JavaScript, tous les liens ouvrent la page du projet (`/projets/<nom>`) : rien ne dépend de ce fichier.
 */
import { dessinerVignettes } from './vignettes.js';

/** @typedef {import('./donnees.js').Donnees} Donnees */

const PREFIXE = '/projets/';

/**
 * @param {{
 *   donnees: Donnees,
 *   surligner: (slugs: string[]) => void,
 *   selectionner: (slug: string | null) => void,
 *   reglages: (r: { noms?: boolean, alertes?: boolean, personnages?: boolean }) => void,
 *   zoomer: (sens: number) => void
 * }} carte
 */
export function brancherInterface(carte) {
  const lateral = document.querySelector('.lateral');
  const vueProjet = /** @type {HTMLElement | null} */ (document.querySelector('.vue-projet'));
  const contenu = /** @type {HTMLElement | null} */ (document.querySelector('.contenu-projet'));
  const vueEnsemble = /** @type {HTMLElement | null} */ (document.querySelector('.vue-ensemble'));

  /**
   * Ouvre la fiche d'un projet dans le panneau, sans quitter la page.
   * @param {string} slug
   * @param {boolean} [pousser]  ajouter l'étape à l'historique
   */
  async function ouvrir(slug, pousser = true) {
    if (!vueProjet || !contenu || !vueEnsemble) return false;
    const chemin = `${PREFIXE}${encodeURIComponent(slug)}`;
    try {
      const reponse = await fetch(`${chemin}?partial=1`, { headers: { Accept: 'text/html' } });
      if (!reponse.ok) throw new Error(String(reponse.status));
      contenu.innerHTML = await reponse.text();
    } catch {
      location.assign(chemin);
      return false;
    }
    vueEnsemble.hidden = true;
    vueProjet.hidden = false;
    lateral?.scrollTo({ top: 0 });
    dessinerVignettes(contenu);
    carte.selectionner(slug);
    if (pousser) history.pushState({ projet: slug }, '', chemin);
    return true;
  }

  function fermer() {
    if (!vueProjet || !vueEnsemble) return;
    vueProjet.hidden = true;
    vueEnsemble.hidden = false;
    carte.selectionner(null);
  }

  // un clic sur un lien de projet (une carte, une quête, le journal) ouvre la fiche dans le panneau
  document.addEventListener('click', (evenement) => {
    const cible = evenement.target instanceof Element ? evenement.target.closest('a[data-projet]') : null;
    if (!cible || /** @type {MouseEvent} */ (evenement).button !== 0) return;
    const e = /** @type {MouseEvent} */ (evenement);
    if (e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) return;
    const href = cible.getAttribute('href');
    if (!href || !href.startsWith(PREFIXE)) return;
    evenement.preventDefault();
    ouvrir(decodeURIComponent(href.slice(PREFIXE.length)));
  });

  document.querySelector('.retour')?.addEventListener('click', () => {
    if (history.state && history.state.projet) history.back();
    else fermer();
  });

  window.addEventListener('popstate', () => {
    if (location.pathname.startsWith(PREFIXE)) ouvrir(decodeURIComponent(location.pathname.slice(PREFIXE.length)), false);
    else fermer();
  });

  // une quête qu'on survole ou qu'on touche éclaire, sur la carte, les projets qui lui restent à faire
  /** @type {string | null} */
  let epinglee = null;
  document.querySelectorAll('.quete').forEach((q) => {
    const slugs = () => (q.getAttribute('data-projets') ?? '').split(' ').filter(Boolean);
    const id = q.getAttribute('data-quete');
    q.addEventListener('mouseenter', () => carte.surligner(slugs()));
    q.addEventListener('mouseleave', () => carte.surligner(epinglee === id ? slugs() : []));
    q.addEventListener('click', (e) => {
      if (e.target instanceof Element && e.target.closest('a')) return;
      epinglee = epinglee === id ? null : id;
      q.classList.toggle('epinglee', epinglee === id);
      document.querySelectorAll('.quete').forEach((autre) => autre !== q && autre.classList.remove('epinglee'));
      carte.surligner(epinglee === id ? slugs() : []);
    });
  });

  // les réglages, mémorisés si le navigateur le permet
  /** @type {Array<['opt-noms' | 'opt-alertes' | 'opt-acteurs', 'noms' | 'alertes' | 'personnages']>} */
  const cases = [['opt-noms', 'noms'], ['opt-alertes', 'alertes'], ['opt-acteurs', 'personnages']];
  for (const [id, cle] of cases) {
    const input = document.getElementById(id);
    if (!(input instanceof HTMLInputElement)) continue;
    try {
      const memo = localStorage.getItem(`abvigie.${cle}`);
      if (memo !== null) input.checked = memo === '1';
    } catch {
      // stockage indisponible : on garde la valeur de la page
    }
    carte.reglages({ [cle]: input.checked });
    input.addEventListener('change', () => {
      carte.reglages({ [cle]: input.checked });
      try {
        localStorage.setItem(`abvigie.${cle}`, input.checked ? '1' : '0');
      } catch {
        // pas grave
      }
    });
  }

  document.querySelectorAll('[data-zoom]').forEach((b) => b.addEventListener('click', () => carte.zoomer(Number(b.getAttribute('data-zoom')))));

  dessinerVignettes(document);
  return { ouvrir };
}
