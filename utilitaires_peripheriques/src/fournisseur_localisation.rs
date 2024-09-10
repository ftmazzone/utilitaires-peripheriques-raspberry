use core::str;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use afficher_temperature_utilitaires::peripherique_usb::PeripheriqueUsb;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde::Serialize;
use tokio::{
    io::{self, Interest},
    net::TcpStream,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DonneesLocalisationTpv {
    pub class: String,
    pub device: String,
    pub(crate) mode: i32,
    pub(crate) time: Option<DateTime<Utc>>,
    pub(crate) ept: Option<Decimal>,
    pub(crate) lat: Option<Decimal>,
    pub(crate) lon: Option<Decimal>,
    #[serde(rename = "altHAE")]
    pub(crate) alt_hae: Option<Decimal>,
    #[serde(rename = "altMSL")]
    pub(crate) alt_msl: Option<Decimal>,
    pub(crate) alt: Option<Decimal>,
    pub(crate) epx: Option<Decimal>,
    pub(crate) epy: Option<Decimal>,
    pub(crate) epv: Option<Decimal>,
    pub(crate) magvar: Option<Decimal>,
    pub(crate) speed: Option<Decimal>,
    pub(crate) climb: Option<Decimal>,
    pub(crate) eps: Option<Decimal>,
    pub(crate) epc: Option<Decimal>,
    pub(crate) geoid_sep: Option<Decimal>,
    pub(crate) eph: Option<Decimal>,
    pub(crate) sep: Option<Decimal>,
}

/// Vérifier que la localisation est connue à l'aide d'un client gpsd
pub async fn verifier_localisation(
    arret_demande: Arc<AtomicBool>,
    systeme_localisation_port_usb: &Option<String>,
) -> Option<DonneesLocalisationTpv> {
    log::info!("Démarrage du système de localisation");

    if systeme_localisation_port_usb.is_none() {
        log::info!("Pas de système de localisation configuré");
        return None;
    }

    let systeme_localisation_port_usb = systeme_localisation_port_usb.as_ref().unwrap();

    let mut systeme_localisation_allume =
        match PeripheriqueUsb::changer_etat_usb(&systeme_localisation_port_usb, true) {
            true => true,
            false => {
                log::warn!(
                    "Erreur lors du changement d'état du port usb {systeme_localisation_port_usb}"
                );
                false
            }
        };

    let mut phrase_tpv = None;
    match TcpStream::connect("127.0.0.1:2947").await {
        Ok(flux) => {
            let mut message_initialisation_envoye = false;

            while !arret_demande.load(Ordering::SeqCst) {
                match flux.ready(Interest::READABLE | Interest::WRITABLE).await {
                    Ok(etat_flux) => {
                        if etat_flux.is_read_closed() {
                            log::warn!("Erreur : {} ", etat_flux.is_read_closed());
                        }

                        if etat_flux.is_readable() {
                            let mut donnees_gpsd = vec![0; 3072];

                            match flux.try_read(&mut donnees_gpsd) {
                                Ok(n) => {
                                    log::debug!("lecture {} octets", n);
                                    let (donnees, _) = donnees_gpsd.split_at(n);
                                    let donnees_localisation_tpv: DonneesLocalisationTpv =
                                        serde_json::from_slice(donnees).unwrap_or_default();

                                    if donnees_localisation_tpv.class == "TPV"
                                        && donnees_localisation_tpv.time.is_some()
                                        && donnees_localisation_tpv.lat.is_some()
                                    {
                                        let difference_heure_systeme_vs_systeme_localisation =
                                            Utc::now()
                                                - donnees_localisation_tpv.time.unwrap_or_default();

                                        log::info!(
                                            "Localisation déterminée : {:?}",
                                            donnees_localisation_tpv
                                        );
                                        log::info!(
                                            "Différence entre l'heure du système et l'heure du système de localisation : {} ms",
                                            difference_heure_systeme_vs_systeme_localisation.num_milliseconds()
                                        );

                                        if difference_heure_systeme_vs_systeme_localisation
                                            .num_milliseconds()
                                            .abs()
                                            < 1000
                                        {
                                            log::info!("Heures du système et du système de localisation synchronisées");
                                            phrase_tpv = Some(donnees_localisation_tpv);
                                            break;
                                        }
                                    }
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                    continue;
                                }
                                Err(e) => {
                                    log::warn!("Erreur lors de la lecture du flux {e}");
                                    break;
                                }
                            }
                        }

                        if !message_initialisation_envoye && etat_flux.is_writable() {
                            // Initialiser le client gpsd
                            match flux.try_write(b"?WATCH={\"enable\":true,\"json\":true};") {
                                Ok(n) => {
                                    log::debug!("écriture {} octets", n);
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                                Err(e) => {
                                    log::warn!("Erreur lors de l'écriture du flux {e}");
                                }
                            }
                            message_initialisation_envoye = true;
                        }
                    }
                    Err(err) => log::warn!("Pas de flux disponible {err}"),
                };
            }
        }
        Err(err) => log::warn!("Erreur lors de l'ouverture du flux {err}"),
    };

    if systeme_localisation_allume {
        systeme_localisation_allume =
            match PeripheriqueUsb::changer_etat_usb(&systeme_localisation_port_usb, false) {
                true => false,
                false => {
                    log::warn!(
                    "Erreur lors du changement d'état du port usb {systeme_localisation_port_usb}"
                );
                    false
                }
            };
    }

    log::info!("Etat du port USB du système de localisation {systeme_localisation_allume}");

    phrase_tpv
}
