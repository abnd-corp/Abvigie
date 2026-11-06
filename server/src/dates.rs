//! Dates sans dépendance : lecture des dates ISO 8601 de GitHub et affichage en français.
//!
//! GitHub renvoie des dates en UTC (« 2026-11-06T12:34:56Z ») : le fuseau n'est pas géré.

use std::time::{SystemTime, UNIX_EPOCH};

const SECONDES_PAR_JOUR: i64 = 86_400;

const MOIS: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];

/// Secondes écoulées depuis l'époque Unix.
pub fn maintenant() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Jours entre le 1er janvier 1970 et une date civile (algorithme de Howard Hinnant).
fn jours_depuis_epoque(annee: i64, mois: i64, jour: i64) -> i64 {
    let a = if mois <= 2 { annee - 1 } else { annee };
    let ere = a.div_euclid(400);
    let annee_ere = a.rem_euclid(400);
    let mp = (mois + 9) % 12;
    let jour_annee = (153 * mp + 2) / 5 + jour - 1;
    let jour_ere = annee_ere * 365 + annee_ere / 4 - annee_ere / 100 + jour_annee;
    ere * 146_097 + jour_ere - 719_468
}

/// Date civile (année, mois, jour) d'un nombre de jours depuis l'époque. Inverse de [`jours_depuis_epoque`].
fn date_civile(jours: i64) -> (i64, i64, i64) {
    let z = jours + 719_468;
    let ere = z.div_euclid(146_097);
    let jour_ere = z.rem_euclid(146_097);
    let annee_ere = (jour_ere - jour_ere / 1_460 + jour_ere / 36_524 - jour_ere / 146_096) / 365;
    let jour_annee = jour_ere - (365 * annee_ere + annee_ere / 4 - annee_ere / 100);
    let mp = (5 * jour_annee + 2) / 153;
    let jour = jour_annee - (153 * mp + 2) / 5 + 1;
    let mois = if mp < 10 { mp + 3 } else { mp - 9 };
    let annee = annee_ere + ere * 400 + i64::from(mois <= 2);
    (annee, mois, jour)
}

/// Lit « AAAA-MM-JJ » ou « AAAA-MM-JJTHH:MM:SSZ » et renvoie les secondes Unix.
pub fn analyser(texte: &str) -> Option<i64> {
    let o = texte.as_bytes();
    if o.len() < 10 || o[4] != b'-' || o[7] != b'-' {
        return None;
    }
    let nombre =
        |debut: usize, fin: usize| -> Option<i64> { texte.get(debut..fin)?.parse::<i64>().ok() };
    let (annee, mois, jour) = (nombre(0, 4)?, nombre(5, 7)?, nombre(8, 10)?);
    if !(1..=12).contains(&mois) || !(1..=31).contains(&jour) {
        return None;
    }
    let heure = if o.len() >= 19 && (o[10] == b'T' || o[10] == b' ') {
        nombre(11, 13)? * 3_600 + nombre(14, 16)? * 60 + nombre(17, 19)?
    } else {
        0
    };
    Some(jours_depuis_epoque(annee, mois, jour) * SECONDES_PAR_JOUR + heure)
}

/// Jours entiers écoulés entre `date` et `maintenant` (0 si `date` est dans le futur).
pub fn jours_ecoules(date: i64, maintenant: i64) -> i64 {
    ((maintenant - date) / SECONDES_PAR_JOUR).max(0)
}

/// « 2026-11-06T12:34:56Z ».
pub fn iso(secondes: i64) -> String {
    let (annee, mois, jour) = date_civile(secondes.div_euclid(SECONDES_PAR_JOUR));
    let reste = secondes.rem_euclid(SECONDES_PAR_JOUR);
    format!(
        "{annee:04}-{mois:02}-{jour:02}T{:02}:{:02}:{:02}Z",
        reste / 3_600,
        reste % 3_600 / 60,
        reste % 60
    )
}

/// « 6 novembre 2026 » (« 1er » pour le premier du mois).
pub fn date_longue(secondes: i64) -> String {
    let (annee, mois, jour) = date_civile(secondes.div_euclid(SECONDES_PAR_JOUR));
    let jour = if jour == 1 {
        "1er".to_string()
    } else {
        jour.to_string()
    };
    format!("{jour} {} {annee}", MOIS[(mois - 1) as usize])
}

/// Date au format HTTP (RFC 9110) : « Fri, 06 Nov 2026 12:34:56 GMT ».
pub fn http(secondes: i64) -> String {
    const JOURS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MOIS_EN: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let jours = secondes.div_euclid(SECONDES_PAR_JOUR);
    let (annee, mois, jour) = date_civile(jours);
    let reste = secondes.rem_euclid(SECONDES_PAR_JOUR);
    // le 1er janvier 1970 était un jeudi
    let semaine = JOURS[(jours + 4).rem_euclid(7) as usize];
    format!(
        "{semaine}, {jour:02} {} {annee:04} {:02}:{:02}:{:02} GMT",
        MOIS_EN[(mois - 1) as usize],
        reste / 3_600,
        reste % 3_600 / 60,
        reste % 60
    )
}

/// « aujourd'hui », « hier », « il y a 12 jours », « il y a 3 mois »…
pub fn il_y_a(jours: i64) -> String {
    match jours {
        0 => "aujourd'hui".into(),
        1 => "hier".into(),
        2..=59 => format!("il y a {jours} jours"),
        60..=729 => format!("il y a {} mois", jours / 30),
        _ => format!("il y a {} ans", jours / 365),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyse_et_formate_dans_les_deux_sens() {
        let t = analyser("2026-11-06T12:34:56Z").unwrap();
        assert_eq!(iso(t), "2026-11-06T12:34:56Z");
        assert_eq!(analyser("1970-01-01"), Some(0));
        assert_eq!(iso(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn gere_les_annees_bissextiles() {
        let t = analyser("2024-02-29T00:00:00Z").unwrap();
        assert_eq!(iso(t + 86_400), "2024-03-01T00:00:00Z");
        assert_eq!(
            iso(analyser("2100-02-28").unwrap() + 86_400),
            "2100-03-01T00:00:00Z"
        );
    }

    #[test]
    fn refuse_les_dates_invalides() {
        for mauvais in [
            "",
            "2026",
            "2026/11/06",
            "2026-13-01",
            "2026-00-10",
            "abcd-ef-gh",
        ] {
            assert_eq!(analyser(mauvais), None, "{mauvais}");
        }
    }

    #[test]
    fn jours_ecoules_ne_devient_jamais_negatif() {
        assert_eq!(jours_ecoules(1_000, 500), 0);
        assert_eq!(jours_ecoules(0, 3 * 86_400 + 10), 3);
    }

    #[test]
    fn date_http() {
        assert_eq!(http(0), "Thu, 01 Jan 1970 00:00:00 GMT");
        assert_eq!(
            http(analyser("2026-11-06T12:34:56Z").unwrap()),
            "Fri, 06 Nov 2026 12:34:56 GMT"
        );
    }

    #[test]
    fn dates_en_francais() {
        assert_eq!(
            date_longue(analyser("2026-11-06").unwrap()),
            "6 novembre 2026"
        );
        assert_eq!(
            date_longue(analyser("2026-12-01").unwrap()),
            "1er décembre 2026"
        );
        assert_eq!(il_y_a(0), "aujourd'hui");
        assert_eq!(il_y_a(1), "hier");
        assert_eq!(il_y_a(12), "il y a 12 jours");
        assert_eq!(il_y_a(90), "il y a 3 mois");
        assert_eq!(il_y_a(800), "il y a 2 ans");
    }
}
