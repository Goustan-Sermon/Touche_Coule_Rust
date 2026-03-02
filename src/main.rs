mod modele;
mod reseau;

use modele::{analyser_saisie, Coordonnee, Grille, Navire, Orientation, ResultatTir};
use reseau::{envoyer_message, heberger_partie, recevoir_message, rejoindre_partie, MessageReseau};
use std::io::{self, Write};

fn main() {
    nettoyer_ecran();
    println!("=====================================");
    println!("        BATAILLE NAVALE RÉSEAU       ");
    println!("=====================================\n");

    // 1. Demander le nom du commandant local
    print!("Entrez votre nom de Commandant : ");
    io::stdout().flush().unwrap();
    let mut mon_nom = String::new();
    io::stdin().read_line(&mut mon_nom).unwrap();
    let mon_nom = mon_nom.trim().to_string();

    let mut flux_tcp; 
    let est_hote;

    // 2. Le menu de connexion
    loop {
        println!("\n1. Héberger une partie");
        println!("2. Rejoindre une partie");
        print!("Choix : ");
        io::stdout().flush().unwrap();
        let mut choix = String::new();
        io::stdin().read_line(&mut choix).unwrap();

        match choix.trim() {
            "1" => {
                if let Some(flux) = heberger_partie("3333") {
                    flux_tcp = flux;
                    est_hote = true;
                    break;
                }
            }
            "2" => {
                print!("Adresse IP du serveur : ");
                io::stdout().flush().unwrap();
                let mut ip = String::new();
                io::stdin().read_line(&mut ip).unwrap();
                
                if let Some(flux) = rejoindre_partie(ip.trim(), "3333") {
                    flux_tcp = flux;
                    est_hote = false;
                    break;
                }
            }
            _ => println!("Choix invalide."),
        }
    }

    // 3. L'Echange des pseudos (Handshake)
    println!("\n>>> Échange des données tactiques avec l'adversaire...");
    let message_hello = MessageReseau::Hello(mon_nom.clone());
    let _ = envoyer_message(&mut flux_tcp, &message_hello);

    let nom_adversaire = match recevoir_message(&mut flux_tcp) {
        Some(MessageReseau::Hello(nom)) => nom,
        _ => "Adversaire Inconnu".to_string(),
    };

    println!(">>> Connexion sécurisée avec l'Amiral {} !", nom_adversaire.to_uppercase());

    // 4. La phase de placement (Chacun le fait de son cote localement)
    let mut ma_grille = Grille::new();
    let mut radar = Grille::new(); // Grille vide pour noter nos tirs
    
    phase_placement(&mut ma_grille, &mon_nom);

    println!("\nEn attente que l'Amiral {} termine son déploiement...", nom_adversaire);
    
    let mut mon_tour = est_hote; 

    println!("\n=====================================");
    println!("          DÉBUT DU COMBAT !          ");
    println!("=====================================");

    loop {
        if mon_tour {
            println!("\n=====================================");
            println!("           A VOTRE TOUR !            ");
            println!("=====================================");
            println!("\n--- RADAR (VOS TIRS) ---");
            radar.afficher(false); // On affiche les X et O de nos precedents tirs

            let cible = loop {
                print!("\nCoordonnées de tir (ex: B2) : ");
                io::stdout().flush().unwrap();
                let mut saisie = String::new();
                io::stdin().read_line(&mut saisie).unwrap();

                if let Some(c) = analyser_saisie(&saisie) {
                    break c;
                }
                println!("Coordonnées invalides !");
            };

            // 1. On envoie la coordonnee a l'adversaire
            let _ = envoyer_message(&mut flux_tcp, &MessageReseau::Tir(cible));
            println!(">>> Tir envoyé ! En attente du rapport de dégâts...");

        
            // 2. On attend sa reponse pour mettre a jour notre radar
            match recevoir_message(&mut flux_tcp) {
                Some(MessageReseau::RepAleau) => {
                    println!("Résultat : Plouf... C'est dans l'eau.");
                    radar.cases[cible.y][cible.x].etat = modele::EtatCase::Aleau;
                }
                Some(MessageReseau::RepTouche) => {
                    println!("Résultat : BOUM ! Vous avez touché un navire !");
                    radar.cases[cible.y][cible.x].etat = modele::EtatCase::Touche;
                }
                Some(MessageReseau::RepCoule(nom)) => {
                    println!("Résultat : TOUCHÉ ET COULÉ ! Vous avez détruit le {} !", nom);
                    radar.cases[cible.y][cible.x].etat = modele::EtatCase::Touche;
                }
                Some(MessageReseau::RepFin) => {
                    println!("\n=================================================");
                    println!("VICTOIRE TOTAL ! La flotte de {} est détruite !", nom_adversaire);
                    println!("=================================================");
                    radar.cases[cible.y][cible.x].etat = modele::EtatCase::Touche;
                    radar.afficher(false);
                    break; // Fin du jeu
                }
                _ => println!("Erreur réseau inattendue."),
            }
            println!("\n--- RADAR (VOS TIRS) ---");
            radar.afficher(false);
            mon_tour = false; // Fin de mon tour

        } else {
            println!("\n>>> En attente de l'attaque de {}...", nom_adversaire);

            match recevoir_message(&mut flux_tcp) {
                Some(MessageReseau::Tir(coord)) => {
                    nettoyer_ecran();
                    // On traduit la coordonnee pour l'affichage (ex: x=1 -> 'B')
                    let lettre = (b'A' + coord.x as u8) as char;
                    println!("ALERTE ! Tir ennemi détecté en {}{} !", lettre, coord.y + 1);

                    // On encaisse le tir sur notre vraie grille
                    let resultat = ma_grille.tirer(coord);

                    // On verifie si ce tir nous a acheve
                    if ma_grille.flotte_coulee() {
                        let _ = envoyer_message(&mut flux_tcp, &MessageReseau::RepFin);
                        println!("\n=================================================");
                        println!("DÉFAITE... Toute votre flotte a été anéantie.");
                        println!("=================================================");
                        ma_grille.afficher(false);
                        break; // Fin du jeu !
                    }

                    // Sinon on renvoie le résultat normal à l'adversaire
                    let reponse = match resultat {
                        ResultatTir::Aleau => MessageReseau::RepAleau,
                        ResultatTir::Touche => MessageReseau::RepTouche,
                        ResultatTir::Coule(nom) => MessageReseau::RepCoule(nom),
                        // Si on tire deux fois au meme endroit ou hors limite on dit "A l'eau" pour simplifier la logique reseau
                        _ => MessageReseau::RepAleau, 
                    };

                    let _ = envoyer_message(&mut flux_tcp, &reponse);
                    
                    println!("\n--- ÉTAT DE VOTRE FLOTTE ---");
                    ma_grille.afficher(false); // On regarde les degats
                }
                None => {
                    println!("L'adversaire s'est déconnecté.");
                    break;
                }
                _ => println!("Message inattendu pendant le tour adverse."),
            }
            mon_tour = true; // L'adversaire a fini
        }
    }
}

