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
use image::{DynamicImage, ImageBuffer, Rgb};
use rppal::spi::Bus;
use rusttype::{point, Font, PositionedGlyph, Scale};
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
    let data = match Local::now().minute() as f32
        - ((Local::now().minute() as f32) / 10.).floor() * 10.
        < 1.
        || luminosite_lux.eq(&String::new())
    {
        true => afficher_jour()?,
        false => afficher_jour()?,
    };

    if ecran.is_some() {
        log::info!("Initialiser");
        ecran.as_mut().unwrap().initialiser().await?;
        ecran.as_mut().unwrap().effacer_memoire_tampon()?;
        ecran
            .as_mut()
            .unwrap()
            .sauvegarder_image_memoire_tampon(&data)?;
    }

    if ecran.is_some() {
        log::info!("Afficher l'image");
        ecran.as_mut().unwrap().mettre_a_jour().await?;
    }
    Ok(())
}

fn creer_glyphe_texte(
    police: Font,
    taille_police: Scale,
    texte: String,
) -> (Vec<PositionedGlyph>, u32, u32) {
    let v_metriques = police.v_metrics(taille_police);

    let glyphes: Vec<PositionedGlyph> = police
        .layout(&texte, taille_police, point(0., 0. + v_metriques.ascent))
        .collect();

    let hauteur = (v_metriques.ascent - v_metriques.descent).ceil() as u32;
    let largeur = {
        let min_x = glyphes
            .first()
            .map(|g| g.pixel_bounding_box().unwrap().min.x)
            .unwrap();
        let max_x = glyphes
            .last()
            .map(|g| g.pixel_bounding_box().unwrap().max.x)
            .unwrap();
        (max_x - min_x) as u32
    };
    (glyphes, hauteur, largeur)
}

fn dessiner_glpyhe(
    glyphes: Vec<PositionedGlyph>,
    couleur: (u8, u8, u8),
    hauteur: u32,
    largeur: u32,
    donnees_rgb565: &mut [u16],
) {
    let couleur_pixel_565 = convertir_rgb_888_en_reg_565(couleur);

    // Create a new rgba image with some padding
    let mut image =
        DynamicImage::new_rgb16(Wepd7In5BV2::largeur() as u32, Wepd7In5BV2::hauteur() as u32)
            .to_rgb16();

    // Loop through the glyphs in the text, positing each one on a line

    for glyphe in glyphes {
        if let Some(bounding_box) = glyphe.pixel_bounding_box() {
            // Draw the glyph into the image per-pixel by using the draw closure
            glyphe.draw(|x, y, v| {
                let pixel;
                if v < 0.5 {
                    pixel = 65535;
                } else {
                    pixel = couleur_pixel_565
                }
                let y_pixel = y as i32 + bounding_box.min.y + hauteur as i32;
                let x_pixel = x as i32 + bounding_box.min.x + largeur as i32;

                if !(y_pixel < 0
                    || x_pixel < 0
                    || y_pixel >= Wepd7In5BV2::hauteur() as i32
                    || x_pixel >= Wepd7In5BV2::largeur() as i32)
                {
                    donnees_rgb565[y_pixel as usize * Wepd7In5BV2::largeur() + x_pixel as usize] =
                        pixel;
                }
            });
        }
    }
}

fn convertir_rgb_888_en_reg_565(couleur: (u8, u8, u8)) -> u16 {
    let rgb_565 = (((couleur.0 & 0b11111000) as u16) << 8)
        + ((couleur.1 & 0b11111100) << 3) as u16
        + (couleur.2 >> 3) as u16;
    rgb_565
}

fn afficher_jour() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    log::info!("Afficher le jour courant");
    let couleur = (255, 0, 0);
    let fichier_police = &fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf").unwrap();
    let police = Font::try_from_bytes(fichier_police).unwrap();
    let taille_police = Scale::uniform(140.);

    let mut donnees_rgb565: Vec<u16> =
        vec![65535; Wepd7In5BV2::largeur() as usize * Wepd7In5BV2::hauteur() as usize];

    // Jour
    let texte_a_afficher = &Local::now()
        .format_localized("%A", Locale::fr_FR)
        .to_string();
    let mut texte_a_afficher_characteres: Vec<char> = texte_a_afficher.chars().collect();
    texte_a_afficher_characteres[0] = texte_a_afficher_characteres[0]
        .to_uppercase()
        .nth(0)
        .unwrap();
    let texte_a_afficher: String = texte_a_afficher_characteres.into_iter().collect();

    let (glyphes, hauteur, largeur) = creer_glyphe_texte(police, taille_police, texte_a_afficher);

    dessiner_glpyhe(
        glyphes,
        couleur,
        Wepd7In5BV2::hauteur() as u32 / 5 - hauteur / 2,
        Wepd7In5BV2::largeur() as u32 / 2 - largeur / 2,
        &mut donnees_rgb565,
    );

    let couleur = (0, 0, 0);
    let police = Font::try_from_bytes(fichier_police).unwrap();
    let texte_a_afficher = Local::now()
        .format_localized("%e %B", Locale::fr_FR)
        .to_string();
    let (glyphes, hauteur, largeur) = creer_glyphe_texte(police, taille_police, texte_a_afficher);
    dessiner_glpyhe(
        glyphes,
        couleur,
        Wepd7In5BV2::hauteur() as u32 / 2 - hauteur / 2,
        Wepd7In5BV2::largeur() as u32 / 2 - largeur / 2,
        &mut donnees_rgb565,
    );

    let police = Font::try_from_bytes(fichier_police).unwrap();
    let texte_a_afficher = Local::now()
        .format_localized("%R", Locale::fr_FR)
        .to_string();
    let (glyphes, hauteur, largeur) = creer_glyphe_texte(police, taille_police, texte_a_afficher);
    dessiner_glpyhe(
        glyphes,
        couleur,
        Wepd7In5BV2::hauteur() as u32 * 4 / 5 - hauteur / 2,
        Wepd7In5BV2::largeur() as u32 / 2 - largeur / 2,
        &mut donnees_rgb565,
    );

    let image = ImageBuffer::from_fn(
        Wepd7In5BV2::largeur() as u32,
        Wepd7In5BV2::hauteur() as u32,
        |x, y| {
            let pixel = donnees_rgb565[(y * Wepd7In5BV2::largeur() as u32 + x) as usize];

            let bleu = ((pixel & 0x001F) << 3) as u8;
            let vert = ((pixel & 0x07E0) >> 3) as u8;
            let rouge = ((pixel & 0xF800) >> 8) as u8;

            image::Rgb::<u8>([rouge, vert, bleu])
        },
    );

    image.save("image_example.png").unwrap();

    let donnees = convertir_vec_u16_vers_vec_u8(&donnees_rgb565);
    let a = donnees.len();
    Ok(donnees)
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

pub fn convertir_vec_u16_vers_vec_u8(input: &[u16]) -> Vec<u8> {
    let mut bytes = vec![0; 2 * input.len()];

    let mut cpt = 0;
    for value in input {
        let pixel = &value.to_le_bytes();
        bytes[cpt] = pixel[0];
        bytes[cpt + 1] = pixel[1];
        cpt = cpt + 2;
    }

    bytes
}
