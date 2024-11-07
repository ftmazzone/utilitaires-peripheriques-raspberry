use std::{env, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use utilitaires_peripheriques::fournisseur_localisation;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    
    let arret_demande = Arc::new(AtomicBool::new(false));
    let arret_demande_clone = arret_demande.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        log::info!("Lecture de la localisation : arrêt demandé");
        arret_demande_clone.store(true, Ordering::SeqCst);
    });
    let arret_demande_clone = arret_demande.clone();

    let _ = fournisseur_localisation::verifier_localisation(
        arret_demande_clone,
        &Some("/sys/bus/usb/devices/1.1:1.0/1-1-port1".to_string()),
    )
    .await;
    Ok(())
}
