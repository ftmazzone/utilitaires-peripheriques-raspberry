use std::cmp;

use rppal::{
    gpio::{InputPin, OutputPin, Gpio},
    spi::{Spi, SlaveSelect, Bus, Mode, Error},
};
use tokio::{time::sleep,time::Duration};

use crate::ecran::instruction::Instruction;

const DISPLAY_WIDTH: usize = 800;
const DISPLAY_HEIGHT: usize = 480;
const BUFFER_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT / 8;

/// Ecran à encre électronique - 7.5inch E-Ink display
/// Modèle : [`800×480, 7.5inch E-Ink display HAT for Raspberry Pi`](https://www.waveshare.com/7.5inch-e-paper-hat.htm)
/// Implémentation python officielle : [epd7in5b_V2.py](https://github.com/waveshare/e-Paper/blob/master/RaspberryPi_JetsonNano/python/lib/waveshare_epd/epd7in5b_V2.py)
pub struct Wepd7In5BV2 {
    spi: Spi,
    dc: OutputPin,
    rst: OutputPin,
    cs: OutputPin,
    busy: InputPin,
    buffer_red: [u8; BUFFER_SIZE],
    buffer_black: [u8; BUFFER_SIZE],
}

impl Wepd7In5BV2 {
    /// Ecran Wepd7In5BV2
    pub fn new(spi_bus:Bus,dc_numero_pin: u8, rst_numero_pin: u8, cs_numero_pin: u8, busy_numero_pin: u8) -> Result<Self,Error> {

        let spi = Spi::new(spi_bus, SlaveSelect::Ss0, 4000000, Mode::Mode0)?;
        let gpio = Gpio::new().unwrap();
        let rst = gpio.get(rst_numero_pin).unwrap().into_output();
        let dc = gpio.get(dc_numero_pin).unwrap().into_output();
        let cs = gpio.get(cs_numero_pin).unwrap().into_output();
        let busy = gpio.get(busy_numero_pin).unwrap().into_input();

       Ok( Self {
            spi: spi,
            dc: dc,
            rst: rst,
            cs: cs,
            busy: busy,
            buffer_red: [0; BUFFER_SIZE],
            buffer_black: [0; BUFFER_SIZE],
        })
    }

    /// Largeur de l'écran
    pub const fn largeur() -> usize {
        DISPLAY_WIDTH
    }

    /// Hauteur de l'écran
    pub const fn hauteur() -> usize {
        DISPLAY_HEIGHT
    }

    /// Réinitialiser l'écran
    pub(crate) async fn reinitialiser(&mut self) -> Result<(), rppal::gpio::Error> {
        self.rst.set_high();
        sleep(Duration::from_millis(200)).await;
        self.rst.set_low();
        sleep(Duration::from_millis(4)).await;
        self.rst.set_high();
        sleep(Duration::from_millis(200)).await;

        Ok(())
    }

    /// Envoyer les instructions à l'écran
    pub(crate) fn envoyer_instruction(
        &mut self,
        commande: Instruction,
        data: &[u8],
    ) -> Result<(), Error> {
        self.dc.set_low();
        self.cs.set_low();
        self.spi.write(&[commande as u8])?;
        self.cs.set_high();

        if data.len() > 0 {
            self.envoyer_donnees(data)?;
        }

        Ok(())
    }

    /// Envoyer les données de la mémoire tampon 
    pub fn envoyer_donnees(&mut self, data: &[u8]) -> Result<(), Error> {
        self.dc.set_high();
        self.cs.set_low();

        let date_len = data.len();
        let mut idx_pixels_sent = 0;
        let multiplier = 4096;

        while idx_pixels_sent < date_len {
            let number_available_pixels_to_send = date_len - idx_pixels_sent;
            let number_pixels_to_send = cmp::min(number_available_pixels_to_send, multiplier);
            self.spi
                .write(&data[idx_pixels_sent..idx_pixels_sent + number_pixels_to_send])?;
            idx_pixels_sent = idx_pixels_sent + number_pixels_to_send;
        }
        self.cs.set_high();
        Ok(())
    }

