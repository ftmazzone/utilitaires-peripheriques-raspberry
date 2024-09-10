use flume::Sender;
use rppal::gpio::{Event, Gpio, InputPin, Trigger};
pub struct DetecteurMouvement {
    pin: Option<InputPin>,
    tx: Sender<bool>,
}

impl DetecteurMouvement {
    pub fn new(pin: u8, tx: Sender<bool>) -> Self {
        let gpio = Gpio::new().expect("Gpio new");
        let pin = gpio.get(pin).expect("gpio get");
        let pin = pin.into_input();
        Self { pin: Some(pin), tx }
    }

    pub async fn demarrer(&mut self) {
        let tx = self.tx.clone();
        if self.pin.is_none() {
            return;
        }
        self.pin
            .as_mut()
            .unwrap()
            .set_async_interrupt(Trigger::Both, None, move |event: Event| {
                let mouvement_detecte = match event.trigger {
                    Trigger::FallingEdge => false,
                    Trigger::RisingEdge => true,
                    _ => false,
                };
                log::debug!("Mouvement détecté : {mouvement_detecte}");
                tx.send(mouvement_detecte).unwrap();
            })
            .unwrap();
    }

    pub fn arreter(&mut self) {
        self.pin.as_mut().unwrap().clear_async_interrupt().unwrap();
    }
}
