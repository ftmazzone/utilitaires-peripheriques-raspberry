use std::time::SystemTime;

use rppal::i2c::I2c;
use tokio::time;

use crate::capteur_luminosite::instruction::{
    AdresseCapteur, Gain, ModeEconomieEnergie, Persistance, Registre,
};

use super::instruction::TempsIntegration;

pub struct Veml7700 {
    i2c: I2c,
    big_endian: bool,
    gain: Gain,
    temps_integration: TempsIntegration,
    persistance: Persistance,
    interruption_active: bool,
    mode_economie_energie: ModeEconomieEnergie,
    derniere_lecture_donnees: SystemTime,
    correction_non_lineaire_resolution: bool,
}

impl Veml7700 {
    pub fn new() -> Result<Self, rppal::i2c::Error> {
        let mut vmel7700 = Self {
            i2c: I2c::new()?,
            big_endian: cfg!(target_endian = "big"),
            gain: Gain::AlsGain1,
            temps_integration: TempsIntegration::AlsIt100MS,
            persistance: Persistance::AlsPers1,
            interruption_active: false,
            mode_economie_energie: ModeEconomieEnergie::AlsPowerSaveMode1,
            derniere_lecture_donnees: SystemTime::now(),
            correction_non_lineaire_resolution: false,
        };

        vmel7700
            .i2c
            .set_slave_address(AdresseCapteur::I2cAddress.adresse())?;

        Ok(vmel7700)
    }

    fn configurer_capteur(&mut self) -> Result<(), rppal::i2c::Error> {
        let configuration = (self.gain.adresse() as u16) << 11
            | (self.temps_integration.adresse() as u16) << 6
            | (self.persistance.adresse() as u16) << 4
            | (self.interruption_active as u16) << 1
            | (self.mode_economie_energie.adresse() as u16) << 0;

        let configuration = match self.big_endian {
            true => configuration.to_be_bytes(),
            false => configuration.to_le_bytes(),
        };
        self.i2c
            .block_write(Registre::AlsConfig as u8, &configuration)
    }

    pub fn initialiser(&mut self) -> Result<(), rppal::i2c::Error> {
        self.configurer_capteur()?;
        Ok(())
    }

    pub fn configurer_gain(&mut self, gain: Gain) {
        self.gain = gain;
    }

    pub fn gain(&self) -> Gain {
        self.gain
    }

    pub fn configurer_temps_integration(&mut self, temps_integration: TempsIntegration) {
        self.temps_integration = temps_integration;
    }

    pub fn temps_integration(&self) -> TempsIntegration {
        self.temps_integration
    }

    pub fn configurer_persistance(&mut self, persistance: Persistance) {
        self.persistance = persistance;
    }

    pub fn configurer_interruption(&mut self, active: bool) {
        self.interruption_active = active;
    }

    pub fn configurer_mode_economie_energie(&mut self, mode_economie_energie: ModeEconomieEnergie) {
        self.mode_economie_energie = mode_economie_energie;
    }

    pub fn demarrer(&mut self) -> Result<(), rppal::i2c::Error> {
        self.mode_economie_energie = ModeEconomieEnergie::AlsPowerSaveMode1;
        self.configurer_capteur()?;
        Ok(())
    }

    pub fn arrêter(&mut self) -> Result<(), rppal::i2c::Error> {
        self.mode_economie_energie = ModeEconomieEnergie::AlsPowerSaveMode2;
        self.configurer_capteur()?;
        Ok(())
    }

    pub async fn attendre_avant_prochaine_lecture(&mut self) {
        let temps_ecoule_derniere_lecture_donnees = self
            .derniere_lecture_donnees
            .elapsed()
            .unwrap_or_default()
            .as_millis() as f64;

        let delai_avant_prochaine_lecture_donnees =
            2. * self.temps_integration.valeur() - temps_ecoule_derniere_lecture_donnees;

        println!(
            "délai {} {} {delai_avant_prochaine_lecture_donnees}",
            self.temps_integration.valeur(),
            temps_ecoule_derniere_lecture_donnees
        );

        if delai_avant_prochaine_lecture_donnees > 0. {
            time::sleep(time::Duration::from_millis(
                delai_avant_prochaine_lecture_donnees as u64,
            ))
            .await;
        }
    }

