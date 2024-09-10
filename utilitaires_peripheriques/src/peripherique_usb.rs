use std::{fs, io::Write, path::Path};

pub struct PeripheriqueUsb {}

impl PeripheriqueUsb {
    pub fn changer_etat_usb(adresse_port_usb: &String, allume: bool) -> bool {
        let texte = match allume {
            true => "0",
            false => "1",
        };

        match fs::OpenOptions::new()
            .write(true)
            .open(Path::new(adresse_port_usb).join("disable"))
        {
            Ok(mut f) => match f.write_all(texte.as_bytes()) {
                Ok(_) => {
                    log::info!("Changement état USB {allume} {adresse_port_usb}");
                    true
                }
                Err(err) => {
                    log::warn!("écriture fichier {err} {adresse_port_usb}");
                    false
                }
            },
            Err(err) => {
                log::warn!("ouverture fichier {err} {adresse_port_usb}");
                false
            }
        }
    }
}
