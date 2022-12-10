use rppal::gpio::{Gpio,OutputPin};

pub struct Eclairage {
      pin:  Option<OutputPin>,
}

impl Eclairage {
    pub fn new(pin: u8) -> Self {
        let gpio = Gpio::new().expect("Gpio new");
        let pin = gpio.get(pin).expect("gpio get").into_output();
        Self {
            pin:Some(pin),
        }
    }

    pub  fn demarrer(&mut self) {
        if self.pin.is_none() {
            return;
        }
        self.pin.as_mut().unwrap().set_high();
        log::debug!("Eclairage allumé : {:?}",self.pin.as_mut().unwrap().is_set_high());
    }

    pub fn arreter( &mut self) {
         self.pin.as_mut().unwrap().set_low();
         log::debug!("Eclairage allumé : {:?}",self.pin.as_mut().unwrap().is_set_high());
    }
}
