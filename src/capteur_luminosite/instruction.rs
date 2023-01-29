pub enum AdresseCapteur {
    I2cAddress,
    Vmel7700DefaultI2cAddress,
}

/// Commandes pour contrôler le capteur
#[derive(Copy, Clone)]
pub enum Instruction {
    AlsConfig,
    Als,
    AlsWhite,

    // Ambient light sensor gain settings
    AlsGain1,
    AlsGain2,
    AlsGain1_8,
    AlsGain1_4,

    // Ambient light intergration time settings
    AlsIt25MS,
    AlsIt50MS,
    AlsIt100MS,
    AlsIt200MS,
    AlsIt400MS,
    AlsIt800MS,

    // Persistence
    AlsPers1,
    AlsPers2,
    AlsPers4,
    AlsPers8,

    AlsPowerSaveMode1,
    AlsPowerSaveMode2,
    AlsPowerSaveMode3,
    AlsPowerSaveMode4
}

///Gain value integers
#[derive(Copy, Clone)]
pub(crate) enum GainValues {
    AlsGain1,
    AlsGain2,
    AlsGain1_4,
    AlsGain1_8,
}

///Integration time value integers
#[derive(Copy, Clone)]
pub(crate) enum IntegrationTimeValues {
    Als25MS,
    Als50MS,
    Als100MS,
    Als200MS,
    Als400MS,
    Als800MS,
}

impl AdresseCapteur {
    pub fn adresse(self) -> u16 {
        match self {
            AdresseCapteur::I2cAddress => 0x10,
            AdresseCapteur::Vmel7700DefaultI2cAddress => 0x10,
        }
    }
}

impl Instruction {
    pub(crate) fn adresse(self) -> u8 {
        match self {
            Instruction::AlsConfig => 0x00,
            Instruction::Als => 0x04,
            Instruction::AlsWhite => 0x05,

            Instruction::AlsGain1 => 0x0,
            Instruction::AlsGain2 => 0x01,
            Instruction::AlsGain1_8 => 0x02,
            Instruction::AlsGain1_4 => 0x03,

            Instruction::AlsIt25MS => 0x0C,
            Instruction::AlsIt50MS => 0x08,
            Instruction::AlsIt100MS => 0x00,
            Instruction::AlsIt200MS => 0x01,
            Instruction::AlsIt400MS => 0x02,
            Instruction::AlsIt800MS => 0x03,

            Instruction::AlsPers1 => 0x00,
            Instruction::AlsPers2 => 0x01,
            Instruction::AlsPers4 => 0x02,
            Instruction::AlsPers8 => 0x03,

            Instruction::AlsPowerSaveMode1 => 0x00,
            Instruction::AlsPowerSaveMode2 => 0x01,
            Instruction::AlsPowerSaveMode3 => 0x02,
            Instruction::AlsPowerSaveMode4 => 0x03,
        }
    }
}

impl GainValues {
    pub(crate) fn adresse(self) -> f32 {
        match self {
            GainValues::AlsGain1 => 1.,
            GainValues::AlsGain2 => 2.,
            GainValues::AlsGain1_4 => 0.25,
            GainValues::AlsGain1_8 => 0.125,
        }
    }
}

impl IntegrationTimeValues {
    pub(crate) fn adresse(self) -> u16 {
        match self {
            IntegrationTimeValues::Als25MS => 25,
            IntegrationTimeValues::Als50MS => 50,
            IntegrationTimeValues::Als100MS => 100,
            IntegrationTimeValues::Als200MS => 200,
            IntegrationTimeValues::Als400MS => 400,
            IntegrationTimeValues::Als800MS => 800,
        }
    }
}
