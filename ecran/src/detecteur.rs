use rppal::gpio::{Level,Gpio,InputPin,Trigger};
use flume::Sender;
pub struct Detecteur {
    pin: Option<InputPin>,
    tx:Sender<bool>
}

impl Detecteur {
    pub fn new(pin: u8,tx:Sender<bool>) -> Self {
        let gpio = Gpio::new().expect("Gpio new");
        let pin = gpio.get(pin).expect("gpio get");
        let pin = pin.into_input();
        Self {
            pin:Some(pin),
            tx
        }
    }

    pub async fn demarrer(&mut self) {
        let tx = self.tx.clone();
        if self.pin.is_none() {
            return;
        }
        self.pin.as_mut().unwrap()
        .set_async_interrupt(Trigger::Both, move |level: Level| {
            let mouvement_detecte= match level {
                Level::Low => false,
                Level::High => true,
            };
            log::debug!("Mouvement détecté : {mouvement_detecte}");
            tx.send(mouvement_detecte).unwrap();
           
        })
        .unwrap();
    }

    pub fn arreter( &mut self) {
         self.pin.as_mut().unwrap().clear_async_interrupt().unwrap();
    }
}
