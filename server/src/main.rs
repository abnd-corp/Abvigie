//! Abvigie : le tableau de bord public de l'organisation Abnd.
//!
//! Au démarrage : lire GitHub (ou des fichiers locaux), fabriquer toutes les pages, puis servir.
//! Toutes les 15 minutes : recommencer, et remplacer les pages d'un coup. Voir `docs/architecture.md`.

mod catalogue;
mod collect;
mod config;
mod dates;
mod http;
mod model;
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
    process::ExitCode,
    sync::{Arc, PoisonError, RwLock},
    thread,
    time::Duration,
};

const AIDE: &str = "\
Abvigie : tableau de bord public de l'organisation Abnd.

Utilisation : abvigie [--fixtures <dossier>] [--adresse <hôte:port>]

Réglages par variables d'environnement : voir docs/deploiement.md
  ABVIGIE_ORG, ABVIGIE_ADRESSE, ABVIGIE_URL, ABVIGIE_RAFRAICHIR,
  ABVIGIE_ENDORMI_JOURS, ABVIGIE_REVUE_JOURS, GITHUB_TOKEN
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
                loop {
                    thread::sleep(Duration::from_secs(config.rafraichir_secondes));
                    match fabriquer(source.as_ref(), &config) {
                        Ok(site) => {
                            *courant.write().unwrap_or_else(PoisonError::into_inner) =
                                Arc::new(site);
                            journal("données rafraîchies");
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

/// Lit les données et fabrique le site. Une panique est transformée en erreur : elle ne doit jamais tuer le service.
fn fabriquer(source: &dyn Source, config: &Config) -> Result<Site, String> {
    let maintenant = dates::maintenant();
    panic::catch_unwind(AssertUnwindSafe(|| {
        let etat = collect::construire(source, config, maintenant).map_err(|e| e.to_string())?;
        Ok(Site::construire(&etat, config, maintenant))
    }))
    .unwrap_or_else(|_| Err("erreur interne pendant la fabrication des pages".into()))
}
