# Gestion-Ecran-Eink

[![Construction de l'exemple Gestion-Ecran-Eink](https://github.com/ftmazzone/gestion-ecran-eink/actions/workflows/deploiement.yaml/badge.svg)](https://github.com/ftmazzone/gestion-ecran-eink/actions/workflows/deploiement.yaml)

# Installation

Voir [canvas](https://github.com/Automattic/node-canvas).

# Exemple 

- Allumer l'écran
- Afficher un texte
- Allumer les diodes éclairant l'écran
- Détecter les mouvements devant l'écran

```bash
cargo run --example afficher_jour
```

### Autoriser le contrôle de l'alimentation des ports USB de l'écran et du système de localisation

```bash
# Port dont l'alimentation est contrôlée
adresse_ecran_port_usb_controle="/sys/bus/usb/devices/1-1.4.4:1.0/1-1.4.4-port4/disable"
adresse_systeme_localisation_port_usb_controle="/sys/bus/usb/devices/1-1.4.4:1.0/1-1.4.4-port2/disable"
utilisateur=$USER

# Donner les droits à un utilisateur de couper l'alimentation des ports USB
cat > /etc/udev/rules.d/52-usb.rules <<EOL
SUBSYSTEM=="usb", DRIVER=="hub",
  RUN+="/bin/sh -c \"chown -f root:dialout $adresse_ecran_port_usb_controle || true\""
  RUN+="/bin/sh -c \"chmod -f 660 $adresse_ecran_port_usb_controle || true\""
  RUN+="/bin/sh -c \"chown -f root:dialout $adresse_systeme_localisation_port_usb_controle || true\""
  RUN+="/bin/sh -c \"chmod -f 660 $adresse_systeme_localisation_port_usb_controle || true\""
EOL

udevadm trigger --attr-match=subsystem=usb
cat /etc/udev/rules.d/52-usb.rules
usermod -a -G dialout $utilisateur

# Pour retirer l'utilisateur du groupe
# usermod -r -G dialout $utilisateur

# Vérifier que l'utilisateur possède suffisamment de droits pour couper l'alimentation des ports USB
# Ecran
su $utilisateur -c "echo \"1\" > $adresse_ecran_port_usb_controle ; \
sleep 30 ; \
echo \"0\" > $adresse_ecran_port_usb_controle"

# Système de localisation
su $utilisateur -c "echo \"1\" > $adresse_systeme_localisation_port_usb_controle ; \
sleep 30 ; \
echo \"0\" > $adresse_systeme_localisation_port_usb_controle"
```

# Credits

* [VEML7700 Light Sensor driver for ESP-IDF](https://github.com/kgrozdanovski/veml7700-esp-idf#veml7700-light-sensor-driver-for-esp-idf)
* [Adafruit_CircuitPython_VEML7700](https://github.com/adafruit/Adafruit_CircuitPython_VEML7700/tree/main)