fn demander_orientation() -> Orientation {
    loop {
        print!("Orientation (H pour Horizontal, V pour Vertical) : ");
        io::stdout().flush().unwrap();
        
        let mut saisie = String::new();
        io::stdin().read_line(&mut saisie).expect("Erreur de lecture");

        // On nettoie la saisie et on verifie
        match saisie.trim().to_uppercase().as_str() {
            "H" => return Orientation::Horizontal,
            "V" => return Orientation::Vertical,
            _ => println!("Saisie invalide. Veuillez taper 'H' ou 'V'."),
        }
    }
}

fn phase_placement(grille: &mut Grille, nom_joueur: &str) {
    // On definit la flotte standard
    let flotte_a_placer = [
        ("Porte-avions", 5),
        ("Croiseur", 4),
        ("Contre-torpilleur", 3),
        ("Sous-marin", 3),
        ("Torpilleur", 2),
    ];

    println!("\n=====================================");
    println!("   PHASE DE PLACEMENT : AMIRAL {}   ", nom_joueur.to_uppercase());
    println!("=====================================");

    for (nom, taille) in flotte_a_placer.iter() {
        loop {
            println!("\n--- VOTRE CARTE ACTUELLE ---");
            grille.afficher(false); // On met "false" car le joueur doit voir ses propres bateaux
            
            println!("\nAmiral, où voulez-vous placer le {} (Taille : {}) ?", nom, taille);
            
            // 1. Demander les coordonnees
            print!("Coordonnées de la proue (ex: A1) : ");
            io::stdout().flush().unwrap();
            let mut saisie_coord = String::new();
            io::stdin().read_line(&mut saisie_coord).expect("Erreur de lecture");

            let coord = match analyser_saisie(&saisie_coord) {
                Some(c) => c,
                None => {
                    println!("Coordonnées invalides ! Recommencez.");
                    continue; // On relance la boucle pour ce meme bateau
                }
            };

            // 2. Demander l'orientation
            let orientation = demander_orientation();

            // 3. Creer le navire et tenter de le placer
            let nouveau_navire = Navire::new(nom, *taille, coord, orientation);
            
            match grille.placer_navire(nouveau_navire) {
                Ok(_) => {
                    println!(">>> {} positionné avec succès !", nom);
                    break; // Le bateau est place, on casse cette boucle pour passer au bateau suivant
                }
                Err(message) => {
                    // Si ca deborde ou chevauche, on affiche l'erreur et on laisse la boucle recommencer
                    println!("ERREUR : {}", message);
                    println!("Veuillez choisir un autre emplacement.");
                }
            }
        }
    }
    println!("\n--- VOTRE CARTE ACTUELLE ---");        
    grille.afficher(false);
    println!("\nTous les navires sont en position !");
}

fn nettoyer_ecran() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}