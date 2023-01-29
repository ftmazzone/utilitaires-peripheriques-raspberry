// Tester cargo run --example tester_ecran

use std::{
    fs::File,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use cairo::{Context, Format, ImageSurface};
use chrono::{Local, Locale, Timelike};
use ecran::capteur_luminosite::capteur::Veml7700;
use ecran::{detecteur::Detecteur, eclairage::Eclairage, ecran::ecran::Wepd7In5BV2};
use rppal::spi::Bus;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    log::info!("Démarrer");

    log::info!("Ecouter les évènements extérieurs");
    let operationnel = Arc::new(AtomicBool::new(true));
    let r = operationnel.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        log::info!("Signal reçu");
        r.store(false, Ordering::SeqCst);
    });

    // Initialiser le capteur de luminosité
    let mut capteur_luminosite = match Veml7700::new() {
        Ok(mut capteur_luminosite) => match capteur_luminosite.initialiser() {
            Ok(_) => Some(capteur_luminosite),
            Err(err) => {
                log::error!("Erreur lors l'initialisation du capteur de luminosité {err}");
                None
            }
        },
        Err(err) => {
            log::error!(
                "Erreur lors de l'initialisation du capteur de luminosité {}",
                err
            );
            None
        }
    };

    // Initialiser l'écran
    let (tx, rx) = flume::unbounded::<bool>();

    let (mut ecran, mut eclairage, detecteur_mouvement) =
        match Wepd7In5BV2::new(Bus::Spi0, 25, 17, 8, 24) {
            Ok(ecran) => {
                log::info!("Configurer l'éclairage");
                let eclairage = Eclairage::new(21);
                let mut detecteur_mouvement = Detecteur::new(16, tx);
                detecteur_mouvement.demarrer().await;

                (Some(ecran), Some(eclairage), Some(detecteur_mouvement))
            }
            Err(err) => {
                log::error!("Erreur lors de l'initialisation de l'écran {}", err);
                (None, None, None)
            }
        };

    let heure_demarrage = Local::now();
    let mut mouvement_detecte = true;
    if eclairage.as_mut().is_some() {
        eclairage.as_mut().unwrap().demarrer();
    }
    afficher_image(&mut ecran, 0, String::new()).await?;

    let mut cpt = 0;

    while operationnel.load(Ordering::SeqCst)
        && !rx.is_disconnected()
        && Local::now() - heure_demarrage < chrono::Duration::minutes(30)
    {
        let resultat = timeout(tokio::time::Duration::from_secs(10), rx.recv_async()).await;

        // Mesurer la luminosité
        let luminosite_lux;
        if capteur_luminosite.is_some() {
            let capteur_luminosite = capteur_luminosite.as_mut().unwrap();

            match capteur_luminosite.arrêter() {
                Ok(_) => {}
                Err(err) => log::error!("Erreur lors du démarrage du capteur de luminosité {err}"),
            }

            match capteur_luminosite.lire_luminosite_lux() {
                Ok(valeur) => {
                    luminosite_lux = valeur.to_string();
                    log::info!("Luminosité mesurée {valeur} lux")
                }
                Err(err) => {
                    log::error!("Erreur lors de lecture de luminosité {err}");
                    luminosite_lux = String::new();
                }
            }

            match capteur_luminosite.arrêter() {
                Ok(_) => {}
                Err(err) => log::error!("Erreur lors de l'arrêt du capteur de luminosité {err}"),
            }
        } else {
            luminosite_lux = String::new();
        }

        // Afficher l'image toutes les dix minutes ou la luminosité en lux mesurée par le capteur
        if mouvement_detecte && (Local::now().minute() % 5) == 0 && Local::now().second() < 10 {
            afficher_image(&mut ecran, cpt, luminosite_lux).await?;
            cpt = (cpt + 1) % 2;
        }

        match resultat {
            Ok(Ok(md)) => match md {
                true => {
                    if eclairage.as_mut().is_some() {
                        eclairage.as_mut().unwrap().demarrer();
                    }
                    mouvement_detecte = true;
                }
                false => {
                    if eclairage.as_mut().is_some() {
                        eclairage.as_mut().unwrap().arreter();
                    }
                    mouvement_detecte = false;
                }
            },
            Ok(Err(e)) => log::error!("read_inputs Error {}", e),
            Err(_e) => (),
        }
    }

    log::info!("Arrêter");
    if detecteur_mouvement.is_some() {
        detecteur_mouvement.unwrap().arreter();
    }
    if eclairage.is_some() {
        eclairage.unwrap().arreter();
    }
    Ok(())
}

