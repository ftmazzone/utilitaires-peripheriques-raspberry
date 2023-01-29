use rppal::i2c::I2c;

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

    pub fn configurer_temps_integration(&mut self, temps_integration: TempsIntegration) {
        self.temps_integration = temps_integration;
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

    pub fn lire_luminosite(&mut self) -> Result<u16, rppal::i2c::Error> {
        let mut tampon = [0u8; 2];
        self.i2c.block_read(Registre::Als.adresse(), &mut tampon)?;
        match self.big_endian {
            true => Ok(u16::from_be_bytes(tampon)),
            false => Ok(u16::from_le_bytes(tampon)),
        }
    }

    pub fn lire_luminosite_blanche(&mut self) -> Result<u16, rppal::i2c::Error> {
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
        let gain_max: f64 = 2.;
        let integration_time_max = 800.;

        if Gain::valeur(self.gain) == gain_max
            && TempsIntegration::valeur(self.temps_integration) == integration_time_max
        {
            return resolution_at_max;
        }
        return resolution_at_max
            * (integration_time_max / TempsIntegration::valeur(self.temps_integration)) as f64
            * (gain_max / Gain::valeur(self.gain)) as f64;
    }

    pub fn lire_luminosite_lux(&mut self) -> Result<f64, rppal::i2c::Error> {
        let resolution = self.resolution();
        let luminosite = self.lire_luminosite()? as f64;
        Ok(resolution * luminosite)
    }
}