    pub async fn lire_luminosite(&mut self) -> Result<u16, rppal::i2c::Error> {
        self.attendre_avant_prochaine_lecture().await;
        self.derniere_lecture_donnees = SystemTime::now();

        let mut tampon = [0u8; 2];
        self.i2c.block_read(Registre::Als.adresse(), &mut tampon)?;
        match self.big_endian {
            true => Ok(u16::from_be_bytes(tampon)),
            false => Ok(u16::from_le_bytes(tampon)),
        }
    }

    pub async fn lire_luminosite_blanche(&mut self) -> Result<u16, rppal::i2c::Error> {
        self.attendre_avant_prochaine_lecture().await;
        self.derniere_lecture_donnees = SystemTime::now();

        let mut tampon = [0u8; 2];
        self.i2c
            .block_read(Registre::AlsWhite.adresse(), &mut tampon)?;
        match self.big_endian {
            true => Ok(u16::from_be_bytes(tampon)),
            false => Ok(u16::from_le_bytes(tampon)),
        }
    }

    pub fn resolution(&mut self) -> f64 {
        let resolution_at_max = 0.0036;
        let gain_max: f64 = Gain::AlsGain2.valeur();
        let integration_time_max = TempsIntegration::AlsIt800MS.valeur();

        return resolution_at_max
            * (integration_time_max / self.temps_integration.valeur())
            * (gain_max / self.gain.valeur());
    }

    pub fn activer_correction_non_lineaire_resolution(&mut self, active: bool) {
        self.correction_non_lineaire_resolution = active;
    }

    pub async fn lire_luminosite_lux(&mut self) -> Result<f64, rppal::i2c::Error> {
        let resolution = self.resolution();
        let luminosite = self.lire_luminosite().await? as f64;
        let lux_non_corrige = resolution * luminosite;

        match self.correction_non_lineaire_resolution {
            true => {
                let lux_corrige = (((6.0135e-13 * lux_non_corrige - 9.3924e-9) * lux_non_corrige
                    + 8.1488e-5)
                    * lux_non_corrige
                    + 1.0023)
                    * lux_non_corrige;
                Ok(lux_corrige)
            }
            false => Ok(lux_non_corrige),
        }
    }

    pub async fn configurer_automatiquement(&mut self) -> Result<(), rppal::i2c::Error> {
        self.configurer_gain(Gain::AlsGain1_8);
        self.configurer_temps_integration(TempsIntegration::AlsIt100MS);
        self.configurer_capteur()?;
        self.correction_non_lineaire_resolution = false;

        let mut luminosite = self.lire_luminosite().await?;
        println!("Luminosité 1 {luminosite}");
        if luminosite < 100 {
            while luminosite <= 100
                && !(self.gain == Gain::AlsGain2
                    && self.temps_integration == TempsIntegration::AlsIt800MS)
            {
                println!("Luminosité 2 {luminosite}");
                if self.gain != Gain::AlsGain2 {
                    self.gain = self.gain.suivant();
                } else {
                    if self.temps_integration != TempsIntegration::AlsIt800MS {
                        self.temps_integration = self.temps_integration.suivant();
                    }
                }
                self.configurer_capteur()?;
                luminosite = self.lire_luminosite().await?;
            }
        } else {
            println!("Luminosité 3 {luminosite}");
            self.correction_non_lineaire_resolution = true;
            while luminosite > 10000 && self.temps_integration != TempsIntegration::AlsIt25MS {
                self.temps_integration = self.temps_integration.precedent();
                self.configurer_capteur()?;
                luminosite = self.lire_luminosite().await?;
            }
        }

        Ok(())
    }
}
