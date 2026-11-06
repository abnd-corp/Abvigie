// @ts-check
/**
 * Les données de la carte : lues dans la page (incorporées par le serveur) ou, à défaut, sur `/api/carte.json`.
 * Elles ne contiennent que des faits publics.
 */

/**
 * @typedef {{
 *   slug: string, nom: string, quartier: string, accent: string,
 *   lumieres: number, jours_inactif: number, chantier: boolean, archive: boolean,
 *   gravite: 'info' | 'attention' | 'alerte' | null,
 *   alertes: number, issues: number, prs: number
 * }} Projet
 */
/** @typedef {{ login: string, robot: boolean, activite: number }} Acteur */
/** @typedef {{ id: string, accomplie: boolean, a_traiter: string[] }} Quete */
/**
 * @typedef {{
 *   version: number, genere_le: string, endormi_jours: number,
 *   quartiers: { id: string, titre: string, n: number }[],
 *   projets: Projet[], acteurs: Acteur[], quetes: Quete[]
 * }} Donnees
 */

/** @returns {Promise<Donnees>} */
export async function charger() {
  const el = document.getElementById('donnees-carte');
  if (el && el.textContent) return JSON.parse(el.textContent);
  const reponse = await fetch('/api/carte.json');
  if (!reponse.ok) throw new Error(`carte.json : HTTP ${reponse.status}`);
  return reponse.json();
}
