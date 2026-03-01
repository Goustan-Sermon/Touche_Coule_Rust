use crate::modele::{analyser_saisie, Coordonnee};

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
    /// Transforme une chaîne de caractères reçue du réseau en MessageReseau
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
                // Si ça commence par COULE:, on recoupe en deux pour extraire le nom
                else if let Some(("COULE", nom_navire)) = donnees.split_once(':') {
                    Some(MessageReseau::RepCoule(nom_navire.to_string()))
                } else {
                    None // Commande REP inconnue
                }
            },
            _ => None, // Commande totalement inconnue
        }
    }

    /// Transforme notre MessageReseau en texte pour l'envoyer sur le réseau
    pub fn vers_chaine(&self) -> String {
        // On ajoute un '\n' à la fin de chaque message. C'est indispensable pour 
        // que les sockets TCP sachent où s'arrête le message !
        match self {
            MessageReseau::Hello(nom) => format!("HELLO:{}\n", nom),
            MessageReseau::Tir(coord) => {
                // Opération inverse : on retransforme x=1 en 'B' et y=1 en '2'
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