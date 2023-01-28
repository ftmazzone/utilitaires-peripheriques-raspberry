pub enum AdresseCapteur {
    I2cAddress,
    Vmel7700DefaultI2cAddress,
}

/// Commandes pour contrôler le capteur
#[derive(Copy, Clone)]
pub(crate) enum Instruction {
    AlsConfig,
    Als,
    AlsWhite,

    // Ambient light sensor gain settings
    AlsGain1,
    AlsGain2,
    AlsGain1_8,
    AlsGain1_4,

    // Ambient light intergration time settings
    Als25MS,
    Als50MS,
    Als100MS,
    Als200MS,
    Als400MS,
    Als800MS,
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
            Instruction::AlsConfig=>0x00,
            Instruction::Als => 0x04,
            Instruction::AlsWhite => 0x05,

            Instruction::AlsGain1 => 0x0,
            Instruction::AlsGain2 => 0x01,
            Instruction::AlsGain1_8 => 0x02,
            Instruction::AlsGain1_4 => 0x03,

            Instruction::Als25MS => 0xC,
            Instruction::Als50MS => 0x8,
            Instruction::Als100MS => 0x0,
            Instruction::Als200MS => 0x1,
            Instruction::Als400MS => 0x2,
            Instruction::Als800MS => 0x3,
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
