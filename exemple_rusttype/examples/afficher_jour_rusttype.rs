// Tester cargo run --example tester_ecran

use std::{
    env,
    fs::{self, File},
    io::Cursor,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use chrono::{Local, Locale, Timelike};
use ecran::capteur_luminosite::capteur::Veml7700;
use ecran::{detecteur::Detecteur, eclairage::Eclairage, ecran::ecran::Wepd7In5BV2};
use image::{DynamicImage, Rgb};
use rppal::spi::Bus;
use rusttype::{point, Font, Scale};
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
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
        Ok(mut capteur_luminosite) => match capteur_luminosite.configurer_capteur() {
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

    afficher_image(&mut ecran, String::new()).await?;

    while operationnel.load(Ordering::SeqCst)
        && !rx.is_disconnected()
        && Local::now() - heure_demarrage < chrono::Duration::minutes(30)
    {
        let resultat = timeout(tokio::time::Duration::from_secs(10), rx.recv_async()).await;

        // Afficher l'image toutes les dix minutes ou la luminosité en lux mesurée par le capteur
        if mouvement_detecte && (Local::now().minute() % 5) == 0 && Local::now().second() < 10 {
            let luminosite_lux = format!(
                "{:.2}",
                lire_luminosite(&mut capteur_luminosite)
                    .await
                    .unwrap_or_default()
            );
            afficher_image(&mut ecran, luminosite_lux).await?;
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
    luminosite_lux: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // // Initialise cairo
    // let mut surface = ImageSurface::create(
    //     Format::Rgb16_565,
    //     Wepd7In5BV2::largeur() as i32,
    //     Wepd7In5BV2::hauteur() as i32,
    // )
    // .expect("Impossible d'initialiser la surface");

    // let contexte = Context::new(&mut surface)?;

    // contexte.set_source_rgb(255.0, 255.0, 255.0);
    // contexte.paint()?;

    // if Local::now().minute() as f32 - ((Local::now().minute() as f32) / 10.).floor() * 10. < 1.
    //     || luminosite_lux.eq(&String::new())
    // {
    //     afficher_jour(&contexte)?;
    // } else {
    //     afficher_valeurs_capteurs(&contexte, luminosite_lux)?;
    // }

    // let mut file = File::create("cairo_output.png").expect("Impossible de créer un fichier");
    // surface
    //     .write_to_png(&mut file)
    //     .expect("Couldn’t write to png");

    // drop(contexte);
    // let data = surface.data()?;

    let data = creer_image();

    if ecran.is_some() {
        log::info!("Initialiser");
        ecran.as_mut().unwrap().initialiser().await?;
        ecran.as_mut().unwrap().effacer_memoire_tampon()?;
        ecran
            .as_mut()
            .unwrap()
            .sauvegarder_image_memoire_tampon(&data)?;
    }
    // drop(data);

    if ecran.is_some() {
        log::info!("Afficher l'image");
        ecran.as_mut().unwrap().mettre_a_jour().await?;
    }
    Ok(())
}

fn creer_image() -> Vec<u8> {
    // Load the font
    let font_data = &fs::read("./STIXTwoMath-Regular.ttf").unwrap();
    // This only succeeds if collection consists of one font
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");

    // The font size to use
    let scale = Scale::uniform(64.0);

    // The text to render
    let text = &format!("Png ! {} ⚠ ↗", '\u{237c}'.to_string());

    // Use a dark red colour
    let colour = (255, 255, 0);

    let v_metrics = font.v_metrics(scale);

    // layout the glyphs in a line with 20 pixels padding
    let glyphs: Vec<_> = font
        .layout(text, scale, point(20.0, 20.0 + v_metrics.ascent))
        .collect();

    // work out the layout size
    let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
    let glyphs_width = {
        let min_x = glyphs
            .first()
            .map(|g| g.pixel_bounding_box().unwrap().min.x)
            .unwrap();
        let max_x = glyphs
            .last()
            .map(|g| g.pixel_bounding_box().unwrap().max.x)
            .unwrap();
        (max_x - min_x) as u32
    };

    let couleur_pixel_565 = convertir_rgb_888_en_reg_565(colour);

    // Create a new rgba image with some padding
    let mut image =
        DynamicImage::new_rgb16(Wepd7In5BV2::largeur() as u32, Wepd7In5BV2::hauteur() as u32)
            .to_rgb16();

    // Loop through the glyphs in the text, positing each one on a line

    let mut donnees_image: Vec<u16> = vec![0; Wepd7In5BV2::largeur() * Wepd7In5BV2::hauteur()];
    let mut cpt = 0;
    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            // Draw the glyph into the image per-pixel by using the draw closure
            glyph.draw(|x, y, v| {
                let pixel;
                if v < 0.5 {
                    pixel = [0, 0, 0];
                } else {
                    pixel = [couleur_pixel_565, 0, 0]
                }
                donnees_image[cpt] = pixel[0];
                cpt = cpt + 1;
                image.put_pixel(
                    // Offset the position by the glyph bounding box
                    x + bounding_box.min.x as u32,
                    y + bounding_box.min.y as u32,
                    // Turn the coverage into an alpha value
                    Rgb(pixel),
                )
            });
        }
    }

    // Save the image to a png file
    image.save("image_example.png").unwrap();
    println!("Generated: image_example.png");
    to_bytes(&donnees_image)
}

fn convertir_rgb_888_en_reg_565(couleur: (u8, u8, u8)) -> u16 {
    let rgb_565 = (((couleur.0 & 0b11111000) as u16) << 8)
        + ((couleur.1 & 0b11111100) << 3) as u16
        + (couleur.2 >> 3) as u16;
    rgb_565
}

async fn lire_luminosite(capteur_luminosite: &mut Option<Veml7700>) -> Option<f64> {
    // Mesurer la luminosité
    let luminosite_lux;
    if capteur_luminosite.is_some() {
        let capteur_luminosite = capteur_luminosite.as_mut().unwrap();

        match capteur_luminosite.demarrer() {
            Ok(_) => {}
            Err(err) => {
                log::error!("Erreur lors du démarrage du capteur de luminosité {err}")
            }
        }

        match capteur_luminosite.lire_luminosite_lux().await {
            Ok(valeur) => {
                log::info!("Luminosité mesurée avant configuration automatique {valeur} lux")
            }
            Err(err) => {
                log::error!(
                    "Erreur lors de lecture de luminosité avant configuration automatique {err}"
                );
            }
        }

        log::info!(
            "Configuration avant configuration autmatique gain : {:?} temps intégration : {:?}",
            capteur_luminosite.gain(),
            capteur_luminosite.temps_integration()
        );
        match capteur_luminosite.configurer_automatiquement().await {
            Ok(_) => log::info!(
                "Configuration : {:?} temps intégration : {:?}",
                capteur_luminosite.gain(),
                capteur_luminosite.temps_integration()
            ),
            Err(err) => log::error!(
                "Erreur lors de la configuration automatique du capteur de luminosité {err}"
            ),
        }

        match capteur_luminosite.lire_luminosite_lux().await {
            Ok(valeur) => {
                luminosite_lux = Some(valeur);
                log::info!("Luminosité mesurée {valeur} lux")
            }
            Err(err) => {
                log::error!("Erreur lors de lecture de luminosité {err}");
                luminosite_lux = None;
            }
        }

        match capteur_luminosite.arrêter() {
            Ok(_) => {}
            Err(err) => {
                log::error!("Erreur lors de l'arrêt du capteur de luminosité {err}")
            }
        }
    } else {
        luminosite_lux = None;
    }
    luminosite_lux
}

pub fn to_bytes(input: &[u16]) -> Vec<u8> {
    let mut bytes = vec![0; 2 * input.len()];

    let mut cpt = 0;
    for value in input {
        let pixel = &value.to_be_bytes();
        bytes[cpt] = pixel[0];
        bytes[cpt + 1] = pixel[1];
        cpt = cpt + 2;
    }
    println!("bytes.len() {} input.len() {}", bytes.len(), input.len());

    bytes
}
