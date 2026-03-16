use crate::modele::{analyser_saisie, Coordonnee};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write, Read};
use std::sync::Arc;
use rustls::{ClientConfig, ServerConfig, StreamOwned, ServerConnection, ClientConnection};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};

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
    println!("Ouverture du port {}...", port);

    let ecouteur = match TcpListener::bind(&adresse) {
        Ok(listener) => listener,
        Err(e) => {
            println!("Erreur : Impossible d'ouvrir le port {}. ({})", port, e);
            return None;
        }
    };

    println!("En attente d'un adversaire (En écoute sur {})...", adresse);

    match ecouteur.accept() {
        Ok((flux_tcp, adresse_client)) => {
            println!(">>> Connexion TCP établie depuis l'IP : {}. Négociation TLS...", adresse_client);
            
            // On active le chiffrement TLS pour sécuriser la communication avec le client
            let (certs, key) = generer_certificat_serveur();
            let config = ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, key)
                .unwrap();

            // On enveloppe le flux TCP basique dans notre armure TLS
            let conn = ServerConnection::new(Arc::new(config)).unwrap();
            let flux_tls = StreamOwned::new(conn, flux_tcp);
            
            println!(">>> Tunnel chiffré établi avec succès !");
            // On retourne le flux masque derriere notre Trait
            Some(Box::new(flux_tls))
        }
        Err(e) => {
            println!("Erreur lors de la connexion du client : {}", e);
            None
        }
    }
}

pub fn rejoindre_partie(ip: &str, port: &str) -> Option<Box<dyn FluxJeu>> {
    let adresse = format!("{}:{}", ip, port);
    println!("Tentative de connexion à l'Amiral adverse sur {}...", adresse);

    match TcpStream::connect(&adresse) {
        Ok(flux_tcp) => {
            println!(">>> Connexion TCP réussie ! Négociation du tunnel TLS...");

            // On active le chiffrement TLS pour sécuriser la communication avec l'hote
            let config = ClientConfig::builder()
                .dangerous() // On autorise les certificats auto-signes
                .with_custom_certificate_verifier(Arc::new(tls_verif::AccepteTout))
                .with_no_client_auth();

            // On s'attend a parler a "localhost" (le nom de notre faux certificat)
            let server_name = ServerName::try_from("localhost").unwrap().to_owned();

            let conn = ClientConnection::new(Arc::new(config), server_name).unwrap();
            let flux_tls = StreamOwned::new(conn, flux_tcp);

            println!(">>> Tunnel chiffré établi avec succès !");
            Some(Box::new(flux_tls))
        }
        Err(e) => {
            println!("Erreur : Impossible d'établir le contact ({}).", e);
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

// Genere un certificat auto-signe et une cle privee a la volee pour l'Hote
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