use crate::modele::{analyser_saisie, Coordonnee};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write, Read};
use std::sync::{Mutex, mpsc, Arc};
use rustls::{ClientConfig, ServerConfig, StreamOwned, ServerConnection, ClientConnection};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

// --- ABSTRACTION ---
pub trait FluxJeu: Read + Write {
    /// Oblige chaque type de connexion a savoir donner l'IP de l'adversaire
    fn adresse_ip(&self) -> IpAddr;
}

// 1. Capacite de lire l'IP sur une connexion TCP basique
impl FluxJeu for TcpStream {
    fn adresse_ip(&self) -> IpAddr {
        self.peer_addr().unwrap().ip()
    }
}

// 2. Capacite de lire l'IP a travers le tunnel TLS du Serveur
impl FluxJeu for StreamOwned<ServerConnection, TcpStream> {
    fn adresse_ip(&self) -> IpAddr {
        self.get_ref().peer_addr().unwrap().ip()
    }
}

// 3. Capacite de lire l'IP a travers le tunnel TLS du Client
impl FluxJeu for StreamOwned<ClientConnection, TcpStream> {
    fn adresse_ip(&self) -> IpAddr {
        self.get_ref().peer_addr().unwrap().ip()
    }
}

// 4. Permet à notre boite polymorphe de relayer la demande d'IP
impl FluxJeu for Box<dyn FluxJeu> {
    fn adresse_ip(&self) -> IpAddr {
        (**self).adresse_ip()
    }
}
// ---------------------------------

#[derive(Debug, PartialEq)]
pub enum MessageReseau {
    Hello(String, String),
    Tir(Coordonnee),
    RepAleau,
    RepTouche,
    RepCoule(String),
    RepFin,
    RepAuthOk,
    RepAuthFail,
}

impl MessageReseau {
    /// Transforme une chaine de caracteres reçue du reseau en MessageReseau
    pub fn parser(texte: &str) -> Option<Self> {
        let (commande, donnees) = texte.trim().split_once(':')?;

        match commande {
            "HELLO" => {
                // On recoupe "donnees" en deux
                if let Some((nom, code)) = donnees.split_once(':') {
                    Some(MessageReseau::Hello(nom.to_string(), code.to_string()))
                } else {
                    None // Si le format n'est pas respecte on rejette
                }
            },
            "TIR" => {
                let coord = analyser_saisie(donnees)?; 
                Some(MessageReseau::Tir(coord))
            },
            "REP" => {
                if donnees == "ALEAU" { Some(MessageReseau::RepAleau) }
                else if donnees == "TOUCHE" { Some(MessageReseau::RepTouche) }
                else if donnees == "FIN" { Some(MessageReseau::RepFin) }
                else if donnees == "AUTH_OK" { Some(MessageReseau::RepAuthOk) }
                else if donnees == "AUTH_FAIL" { Some(MessageReseau::RepAuthFail) }
                else if let Some(("COULE", nom_navire)) = donnees.split_once(':') {
                    Some(MessageReseau::RepCoule(nom_navire.to_string()))
                } else {
                    None 
                }
            },
            _ => None,
        }
    }

    /// Transforme notre MessageReseau en texte pour l'envoyer sur le reseau
    pub fn vers_chaine(&self) -> String {
        match self {
            MessageReseau::Hello(nom, code) => format!("HELLO:{}:{}\n", nom, code),
            
            MessageReseau::Tir(coord) => {
                let lettre = (b'A' + coord.x as u8) as char;
                let chiffre = coord.y + 1;
                format!("TIR:{}{}\n", lettre, chiffre)
            },
            MessageReseau::RepAleau => "REP:ALEAU\n".to_string(),
            MessageReseau::RepTouche => "REP:TOUCHE\n".to_string(),
            MessageReseau::RepCoule(nom) => format!("REP:COULE:{}\n", nom),
            MessageReseau::RepFin => "REP:FIN\n".to_string(),
            MessageReseau::RepAuthOk => "REP:AUTH_OK\n".to_string(),
            MessageReseau::RepAuthFail => "REP:AUTH_FAIL\n".to_string(),
        }
    }
}

pub fn heberger_partie(port: &str) -> Option<Box<dyn FluxJeu>> {
    let adresse = format!("0.0.0.0:{}", port);
    println!("\x1b[1;36m[RÉSEAU]\x1b[0m Ouverture du port {}...", port);

    let ecouteur = match TcpListener::bind(&adresse) {
        Ok(listener) => listener,
        Err(e) => {
            println!("\x1b[1;36m[RÉSEAU]\x1b[0m Erreur : Impossible d'ouvrir le port {}. ({})", port, e);
            return None;
        }
    };

    println!("\x1b[1;36m[RÉSEAU]\x1b[0m En attente d'un adversaire (En écoute sur {})...", adresse);

    match ecouteur.accept() {
        Ok((flux_tcp, adresse_client)) => {
            println!("\x1b[1;36m[RÉSEAU]\x1b[0m Connexion TCP établie depuis l'IP : {}. Négociation TLS...", adresse_client);
            
            // On active le chiffrement TLS pour sécuriser la communication avec le client
            let (certs, key) = generer_certificat_serveur();
            let config = ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, key)
                .unwrap();

            // On enveloppe le flux TCP basique dans notre armure TLS
            let conn = ServerConnection::new(Arc::new(config)).unwrap();
            let flux_tls = StreamOwned::new(conn, flux_tcp);
            
            println!("\x1b[1;33m[TLS]\x1b[0m Tunnel chiffré établi avec succès !");
            // On retourne le flux masque derriere notre Trait
            Some(Box::new(flux_tls))
        }
        Err(e) => {
            println!("\x1b[1;33m[TLS]\x1b[0m Erreur lors de la connexion du client : {}", e);
            None
        }
    }
}

