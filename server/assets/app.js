// @ts-check
/**
 * Abvigie : le panneau latéral. Un clic sur un bâtiment ou une carte ouvre la fiche du projet sans quitter la page.
 *
 * Tout fonctionne sans ce script : les liens ouvrent alors la page du projet (`/projets/<nom>`).
 * Aucune dépendance. Le HTML reçu vient du même serveur, qui échappe toutes les données.
 */
(() => {
  const element = document.getElementById('panneau');
  if (!(element instanceof HTMLDialogElement) || typeof element.showModal !== 'function') return;
  const panneau = element;
  const zone = /** @type {Element} */ (panneau.querySelector('.panneau-contenu'));
  if (!zone) return;

  const PREFIXE = '/projets/';

  /**
   * @param {string} chemin  chemin d'une fiche, par exemple « /projets/abcom »
   * @param {boolean} pousser  ajouter l'étape dans l'historique du navigateur
   */
  async function ouvrir(chemin, pousser) {
    try {
      const reponse = await fetch(`${chemin}?partial=1`, { headers: { Accept: 'text/html' } });
      if (!reponse.ok) throw new Error(String(reponse.status));
      zone.innerHTML = await reponse.text();
    } catch {
      // en cas de souci, on retombe sur la navigation classique
      location.assign(chemin);
      return;
    }
    if (!panneau.open) panneau.showModal();
    panneau.scrollTop = 0;
    if (pousser) history.pushState({ panneau: chemin }, '', chemin);
  }

  function fermer() {
    if (panneau.open) panneau.close();
  }

  document.addEventListener('click', (evenement) => {
    const cible = evenement.target instanceof Element ? evenement.target.closest('a[data-projet]') : null;
    if (!cible || evenement.button !== 0 || evenement.ctrlKey || evenement.metaKey || evenement.shiftKey || evenement.altKey) return;
    const chemin = cible.getAttribute('href');
    if (!chemin || !chemin.startsWith(PREFIXE)) return;
    evenement.preventDefault();
    ouvrir(chemin, true);
  });

  // un clic sur le fond, ou sur le bouton de fermeture
  panneau.addEventListener('click', (evenement) => {
    const cible = evenement.target;
    if (cible === panneau || (cible instanceof Element && cible.closest('.fermer'))) fermer();
  });

  // Échap ou fermeture : on revient à l'étape précédente de l'historique, sauf si c'est déjà fait
  panneau.addEventListener('close', () => {
    if (history.state && history.state.panneau) history.back();
  });

  // boutons précédent / suivant du navigateur
  window.addEventListener('popstate', () => {
    if (location.pathname.startsWith(PREFIXE)) ouvrir(location.pathname, false);
    else fermer();
  });
})();
