//! Abvigie : le tableau de bord public de l'organisation Abnd.
//!
//! Au démarrage : lire GitHub (ou des fichiers locaux), fabriquer toutes les pages, puis servir.
//! À l'intervalle réglé dans `abvigie.conf` (6 heures par défaut) : recommencer, et remplacer les pages d'un coup.
//! Avec `--export <dossier>`, écrire le site en fichiers statiques (GitHub Pages) et s'arrêter.
//! Voir `docs/architecture.md`.

mod catalogue;
mod collect;
mod config;
mod dates;
mod http;
mod model;
mod progression;
mod render;
mod rules;
mod site;
mod sources;

use config::Config;
use site::Site;
use sources::{Source, fixtures::Fixtures, github::GitHub};
use std::{
    net::TcpListener,
    panic::{self, AssertUnwindSafe},
    path::Path,
    process::ExitCode,
    sync::{Arc, PoisonError, RwLock},
    thread,
    time::Duration,
};

const AIDE: &str = "\
Abvigie : tableau de bord public de l'organisation Abnd.

Utilisation : abvigie [--config <fichier>] [--fixtures <dossier>] [--adresse <hôte:port>]
              abvigie --export <dossier> [--fixtures <dossier>]   (site statique, sans serveur)

Réglages : fichier abvigie.conf (commenté), puis variables d'environnement, puis arguments.
  ABVIGIE_ORG, ABVIGIE_ADRESSE, ABVIGIE_URL, ABVIGIE_RAFRAICHIR,
  ABVIGIE_ENDORMI_JOURS, ABVIGIE_REVUE_JOURS, ABVIGIE_CONFIG, GITHUB_TOKEN (jamais dans le fichier)
Voir docs/deploiement.md
";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments
        .iter()
        .any(|a| a == "--aide" || a == "--help" || a == "-h")
    {
        print!("{AIDE}");
        return ExitCode::SUCCESS;
    }
    if arguments.iter().any(|a| a == "--version") {
        println!("abvigie {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    match executer(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("abvigie : {e}");
            ExitCode::FAILURE
        }
    }
}

fn journal(message: &str) {
    eprintln!("{} {message}", dates::iso(dates::maintenant()));
}

fn executer(arguments: &[String]) -> Result<(), String> {
    let config = Config::depuis_environnement(arguments)?;
    let source: Box<dyn Source> = match &config.fixtures {
        Some(dossier) => Box::new(Fixtures::new(dossier.clone())),
        None => Box::new(GitHub::new(&config)),
    };

    if let Some(dossier) = &config.exporter {
        return exporter(source.as_ref(), &config, dossier);
    }

    let ecouteur = TcpListener::bind(&config.adresse)
        .map_err(|e| format!("impossible d'écouter sur {} : {e}", config.adresse))?;
    let origine = match (&config.fixtures, &config.jeton) {
        (Some(d), _) => format!("fichiers locaux ({})", d.display()),
        (None, Some(_)) => "GitHub, avec jeton".to_string(),
        (None, None) => "GitHub, sans jeton (60 requêtes par heure)".to_string(),
    };
    journal(&format!(
        "Abvigie {} sur http://{} · organisation {} · source : {origine}",
        env!("CARGO_PKG_VERSION"),
        config.adresse,
        config.organisation
    ));
    journal(&format!(
        "configuration : {} · lecture de GitHub toutes les {}",
        config
            .fichier
            .as_ref()
            .map_or("valeurs par défaut".to_string(), |f| f
                .display()
                .to_string()),
        config::duree_lisible(config.rafraichir_secondes)
    ));

    let courant = match fabriquer(source.as_ref(), &config) {
        Ok(site) => {
            journal(&format!("{} projets lus", site.projets));
            site
        }
        Err(raison) => {
            journal(&format!("lecture impossible au démarrage : {raison}"));
            Site::degrade(&config, &raison, dates::maintenant())
        }
    };
    let courant = Arc::new(RwLock::new(Arc::new(courant)));

    {
        let courant = Arc::clone(&courant);
        let config = config.clone();
        thread::Builder::new()
            .name("rafraichissement".into())
            .spawn(move || {
                let mut partiels = 0;
                loop {
                    thread::sleep(Duration::from_secs(config.rafraichir_secondes));
                    match fabriquer(source.as_ref(), &config) {
                        Ok(site) => {
                            let actuel =
                                Arc::clone(&courant.read().unwrap_or_else(PoisonError::into_inner));
                            if site::faut_remplacer(&actuel, &site, &mut partiels) {
                                *courant.write().unwrap_or_else(PoisonError::into_inner) =
                                    Arc::new(site);
                                journal("données rafraîchies");
                            } else {
                                journal("lecture partielle (limite de GitHub ?) : pages précédentes conservées");
                            }
                        }
                        // on garde les pages précédentes : mieux vaut une donnée un peu ancienne que pas de page
                        Err(raison) => journal(&format!(
                            "rafraîchissement impossible, pages conservées : {raison}"
                        )),
                    }
                }
            })
            .map_err(|e| format!("impossible de démarrer le rafraîchissement : {e}"))?;
    }

    http::servir(ecouteur, courant).map_err(|e| format!("serveur arrêté : {e}"))
}

/// Mode statique : fabrique le site une fois, l'écrit en fichiers dans `dossier`, puis s'arrête.
fn exporter(source: &dyn Source, config: &Config, dossier: &Path) -> Result<(), String> {
    let site = fabriquer(source, config)?;
    if site.partiel {
        // un site statique est publié tel quel : on préfère ne rien publier qu'une ville qui affiche « rien à signaler » à tort
        return Err(
            "lecture partielle des données : rien n'est écrit (voir les avertissements)".into(),
        );
    }
    let n = site.exporter(dossier)?;
    journal(&format!(
        "{} projets, {n} fichiers écrits dans {} (adresse publique : {})",
        site.projets,
        dossier.display(),
        config.url_publique
    ));
    Ok(())
}

/// Lit les données et fabrique le site. Une panique est transformée en erreur : elle ne doit jamais tuer le service.
fn fabriquer(source: &dyn Source, config: &Config) -> Result<Site, String> {
    let maintenant = source.instant().unwrap_or_else(dates::maintenant);
    panic::catch_unwind(AssertUnwindSafe(|| {
        let etat = collect::construire(source, config, maintenant).map_err(|e| e.to_string())?;
        // la date réelle, pas celle d'une capture figée : elle sert à l'expiration de security.txt
        Ok(Site::construire(&etat, config, dates::maintenant()))
    }))
    .unwrap_or_else(|_| Err("erreur interne pendant la fabrication des pages".into()))
}
