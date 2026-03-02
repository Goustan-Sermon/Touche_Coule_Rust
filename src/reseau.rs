use crate::modele::{analyser_saisie, Coordonnee};
use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};

#[derive(Debug, PartialEq)]
pub enum MessageReseau {
    Hello(String),
    Tir(Coordonnee),
    RepAleau,
    RepTouche,
    RepCoule(String),
    RepFin,
}

impl MessageReseau {
    /// Transforme une chaine de caracteres reçue du reseau en MessageReseau
    pub fn parser(texte: &str) -> Option<Self> {
        // split_once(':') coupe le texte en deux au niveau du premier ':'
        let (commande, donnees) = texte.trim().split_once(':')?;

        match commande {
            "HELLO" => Some(MessageReseau::Hello(donnees.to_string())),
            "TIR" => {
                let coord = analyser_saisie(donnees)?; 
                Some(MessageReseau::Tir(coord))
            },
            "REP" => {
                if donnees == "ALEAU" { Some(MessageReseau::RepAleau) }
                else if donnees == "TOUCHE" { Some(MessageReseau::RepTouche) }
                else if donnees == "FIN" { Some(MessageReseau::RepFin) }
                // Si ca commence par COULE:, on recoupe en deux pour extraire le nom
                else if let Some(("COULE", nom_navire)) = donnees.split_once(':') {
                    Some(MessageReseau::RepCoule(nom_navire.to_string()))
                } else {
                    None // Commande REP inconnue
                }
            },
            _ => None, // Commande totalement inconnue
        }
    }

    /// Transforme notre MessageReseau en texte pour l'envoyer sur le reseau
    pub fn vers_chaine(&self) -> String {
        // On ajoute un '\n' a la fin de chaque message, indispensable pour 
        // que les sockets TCP sachent ou s'arrete le message
        match self {
            MessageReseau::Hello(nom) => format!("HELLO:{}\n", nom),
            MessageReseau::Tir(coord) => {
                // Operation inverse : on retransforme x=1 en 'B' et y=1 en '2'
                let lettre = (b'A' + coord.x as u8) as char;
                let chiffre = coord.y + 1;
                format!("TIR:{}{}\n", lettre, chiffre)
            },
            MessageReseau::RepAleau => "REP:ALEAU\n".to_string(),
            MessageReseau::RepTouche => "REP:TOUCHE\n".to_string(),
            MessageReseau::RepCoule(nom) => format!("REP:COULE:{}\n", nom),
            MessageReseau::RepFin => "REP:FIN\n".to_string(),
        }
    }
}

pub fn heberger_partie(port: &str) -> Option<TcpStream> {
    // 0.0.0.0 : j'ecoute sur toutes les cartes reseau de mon ordinateur" (Wifi, Ethernet, reseau local)
    let adresse = format!("0.0.0.0:{}", port);
    
    println!("Ouverture du port {}...", port);

    // TcpListener::bind reserve le port
    let ecouteur = match TcpListener::bind(&adresse) {
        Ok(listener) => listener,
        Err(e) => {
            println!("Erreur : Impossible d'ouvrir le port {}. Est-il déjà utilisé ? ({})", port, e);
            return None;
        }
    };

    println!("En attente d'un adversaire (En écoute sur {})...", adresse);

    // .accept() bloque le programme ici : il s'arrete jusqu'a ce qu'une connexion reseau entrante arrive
    match ecouteur.accept() {
        Ok((flux, adresse_client)) => {
            println!(">>> Connexion établie avec l'adversaire depuis l'IP : {}", adresse_client);
            // On retourne le flux TCP (TcpStream)
            Some(flux)
        }
        Err(e) => {
            println!("Erreur lors de la connexion du client : {}", e);
            None
        }
    }
}

/// Tente de se connecter a un serveur distant
pub fn rejoindre_partie(ip: &str, port: &str) -> Option<TcpStream> {
    let adresse = format!("{}:{}", ip, port);
    println!("Tentative de connexion à l'Amiral adverse sur {}...", adresse);

    // TcpStream::connect bloque jusqu'a ce que la connexion reussisse ou echoue
    match TcpStream::connect(&adresse) {
        Ok(flux) => {
            println!(">>> Connexion réussie ! Le canal de communication est ouvert.");
            Some(flux)
        }
        Err(e) => {
            println!("Erreur : Impossible d'établir le contact ({}).", e);
            None
        }
    }
}

pub fn envoyer_message(flux: &mut TcpStream, message: &MessageReseau) -> Result<(), std::io::Error> {
    // 1. On transforme notre enum en texte (ex: "TIR:B2\n")
    let texte = message.vers_chaine();
    
    // 2. On convertit le texte en octets (bytes) et on l'envoie dans le tuyau
    flux.write_all(texte.as_bytes())?;
    
    // 3. On force l'envoi immediat (pour eviter que le système ne mette en cache)
    flux.flush()?; 
    
    Ok(())
}

pub fn recevoir_message(flux: &mut TcpStream) -> Option<MessageReseau> {
    // On emballe notre flux dans un BufReader.
    let mut lecteur = BufReader::new(flux);
    let mut ligne = String::new();

    // On bloque et on attend qu'une ligne arrive sur le reseau
    match lecteur.read_line(&mut ligne) {
        Ok(0) => {
            // Si on reçoit 0 octet, c'est que l'adversaire a coupe la connexion
            println!("Connexion perdue avec l'adversaire.");
            None
        }
        Ok(_) => {
            // On a recu du texte, on demande a notre parser de le traduire en objet Rust
            MessageReseau::parser(&ligne)
        }
        Err(e) => {
            println!("Erreur de lecture réseau : {}", e);
            None
        }
    }
}