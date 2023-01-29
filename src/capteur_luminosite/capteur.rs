use rppal::i2c::I2c;

use crate::capteur_luminosite::instruction::{AdresseCapteur, Instruction};

pub struct Veml7700 {
    i2c: I2c,
    big_endian: bool,
    gain: u8,
    temps_integration: u8,
    persistance: u8,
    interruption_active: u8,
    mode_economie_energie: u8,
}

impl Veml7700 {
    pub fn new() -> Result<Self, rppal::i2c::Error> {
        let mut vmel7700 = Self {
            i2c: I2c::new()?,
            big_endian: cfg!(target_endian = "big"),
            gain: Instruction::AlsGain1.adresse(),
            temps_integration: Instruction::AlsIt100MS.adresse(),
            persistance: Instruction::AlsPers1.adresse(),
            interruption_active: 0,
            mode_economie_energie: Instruction::AlsPowerSaveMode1.adresse(),
        };

        vmel7700
            .i2c
            .set_slave_address(AdresseCapteur::I2cAddress.adresse())?;

        Ok(vmel7700)
    }

    fn configurer_capteur(&mut self) -> Result<(), rppal::i2c::Error> {
        let configuration = (self.gain as u16) << 11
            | (self.temps_integration as u16) << 6
            | (self.persistance as u16) << 4
            | (self.interruption_active as u16) << 1
            | (self.mode_economie_energie as u16) << 0;

        let configuration = match self.big_endian {
            true => configuration.to_be_bytes(),
            false => configuration.to_le_bytes(),
        };
        self.i2c
            .block_write(Instruction::AlsConfig as u8, &configuration)
    }

    pub fn initialiser(&mut self) -> Result<(), rppal::i2c::Error> {
        self.configurer_capteur()?;
        Ok(())
    }

    pub fn configurer_gain(&mut self, gain: u8) {
        self.gain = gain;
    }

    pub fn configurer_temps_integration(&mut self, temps_integration: u8) {
        self.temps_integration = temps_integration;
    }

    pub fn configurer_persistance(&mut self, persistance: u8) {
        self.persistance = persistance;
    }

    pub fn configurer_interruption(&mut self, active: bool) {
        match active {
            false => self.interruption_active = 0x00,
            true => self.interruption_active = 0x01,
        }
    }

    pub fn configurer_mode_economie_energie(&mut self, mode_economie_energie: u8) {
        self.mode_economie_energie = mode_economie_energie;
    }

    pub fn lire_luminosite(&mut self) -> Result<u16, rppal::i2c::Error> {
        let mut tampon = [0u8; 2];
        self.i2c
            .block_read(Instruction::Als.adresse(), &mut tampon)?;
        match self.big_endian {
            true => Ok(u16::from_be_bytes(tampon)),
            false => Ok(u16::from_le_bytes(tampon)),
        }
    }

    pub fn lire_luminosite_blanche(&mut self) -> Result<u16, rppal::i2c::Error> {
        let mut tampon = [0u8; 2];
        self.i2c
            .block_read(Instruction::AlsWhite.adresse(), &mut tampon)?;
        match self.big_endian {
            true => Ok(u16::from_be_bytes(tampon)),
            false => Ok(u16::from_le_bytes(tampon)),
        }
    }

    // pub fn lire_lux(&mut self)->Result<u16,rppal::i2c::Error>{
    //     let factor = get_lux_raw_conversion_factor(it, gain);
    //     let lux = f32::from(raw_als) * f32::from(factor);
    //     if (gain == Gain::OneQuarter || gain == Gain::OneEighth) && lux > 1000.0 {
    //         correct_high_lux(lux) as f32
    //     } else {
    //         lux as f32
    //     }
    // }
}