pub async fn afficher_image(
    ecran: &mut Option<Wepd7In5BV2>,
    cpt: u8,
    luminosite_lux: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialise cairo
    let mut surface = ImageSurface::create(
        Format::Rgb16_565,
        Wepd7In5BV2::largeur() as i32,
        Wepd7In5BV2::hauteur() as i32,
    )
    .expect("Impossible d'initialiser la surface");

    let contexte = Context::new(&mut surface)?;

    contexte.set_source_rgb(255.0, 255.0, 255.0);
    contexte.paint()?;

    if cpt % 2 == 1 {
        afficher_jour(&contexte)?;
    } else {
        afficher_valeurs_capteurs(&contexte, luminosite_lux)?;
    }

    let mut file = File::create("cairo_output.png").expect("Impossible de créer un fichier");
    surface
        .write_to_png(&mut file)
        .expect("Couldn’t write to png");

    drop(contexte);
    let data = surface.data()?;

    if ecran.is_some() {
        log::info!("Initialiser");
        ecran.as_mut().unwrap().initialiser().await?;
        ecran.as_mut().unwrap().effacer_memoire_tampon()?;
        ecran
            .as_mut()
            .unwrap()
            .sauvegarder_image_memoire_tampon(&data)?;
    }
    drop(data);

    if ecran.is_some() {
        log::info!("Afficher l'image");
        ecran.as_mut().unwrap().mettre_a_jour().await?;
    }
    Ok(())
}

fn afficher_jour(contexte: &Context) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Afficher le jour courant");
    contexte.select_font_face("serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);

    contexte.set_font_size(120.0);
    contexte.set_source_rgb(255., 0., 0.);
    let texte_a_afficher = &Local::now()
        .format_localized("%A", Locale::fr_FR)
        .to_string();
    let mut texte_a_afficher_characteres: Vec<char> = texte_a_afficher.chars().collect();
    texte_a_afficher_characteres[0] = texte_a_afficher_characteres[0].to_uppercase().nth(0).unwrap();
    let texte_a_afficher: String = texte_a_afficher_characteres.into_iter().collect();

    let text_extent = contexte.text_extents(&texte_a_afficher)?;
    let x_offset = (Wepd7In5BV2::largeur() as f64 - text_extent.width()) / 2.0;
    let y_offset = (Wepd7In5BV2::hauteur() as f64 + text_extent.height()) / 4.;
    contexte.move_to(x_offset, y_offset);
    contexte.show_text(&texte_a_afficher)?;

    contexte.set_font_size(120.0);
    contexte.set_source_rgb(0., 0., 0.);
    let texte_a_afficher = &Local::now()
        .format_localized("%e %B", Locale::fr_FR)
        .to_string();
    let text_extent = contexte.text_extents(&texte_a_afficher)?;
    let x_offset = (Wepd7In5BV2::largeur() as f64 - text_extent.width()) / 2.0;
    let y_offset = (Wepd7In5BV2::hauteur() as f64 + text_extent.height()) / 2.;
    contexte.move_to(x_offset, y_offset);
    contexte.show_text(texte_a_afficher)?;

    contexte.set_font_size(120.0);
    contexte.set_source_rgb(0., 0., 0.);
    let texte_a_afficher = &Local::now()
        .format_localized("%R", Locale::fr_FR)
        .to_string();

    let text_extent = contexte.text_extents(&texte_a_afficher)?;
    let x_offset = (Wepd7In5BV2::largeur() as f64 - text_extent.width()) / 2.0;
    let y_offset = (Wepd7In5BV2::hauteur() as f64 + text_extent.height() + 120. / 4.) * 3. / 4.;
    contexte.move_to(x_offset, y_offset);
    contexte.show_text(texte_a_afficher)?;
    Ok(())
}

fn afficher_valeurs_capteurs(
    contexte: &Context,
    luminosite_lux: String,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Afficher le jour courant");
    contexte.select_font_face("serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);

    contexte.set_font_size(50.0);
    contexte.set_source_rgb(0., 0., 0.);
    let texte_a_afficher =format!("Luminosité: {luminosite_lux} lux");

    let text_extent = contexte.text_extents(&texte_a_afficher)?;
    let x_offset = (Wepd7In5BV2::largeur() as f64 - text_extent.width()) / 2.0;
    let y_offset = (Wepd7In5BV2::hauteur() as f64 + text_extent.height() + 120. / 4.) * 1. / 4.;
    contexte.move_to(x_offset, y_offset);
    contexte.show_text(&texte_a_afficher)?;
    Ok(())
}
