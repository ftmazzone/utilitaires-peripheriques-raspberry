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
    afficher_image(&mut ecran).await?;

    while operationnel.load(Ordering::SeqCst)
        && !rx.is_disconnected()
        && Local::now() - heure_demarrage < chrono::Duration::minutes(30)
    {
        let resultat = timeout(tokio::time::Duration::from_secs(10), rx.recv_async()).await;

        // Affichager l'image toutes les cinq minutes
        if mouvement_detecte && (Local::now().minute() % 5) == 0 && Local::now().second() < 10 {
            afficher_image(&mut ecran).await?;
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
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialise cairo
    let mut surface = ImageSurface::create(
        Format::Rgb16_565,
        Wepd7In5BV2::largeur() as i32,
        Wepd7In5BV2::hauteur() as i32,
    )
    .expect("Couldn’t create surface");

    let context = Context::new(&mut surface)?;

    context.set_source_rgb(255.0, 255.0, 255.0);
    context.paint()?;

    log::info!("Afficher le jour courant");
    context.select_font_face("serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);

    context.set_font_size(120.0);
    context.set_source_rgb(255., 0., 0.);
    let text_to_display = &Local::now()
        .format_localized("%A", Locale::fr_FR)
        .to_string();
    let mut text_to_display_chars: Vec<char> = text_to_display.chars().collect();
    text_to_display_chars[0] = text_to_display_chars[0].to_uppercase().nth(0).unwrap();
    let text_to_display: String = text_to_display_chars.into_iter().collect();

    let text_extent = context.text_extents(&text_to_display)?;
    let x_offset = (Wepd7In5BV2::largeur() as f64 - text_extent.width()) / 2.0;
    let y_offset = (Wepd7In5BV2::hauteur() as f64 + text_extent.height()) / 4.;
    context.move_to(x_offset, y_offset);
    context.show_text(&text_to_display)?;

    context.set_font_size(120.0);
    context.set_source_rgb(0., 0., 0.);
    let text_to_display = &Local::now()
        .format_localized("%e %B", Locale::fr_FR)
        .to_string();
    let text_extent = context.text_extents(&text_to_display)?;
    let x_offset = (Wepd7In5BV2::largeur() as f64 - text_extent.width()) / 2.0;
    let y_offset = (Wepd7In5BV2::hauteur() as f64 + text_extent.height()) / 2.;
    context.move_to(x_offset, y_offset);
    context.show_text(text_to_display)?;

    context.set_font_size(120.0);
    context.set_source_rgb(0., 0., 0.);
    let text_to_display = &Local::now()
        .format_localized("%R", Locale::fr_FR)
        .to_string();

    let text_extent = context.text_extents(&text_to_display)?;
    let x_offset = (Wepd7In5BV2::largeur() as f64 - text_extent.width()) / 2.0;
    let y_offset = (Wepd7In5BV2::hauteur() as f64 + text_extent.height() + 120. / 4.) * 3. / 4.;
    context.move_to(x_offset, y_offset);
    context.show_text(text_to_display)?;

    let mut file = File::create("cairo_output.png").expect("Couldn’t create file");
    surface
        .write_to_png(&mut file)
        .expect("Couldn’t write to png");

    drop(context);
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