pub fn rejoindre_partie(ip: &str, port: &str) -> Option<Box<dyn FluxJeu>> {

    // Port Knocking : Avant de tenter la connexion normale on doit d'abord frapper 
    // furtivement sur une serie de ports pour deverrouiller le vrai port de jeu
    println!("\x1b[1;35m[INFILTRATION]\x1b[0m Exécution de la séquence de frappe furtive...");
    let ports_secrets = [7777, 8888, 9999];
    
    for p in ports_secrets {
        let adresse_toc = format!("{}:{}", ip, p);
        
        // On se connecte tres brievement
        if let Ok(stream) = TcpStream::connect(&adresse_toc) {
            println!(" -> Toc toc sur le port {}...", p);
            // et on referme la connexion immediatement
            drop(stream); 
        }
        
        // Pause de 50 millisecondes pour etre sur que les paquets TCP 
        // arrivent bien dans le bon ordre sur le reseau
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    println!("\x1b[1;35m[INFILTRATION]\x1b[0m Séquence terminée, tentative d'accès au port réel...");

    // On attend que le Gardien du serveur ait le temps de nettoyer ses threads et de liberer le port de jeu
    std::thread::sleep(std::time::Duration::from_millis(300));

    let adresse = format!("{}:{}", ip, port);
    println!("\x1b[1;36m[RÉSEAU]\x1b[0m Tentative de connexion à l'Amiral adverse sur {}...", adresse);

    match TcpStream::connect(&adresse) {
        Ok(flux_tcp) => {
            println!("\x1b[1;36m[RÉSEAU]\x1b[0m Connexion TCP réussie ! Négociation du tunnel TLS...");

            // On active le chiffrement TLS pour sécuriser la communication avec l'hote
            let config = ClientConfig::builder()
                .dangerous() // On autorise les certificats auto-signes
                .with_custom_certificate_verifier(Arc::new(tls_verif::AccepteTout))
                .with_no_client_auth();

            // On s'attend a parler a "localhost" (le nom de notre faux certificat)
            let server_name = ServerName::try_from("localhost").unwrap().to_owned();

            let conn = ClientConnection::new(Arc::new(config), server_name).unwrap();
            let flux_tls = StreamOwned::new(conn, flux_tcp);

            println!("\x1b[1;33m[TLS]\x1b[0m Tunnel chiffré établi avec succès !");
            Some(Box::new(flux_tls))
        }
        Err(e) => {
            println!("\x1b[1;36m[RÉSEAU]\x1b[0m Erreur : Impossible d'établir le contact ({}).", e);
            None
        }
    }
}

pub fn envoyer_message(flux: &mut dyn FluxJeu, message: &MessageReseau) -> Result<(), std::io::Error> {
    // 1. On transforme notre enum en texte (ex: "TIR:B2\n")
    let texte = message.vers_chaine();
    
    // 2. On convertit le texte en octets (bytes) et on l'envoie dans le tuyau
    flux.write_all(texte.as_bytes())?;
    
    // 3. On force l'envoi immediat (pour eviter que le système ne mette en cache)
    flux.flush()?; 
    
    Ok(())
}

pub fn recevoir_message(flux: &mut dyn FluxJeu) -> Option<MessageReseau> {

    let mut reader = BufReader::new(flux).take(64); 
    let mut ligne = String::new();

    match reader.read_line(&mut ligne) {
        Ok(0) => None, // La connexion a ete fermee proprement
        Ok(_) => {
            // si on a rempli les 64 octets et qu'il n'y a toujours pas de retour à la ligne c'est une attaque DoS ou un paquet corrompu 
            if ligne.len() == 64 && !ligne.ends_with('\n') {
                println!("ALERTE SÉCURITÉ : Tentative de Buffer Overflow détectée et bloquée !");
                return None; // On rejette le paquet malveillant
            }
            
            // Si tout va bien on parse le message normalement
            MessageReseau::parser(ligne.trim())
        }
        Err(_) => None, // Erreur de lecture
    }
}

/// Genere un certificat auto-signe et une cle privee a la volee pour l'Hote
fn generer_certificat_serveur() -> (Vec<CertificateDer<'static>>, PrivateKeyDer<'static>) {
    // On genere un certificat pour "localhost"
    let subject_alt_names = vec!["localhost".to_string()];
    
    // rcgen fait le travail cryptographique 
    let cert = rcgen::generate_simple_self_signed(subject_alt_names).unwrap();
    
    // On extrait le certificat public et la cle privee au format attendu par rustls
    let cert_der = cert.cert.into();
    
    // On extrait le certificat public et la cle privee au format attendu par rustls
    let key_der = PrivateKeyDer::Pkcs8(cert.signing_key.serialize_der().into());
    
    (vec![cert_der], key_der)
}

/// Port Knocking : On ecoute sur 3 ports et bloque le programme tant que 
/// la combinaison (7777 -> 8888 -> 9999) n'est pas effectuee dans le bon ordre
pub fn attendre_port_knocking() -> Result<(), String> {
    println!("\n\x1b[1;35m[GARDIEN]\x1b[0m Activation du mode Furtif. Le port 3333 est masqué.");
    println!("\x1b[1;35m[GARDIEN]\x1b[0m En attente du signal (Toc-Toc sur 7777, 8888, 9999)...");

    let (tx, rx) = mpsc::channel();
    let progression = Arc::new(Mutex::new(HashMap::new()));
    let ports_secrets = [7777, 8888, 9999];

    // flag d'arret partage entre tous les threads
    let stop_flag = Arc::new(AtomicBool::new(false));

    let mut ecouteurs = Vec::new();
    for &port in &ports_secrets {
        match TcpListener::bind(format!("0.0.0.0:{}", port)) {
            Ok(l) => ecouteurs.push(l),
            Err(_) => return Err(format!("Le port {} est déjà utilisé. Avez-vous une autre partie en cours ?", port)),
        }
    }

    for (etape, ecouteur) in ecouteurs.into_iter().enumerate() {
        let progression_clone = Arc::clone(&progression);
        let tx_clone = tx.clone();
        let stop_flag_clone = Arc::clone(&stop_flag);
        let etape_requise = etape as u8;

        thread::spawn(move || {
            // On passe l'ecouteur en mode non-bloquant
            // Au lieu de figer le thread en attendant une connexion il verifie en continu
            ecouteur.set_nonblocking(true).unwrap();
            
            // Le thread tourne tant que le stop_flag est sur 'false'
            while !stop_flag_clone.load(Ordering::Relaxed) {
                match ecouteur.accept() {
                    Ok((stream, _)) => {
                        let ip = stream.peer_addr().unwrap().ip();
                        let mut registre = progression_clone.lock().unwrap();
                        let niveau_actuel = registre.entry(ip).or_insert(0);
                        
                        if *niveau_actuel == etape_requise {
                            *niveau_actuel += 1;
                            if *niveau_actuel == 3 {
                                let _ = tx_clone.send(ip);
                            }
                        } else {
                            *niveau_actuel = 0;
                        }
                        drop(stream);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // Personne ne frappe a la porte on dort 50ms pour ne pas epuiser le processeur puis on reverifie
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(_) => {}
                }
            }
        }); // Des qu'il sort du while le thread meurt et libere son port
    }

    // Le programme principal attend que la sequence soit validee
    let ip_validee = rx.recv().unwrap();
    
    // La sequence est bonne : On arrete les 3 threads
    stop_flag.store(true, Ordering::Relaxed);
    
    // On laisse 100 millisecondes aux threads pour voir le flag s'eteindre et relacher les ports
    thread::sleep(Duration::from_millis(100));
    
    println!("\x1b[1;35m[GARDIEN]\x1b[0m Séquence parfaite de {} ! Déverrouillage du vrai port de jeu...", ip_validee);
    Ok(())
}

// --- MODULE DE SECURITE POUR ACCEPTER LE CERTIFICAT AUTO-SIGNE ---
mod tls_verif {
    use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
    use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
    use rustls::{DigitallySignedStruct, SignatureScheme, Error};

    #[derive(Debug)]
    pub struct AccepteTout;

    impl ServerCertVerifier for AccepteTout {
        fn verify_server_cert(
            &self,
            _end_entity: &CertificateDer<'_>,
            _intermediates: &[CertificateDer<'_>],
            _server_name: &ServerName<'_>,
            _ocsp_response: &[u8],
            _now: UnixTime,
        ) -> Result<ServerCertVerified, Error> {
            // On valide tous les certificats
            Ok(ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &CertificateDer<'_>,
            _dss: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, Error> {
            // On force l'acceptation de la signature sans faire de calculs
            Ok(HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &CertificateDer<'_>,
            _dss: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, Error> {
            // On force l'acceptation de la signature sans faire de calculs
            Ok(HandshakeSignatureValid::assertion())
        }

        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            // On annonce supporter les algorithmes de base pour que la connexion s'etablisse
            vec![
                SignatureScheme::RSA_PKCS1_SHA256,
                SignatureScheme::ECDSA_NISTP256_SHA256,
                SignatureScheme::RSA_PSS_SHA256,
                SignatureScheme::ED25519,
            ]
        }
    }
}