    /// Attendre que le contrôleur de l'écran soit disponible
    pub async fn est_occupe(&mut self) -> Result<(), Error> {
        let mut i = 0;
        self.envoyer_instruction(Instruction::BusyStatus, &[])?;
        let mut busy = self.busy.is_low();
        while busy && i < 1000 {
            sleep(Duration::from_millis(100)).await;
            self.envoyer_instruction(Instruction::BusyStatus, &[])?;
            busy = self.busy.is_low();
            i += 1;
        }
        if i ==1000{
            log::error!("Attente maximale atteinte")
        }
        Ok(())
    }

    /// Initialiser l'écran
    pub async fn initialiser(&mut self) -> Result<(), Error> {
        log::debug!("Initialisation");
        self.reinitialiser().await.unwrap();

        self.envoyer_instruction(Instruction::PowerSetting, &[0x07, 0x07, 0x3f, 0x3f])?;
        self.envoyer_instruction(Instruction::PowerOn, &[0xF7])?;
        sleep(Duration::from_millis(100)).await;
        self.est_occupe().await?;
        self.envoyer_instruction(Instruction::PanelSetting, &[0x0F])?;
        self.envoyer_instruction(Instruction::Tres, &[0x03, 0x20, 0x01, 0xE0])?;
        self.envoyer_instruction(Instruction::Unknown1, &[0x00])?;
        self.envoyer_instruction(Instruction::VcomAndDataIntervalSetting, &[0x11, 0x07])?;
        self.envoyer_instruction(Instruction::TconSetting, &[0x22])?;
        self.envoyer_instruction(Instruction::Unknown2, &[0x00, 0x00, 0x00, 0x00])?;
        log::debug!("Initialisation terminée");
        Ok(())
    }

    /// Convertir une image RGB565 et la sauvegarder dans la mémoire tampon du programme
    /// L'image n'est pas transfée à l'écran
    pub fn sauvegarder_image_memoire_tampon(&mut self, image: &[u8]) -> Result<(), Error> {

        for i in (0..image.len()).step_by(16) {
            let mut couleur_8pixels_noir: u8 = 0xFF;
            let mut couleur_8pixels_rouge: u8 = 0xFF;
            for j in 0..8 {
                if image[i + j * 2] == 0 && image[i + j * 2 + 1] == 0 {
                    couleur_8pixels_noir &= !(0x80 >> (j % 8));
                } else if image[i + j * 2] == 0 && image[i + j * 2 + 1] != 0 {
                    couleur_8pixels_rouge &= !(0x80 >> (j % 8));
                }
            }
            self.buffer_black[i / 8 / 2] = couleur_8pixels_noir;
            self.buffer_red[i / 8 / 2] = !couleur_8pixels_rouge;
        }

        Ok(())
    }

    /// Attendre que le contrôleur de l'écran soit disponible
    pub async fn eteindre(&mut self) -> Result<(), Error> {
        log::debug!("Extinction");
        self.envoyer_instruction(Instruction::PowerOff, &[])?;
        self.est_occupe().await?;

        self.envoyer_instruction(Instruction::DeepSleep, &[0xA5])?;
        log::debug!("Extinction terminée");
        Ok(())
    }

    /// Mettre à jour l'écran en transférant le contenu de la mémoire tampon vers le contrôleur de l'écran
    pub async fn mettre_a_jour(&mut self) -> Result<(), Error> {

        log::debug!("Mise à jour");
        let buffer_black = self.buffer_black;
        let buffer_red = self.buffer_red;

        self.envoyer_instruction(Instruction::DataStartTransmission1, &[])?;
        self.envoyer_donnees(&buffer_black)?;
        self.envoyer_instruction(Instruction::DataStartTransmission2, &[])?;
        self.envoyer_donnees(&buffer_red)?;
        self.envoyer_instruction(Instruction::DisplayRefresh, &[])?;
        sleep(Duration::from_millis(100)).await;
        self.est_occupe().await?;
        self.eteindre().await?;
        log::debug!("Mise à jour terminée");
        Ok(())
    }

    /// Effacer la mémoire tampon du programme
    pub fn effacer_memoire_tampon(&mut self) -> Result<(), Error> {
        for i in 0..self.buffer_black.len() - 1 {
            self.buffer_black[i] = 0xff;
        }
        for i in 0..self.buffer_red.len() - 1 {
            self.buffer_red[i] = 0x00;
        }
        Ok(())
    }
}
