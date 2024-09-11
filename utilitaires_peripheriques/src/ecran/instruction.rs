/// Commandes pour contrôler l'affichage de l'écran
#[derive(Copy, Clone)]
pub(crate) enum Instruction {
    PowerSetting = 0x01,
    PowerOn = 0x04,
    BusyStatus = 0x71,
    PanelSetting = 0x00,
    Tres = 0x61,
    Unknown1 = 0x15,
    VcomAndDataIntervalSetting = 0x50,
    TconSetting = 0x60,
    Unknown2=0x65,
    PowerOff=0x02,
    DeepSleep=0x07,
    DataStartTransmission1=0x10,
    DataStartTransmission2=0x13,
    DisplayRefresh=0x12
}
