mod modele;
mod reseau;

use modele::{Coordonnee, Grille, Navire, Orientation, ResultatTir, TAILLE_GRILLE};
use reseau::{attendre_port_knocking, envoyer_message, heberger_partie, recevoir_message, rejoindre_partie, MessageReseau};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::{self, stdout, Write};
use rand::RngExt;
use std::collections::HashMap;
use std::net::IpAddr;

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
    let mut mon_nom = mon_nom.trim().to_string();

    if mon_nom.is_empty() {
        mon_nom = "Anonyme".to_string();
    }

    let est_hote: bool;
    let code_secret: String;
    let mut ip_serveur = String::new();
    let mut tentatives_echouees: HashMap<IpAddr, u32> = HashMap::new();

    // 2. Le menu de connexion
    loop {
        let mut terminal = io::stdout();
        execute!(terminal, cursor::MoveTo(0, 0), Clear(ClearType::All)).unwrap();

        println!("=====================================");
        println!("        BATAILLE NAVALE RÉSEAU       ");
        println!("=====================================\n");
        println!("Bienvenue, Amiral {} !\n", mon_nom);

        println!("1. Héberger une partie");
        println!("2. Rejoindre une partie");
        println!("3. Guide pratique et règles du jeu");
        println!("4. Quitter le jeu");
        print!("\nVotre choix (1, 2, 3 ou 4) : ");
        io::stdout().flush().unwrap();

        let mut choix = String::new();
        io::stdin().read_line(&mut choix).unwrap();

        match choix.trim() {
            "1" => {
                let pin = (rand::random::<u16>() % 9000) + 1000;
                code_secret = pin.to_string();
                
                println!("\n===================================================");
                println!("  SALON SÉCURISÉ CRÉÉ !");
                println!("  Transmettez ce code à votre adversaire : {}", code_secret);
                println!("===================================================\n");

                est_hote = true;
                break;
            }
            "2" => {
                // Boucle de validation stricte de l'adresse IP
                loop {
                    print!("Adresse IP du serveur : ");
                    io::stdout().flush().unwrap();
                    let mut ip = String::new();
                    io::stdin().read_line(&mut ip).unwrap();
                    
                    // On tente de convertir le texte en vraie adresse IP
                    if ip.trim().parse::<IpAddr>().is_ok() {
                        ip_serveur = ip.trim().to_string();
                        break;
                    } else {
                        println!("\x1b[31m[ERREUR]\x1b[0m Format invalide. Veuillez entrer une adresse IPv4 ou IPv6 (ex: 127.0.0.1).\n");
                    }
                }
                
                print!("Code secret du salon : ");
                io::stdout().flush().unwrap();
                let mut saisie_code = String::new();
                io::stdin().read_line(&mut saisie_code).unwrap();
                code_secret = saisie_code.trim().to_string(); 
                
                est_hote = false;
                break;
            }
            "3" => {
                afficher_guide();
            }
            "4" => {
                // On nettoie l'ecran une derniere fois et on quitte 
                execute!(terminal, Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();
                println!("\x1b[1;36m[SYSTÈME]\x1b[0m Fermeture du Centre de Commandement. Au revoir Amiral \x1b[1m{}\x1b[0m !\n", mon_nom.to_uppercase());
                std::process::exit(0);
            }
            _ => {
                println!("\x1b[1;31m[ERREUR]\x1b[0m Choix invalide. Appuyez sur Entrée pour réessayer...");
                let mut attente = String::new();
                io::stdin().read_line(&mut attente).unwrap();
            }
        }
    }

    // --- 3. CONNEXION ET AUTHENTIFICATION ---
    // La boucle tourne en boucle tant que l'authentification n'est pas un succes
    let (mut flux_tcp, nom_adversaire) = loop {
        
        // 1. On tente d'etablir la connexion reseau (heberger ou rejoindre)
        let resultat_connexion = if est_hote {
            if let Err(msg) = attendre_port_knocking() {
                println!("\n\x1b[1;31m[ERREUR]\x1b[0m {}", msg);
                std::process::exit(1);
            }
            heberger_partie("3333")
        } else {
            rejoindre_partie(&ip_serveur, "3333")
        };

        // 2. On verifie si la connexion a reussi pour recuperer le flux
        let mut flux = match resultat_connexion {
            Some(f) => f,
            None => {
                if est_hote {
                    println!("\n\x1b[1;31m[ERREUR]\x1b[0m Impossible de créer le salon.");
                } else {
                    println!("\n\x1b[1;31m[ERREUR]\x1b[0m Impossible de joindre le serveur. Vérifiez l'IP.");
                }
                std::process::exit(1);
            }
        };

        // 3. Controle de securite Fail2Ban et code secret
        if est_hote {
            let ip_client = flux.adresse_ip();

            // Verification sur la liste noire avant de demander le code
            if let Some(&nb_echecs) = tentatives_echouees.get(&ip_client) {
                if nb_echecs >= 3 {
                    println!("\x1b[1;31m[BAN]\x1b[0m Tentative de connexion bloquée pour {}", ip_client);
                    let _ = envoyer_message(&mut *flux, &MessageReseau::RepAuthFail);
                    continue; // On refuse la connexion et on retourne au menu d'attente pour le prochain client
                }
            }

            println!("\x1b[1;34m[AUTH]\x1b[0m En attente de l'authentification de {}...", ip_client);
            
            match recevoir_message(&mut *flux) {
                Some(MessageReseau::Hello(nom_client, code_client)) => {
                    if code_client == code_secret {
                        println!("\x1b[1;32m[SUCCÈS]\x1b[0m Authentification réussie pour {}.", nom_client);
                        tentatives_echouees.remove(&ip_client); // On efface ses erreurs precedentes en cas de succes
                        
                        envoyer_message(&mut *flux, &MessageReseau::RepAuthOk).unwrap();
                        let mon_hello = MessageReseau::Hello(mon_nom.clone(), "".to_string());
                        envoyer_message(&mut *flux, &mon_hello).unwrap();
                        
                        // SUCCES : On casse la boucle et on renvoie le flux et le nom valide au jeu
                        break (flux, nom_client); 
                    } else {
                        // ECHEC : Ajout à la liste noire
                        let n = tentatives_echouees.entry(ip_client).or_insert(0);
                        *n += 1;
                        println!("\x1b[1;31m[ALERTE]\x1b[0m Mauvais code ({}/3) de {}", *n, ip_client);
                        let _ = envoyer_message(&mut *flux, &MessageReseau::RepAuthFail);
                    }
                }
                _ => println!("\x1b[1;31m[ALERTE]\x1b[0m Déconnexion inattendue pendant l'authentification."),
            }
        } else {
            // Logique du Client
            let mon_hello = MessageReseau::Hello(mon_nom.clone(), code_secret.clone());
            envoyer_message(&mut *flux, &mon_hello).unwrap();
            
            match recevoir_message(&mut *flux) {
                Some(MessageReseau::RepAuthOk) => {
                    println!("\x1b[1;32m[SUCCÈS]\x1b[0m Accès autorisé !");
                    if let Some(MessageReseau::Hello(nom_hote, _)) = recevoir_message(&mut *flux) {
                        break (flux, nom_hote); // Succes pour le client aussi
                    }
                }
                _ => {
                    println!("\x1b[1;31m[BAN]\x1b[0m Le code de salon est incorrect ou vous êtes banni.");
                    std::process::exit(1); // Le client ferme son jeu
                }
            }
        }
    };

    println!("\n\x1b[1;32m[ALLIANCE]\x1b[0m Connexion sécurisée avec l'Amiral {} !\n", nom_adversaire.to_uppercase());
    
    'partie: loop {

        // 4. La phase de placement (Chacun le fait de son cote localement)
        let mut ma_grille = Grille::new();
        let mut radar = Grille::new(); // Grille vide pour noter nos tirs
        
        phase_placement(&mut ma_grille, &mon_nom);

        println!("\n\x1b[1;36m[RÉSEAU]\x1b[0m En attente que l'Amiral \x1b[1m{}\x1b[0m termine son déploiement...", nom_adversaire);
        
        let mut grille_adversaire = Grille::new();

        if est_hote {
            println!("\x1b[1;36m[RÉSEAU]\x1b[0m Synchronisation : Réception de la flotte ennemie...");
            loop {
                match recevoir_message(&mut *flux_tcp) {
                    Some(MessageReseau::EnvoiNavire(nom, taille, x, y, ori)) => {
                        let orientation = if ori == "H" { modele::Orientation::Horizontal } else { modele::Orientation::Vertical };
                        let navire = Navire::new(&nom, taille, Coordonnee { x, y }, orientation);
                        grille_adversaire.placer_navire(navire).unwrap();
                    }
                    Some(MessageReseau::FlotteOk) => {
                        println!("\x1b[1;32m[SUCCÈS]\x1b[0m Flotte adverse verrouillée sur le serveur !");
                        break;
                    }
                    _ => {}
                }
            }
        } else {
            println!("\x1b[1;36m[RÉSEAU]\x1b[0m Synchronisation : Envoi sécurisé de votre flotte au serveur...");
            for navire in &ma_grille.navires {
                let ori = match navire.orientation {
                    Orientation::Horizontal => "H",
                    Orientation::Vertical => "V",
                };
                let msg = MessageReseau::EnvoiNavire(navire.nom.clone(), navire.taille, navire.coord_depart.x, navire.coord_depart.y, ori.to_string());
                envoyer_message(&mut *flux_tcp, &msg).unwrap();
                
                // ASTUCE RÉSEAU : Cette micro-pause empêche le protocole TCP de coller 
                // tous les bateaux dans un seul paquet, ce qui saturerait notre BufReader !
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            envoyer_message(&mut *flux_tcp, &MessageReseau::FlotteOk).unwrap();
        }

        std::thread::sleep(std::time::Duration::from_millis(500));
        nettoyer_ecran();

        // Tirage au sort pour savoir qui commence en premier
        let mut mon_tour;
        if est_hote {
            mon_tour = rand::rng().random_bool(0.5);
            
            // Il informe le client
            let msg_client = MessageReseau::InfoTour(!mon_tour); 
            envoyer_message(&mut *flux_tcp, &msg_client).unwrap();
        } else {
            println!("\x1b[1;36m[RÉSEAU]\x1b[0m En attente du tirage au sort de l'arbitre...");
            // Le client ecoute la decision
            match recevoir_message(&mut *flux_tcp) {
                Some(MessageReseau::InfoTour(a_moi)) => mon_tour = a_moi,
                _ => mon_tour = false, // Securite par defaut
            }
        }

        let nom_qui_commence = if mon_tour { mon_nom.clone() } else { nom_adversaire.clone() };
        println!("\n\x1b[1;35m[ARBITRE]\x1b[0m Le sort a désigné l'Amiral {} pour lancer la première offensive !", nom_qui_commence.to_uppercase());
        
        // On laisse le temps de lire le message avant de lancer l'interface de combat
        std::thread::sleep(std::time::Duration::from_secs(4));

        nettoyer_ecran();

        println!("\n=========================================================================");
        println!("                            DÉBUT DU COMBAT !                            ");
        println!("=========================================================================");

        loop {
            if mon_tour {
                // --- C'EST MON TOUR ---                       
                // Afficher la grille et gerer les fleches
                let cible = choisir_coordonnee_interactive(&ma_grille, &radar);

                // On traduit la coordonnee pour l'affichage
                let lettre = (b'A' + cible.x as u8) as char;
                let chiffre = cible.y + 1;
                println!("\n\x1b[1;33m[CIBLE]\x1b[0m Verrouillage des missiles sur {}{}...", lettre, chiffre);

                if envoyer_message(&mut *flux_tcp, &MessageReseau::Tir(cible)).is_err() {
                    println!("\n\x1b[1;31m[DÉCONNEXION]\x1b[0m L'Amiral ennemi a déserté le champ de bataille !");
                    break 'partie;
                }
                println!("\x1b[1;36m[RÉSEAU]\x1b[0m Tir envoyé ! En attente du rapport de dégâts...");

                // --- LE SERVEUR EST LE SEUL JUGE ---
                let reponse_serveur = if est_hote {
                    // L'Hôte calcule l'impact en local sur la grille qu'il a reçue
                    let resultat = grille_adversaire.tirer(cible);
                    let rep = match resultat {
                        ResultatTir::Aleau => MessageReseau::RepAleau,
                        ResultatTir::Touche => MessageReseau::RepTouche,
                        ResultatTir::Coule(nom) => MessageReseau::RepCoule(nom),
                        _ => MessageReseau::RepAleau,
                    };
                    
                    let rep_finale = if grille_adversaire.flotte_coulee() { MessageReseau::RepFin } else { rep };
                    
                    // L'Hôte informe le client du résultat
                    if envoyer_message(&mut *flux_tcp, &rep_finale).is_err() {
                        println!("\n\x1b[1;31m[DÉCONNEXION]\x1b[0m L'Amiral ennemi a déserté le champ de bataille !");
                        break 'partie;
                    }
                    rep_finale // L'Hôte s'auto-renvoie le résultat pour l'afficher
                } else {
                    // Le client attend bêtement la décision du serveur
                    recevoir_message(&mut *flux_tcp).unwrap_or(MessageReseau::RepAleau)
                };

                // On utilise la réponse du serveur pour mettre à jour l'écran
                match reponse_serveur {
                    MessageReseau::RepAleau => {
                        println!("\x1b[1;33m[RÉSULTAT]\x1b[0m \x1b[90mPlouf... C'est dans l'eau.\x1b[0m\n");
                        radar.cases[cible.y][cible.x].etat = modele::EtatCase::Aleau;
                    }
                    MessageReseau::RepTouche => {
                        println!("\x1b[1;33m[RÉSULTAT]\x1b[0m \x1b[31mBOUM ! Vous avez touché un navire !\x1b[0m\n");
                        radar.cases[cible.y][cible.x].etat = modele::EtatCase::Touche;
                    }
                    MessageReseau::RepCoule(nom) => {
                        println!("\x1b[1;33m[RÉSULTAT]\x1b[0m \x1b[31mTOUCHÉ ET COULÉ ! Vous avez détruit le {} !\x1b[0m\n", nom);
                        radar.cases[cible.y][cible.x].etat = modele::EtatCase::Touche;
                    }
                    MessageReseau::RepFin => {
                        println!("\n\x1b[1;32m=========================================================================\x1b[0m");
                        println!("\x1b[1;32m           VICTOIRE TOTALE ! La flotte ennemie est détruite !            \x1b[0m");
                        println!("\x1b[1;32m=========================================================================\x1b[0m\n");
                        radar.cases[cible.y][cible.x].etat = modele::EtatCase::Touche;
                        afficher_plateau_double(&ma_grille, &radar, None);
                        break; // Fin du jeu 
                    }
                    _ => println!("Erreur réseau inattendue."),
                }
                
                println!("=========================================================================");
                println!("                            RADAR MIS À JOUR                            ");
                println!("=========================================================================\n");
                afficher_plateau_double(&ma_grille, &radar, None);
                
                mon_tour = false; // Fin de mon tour

            } else {
                println!("\n\x1b[1;36m[RÉSEAU]\x1b[0m En attente de l'attaque de \x1b[1m{}\x1b[0m...", nom_adversaire);

                match recevoir_message(&mut flux_tcp) {
                    Some(MessageReseau::Tir(coord)) => {
                        nettoyer_ecran();
                        let lettre = (b'A' + coord.x as u8) as char;
                        println!("\n\x1b[1;31m[ALERTE]\x1b[0m Tir ennemi détecté en \x1b[1m{}{}\x1b[0m !", lettre, coord.y + 1);

                        // --- LE SERVEUR EST LE SEUL JUGE ---
                        if est_hote {
                            // L'Hôte encaisse le tir, calcule et donne le verdict au client
                            let resultat = ma_grille.tirer(coord);
                            if ma_grille.flotte_coulee() {
                                if envoyer_message(&mut *flux_tcp, &MessageReseau::RepFin).is_err() {
                                    println!("\n\x1b[1;31m[DÉCONNEXION]\x1b[0m L'Amiral ennemi a fui la bataille !");
                                    break 'partie;
                                }
                                println!("\n\x1b[1;31m=========================================================================\x1b[0m");
                                println!("\x1b[1;31m              DÉFAITE... Toute votre flotte a été anéantie.              \x1b[0m");
                                println!("\x1b[1;31m=========================================================================\x1b[0m\n");
                                afficher_plateau_double(&ma_grille, &radar, None);
                                break;
                            } else {
                                let reponse = match resultat { 
                                    ResultatTir::Aleau => {
                                        println!("\x1b[1;33m[RÉSULTAT]\x1b[0m \x1b[90mPlouf... C'est dans l'eau.\x1b[0m\n");
                                        MessageReseau::RepAleau
                                    },
                                    ResultatTir::Touche => {
                                        println!("\x1b[1;33m[RÉSULTAT]\x1b[0m \x1b[31mBOUM ! Un de vos navires a été touché !\x1b[0m\n");
                                        MessageReseau::RepTouche
                                    },
                                    ResultatTir::Coule(nom) => {
                                        println!("\x1b[1;33m[RÉSULTAT]\x1b[0m \x1b[31mATTAQUE DÉVASTATRICE ! Votre {} a été coulé !\x1b[0m\n", nom);
                                        MessageReseau::RepCoule(nom)
                                    },
                                    _ => MessageReseau::RepAleau,  
                                };
                                if envoyer_message(&mut *flux_tcp, &reponse).is_err() {
                                    println!("\n\x1b[1;31m[DÉCONNEXION]\x1b[0m L'Amiral ennemi a déserté le champ de bataille !");
                                    break 'partie;
                                }
                            }
                        } else {
                            // Le Client encaisse le tir, mais DOIT attendre le verdict de l'Hôte
                            match recevoir_message(&mut *flux_tcp) {
                                Some(MessageReseau::RepAleau) => {
                                    println!("\x1b[1;33m[RÉSULTAT]\x1b[0m \x1b[90mPlouf... L'ennemi a raté.\x1b[0m\n");
                                    ma_grille.cases[coord.y][coord.x].etat = modele::EtatCase::Aleau;
                                }
                                Some(MessageReseau::RepTouche) => {
                                    println!("\x1b[1;33m[RÉSULTAT]\x1b[0m \x1b[31mBOUM ! Vous avez été touché !\x1b[0m\n");
                                    ma_grille.cases[coord.y][coord.x].etat = modele::EtatCase::Touche;
                                }
                                Some(MessageReseau::RepCoule(nom)) => {
                                    println!("\x1b[1;33m[RÉSULTAT]\x1b[0m \x1b[31mDÉSASTRE ! Votre {} a été coulé !\x1b[0m\n", nom);
                                    ma_grille.cases[coord.y][coord.x].etat = modele::EtatCase::Touche;
                                }
                                Some(MessageReseau::RepFin) => {
                                    ma_grille.cases[coord.y][coord.x].etat = modele::EtatCase::Touche;
                                    println!("\n\x1b[1;31m=========================================================================\x1b[0m");
                                    println!("\x1b[1;31m              DÉFAITE... Toute votre flotte a été anéantie.              \x1b[0m");
                                    println!("\x1b[1;31m=========================================================================\x1b[0m\n");
                                    afficher_plateau_double(&ma_grille, &radar, None);
                                    break;
                                }
                                _ => {}
                            }
                        }
                        afficher_plateau_double(&ma_grille, &radar, None);
                    }
                    None => {
                        println!("\n\x1b[1;31m[DÉCONNEXION]\x1b[0m L'Amiral ennemi a déserté le champ de bataille !");
                        break 'partie;
                    }
                    _ => println!("\x1b[1;31m[ALERTE]\x1b[0m Message inattendu pendant le tour adverse."),
                }
                mon_tour = true; // L'adversaire a fini
            }
        } // Fin de la boucle de combat

        // --- NEGOCIATION DE LA REVANCHE ---
        println!("\n\x1b[1;34m --- FIN DES HOSTILITÉS ---\x1b[0m");
        print!("Voulez-vous proposer une revanche ? (O/N) : ");
        io::stdout().flush().unwrap();
        
        let mut choix = String::new();
        io::stdin().read_line(&mut choix).unwrap();
        let veut_rejouer = choix.trim().eq_ignore_ascii_case("o");

        if envoyer_message(&mut *flux_tcp, &MessageReseau::Revanche(veut_rejouer)).is_err() {
            println!("\n\x1b[1;31m[DÉCONNEXION]\x1b[0m Impossible d'envoyer la demande de revanche.");
            break 'partie; 
        }

        println!("\x1b[1;36m[RÉSEAU]\x1b[0m En attente de la décision de l'Amiral {}...", nom_adversaire);

        match recevoir_message(&mut *flux_tcp) {
            Some(MessageReseau::Revanche(adversaire_veut_rejouer)) => {
                if veut_rejouer && adversaire_veut_rejouer {
                    println!("\n\x1b[1;32m[SUCCÈS]\x1b[0m La revanche est acceptée ! Nettoyage du pont...");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    nettoyer_ecran();
                    continue 'partie;
                } else {
                    println!("\n\x1b[1;31m[DÉCONNEXION]\x1b[0m La revanche a été refusée. Fin des transmissions.");
                    break 'partie;
                }
            }
            _ => {
                println!("\n\x1b[1;31m[DÉCONNEXION]\x1b[0m La communication a été rompue.");
                break 'partie;
            }
        }
    } // Fin de 'partie: loop

    println!("\x1b[1;36m[SYSTÈME]\x1b[0m Fermeture du Centre de Commandement. Au revoir Amiral \x1b[1m{}\x1b[0m !\n", mon_nom.to_uppercase());
}

fn afficher_guide() {
    let mut terminal = io::stdout();
    execute!(terminal, cursor::MoveTo(0, 0), Clear(ClearType::All)).unwrap();

    println!("\x1b[1;36m===========================================================\x1b[0m");
    println!("\x1b[1;36m                  GUIDE DE L'AMIRAL                        \x1b[0m");
    println!("\x1b[1;36m===========================================================\x1b[0m\n");
    
    println!("\x1b[1;34m --- CONNEXION RÉSEAU & ADRESSE IP ---\x1b[0m");
    println!("L'Hôte doit communiquer son adresse IP locale au joueur qui le rejoint.");
    println!("  - Sous \x1b[1mWindows\x1b[0m : Ouvrez l'invite de commande et tapez '\x1b[1;33mipconfig\x1b[0m' (cherchez l'adresse IPv4).");
    println!("  - Sous \x1b[1mLinux/Mac\x1b[0m : Ouvrez le terminal et tapez '\x1b[1;33mhostname -I\x1b[0m' ou '\x1b[1;33mip a\x1b[0m'.");
    println!("  (\x1b[35mAstuce\x1b[0m : Tapez '\x1b[1;33m127.0.0.1\x1b[0m' pour jouer contre vous-même sur le même PC !)\n");

    println!("\x1b[1;34m --- COMMANDES DE JEU ---\x1b[0m");
    println!("  - \x1b[1;33mFLÈCHES\x1b[0m : Déplacer le curseur de ciblage ou votre bateau.");
    println!("  - \x1b[1;33mTOUCHE 'R'\x1b[0m : Faire pivoter le navire lors du déploiement.");
    println!("  - \x1b[1;33mENTRÉE\x1b[0m : Valider un tir ou confirmer le placement d'un navire.\n");

    println!("\x1b[1;34m --- DÉROULEMENT D'UN TOUR ---\x1b[0m");
    println!("  \x1b[1;32m1.\x1b[0m C'est votre tour : Vous visez sur le radar et tirez.");
    println!("  \x1b[1;32m2.\x1b[0m Le jeu vous informe immédiatement du résultat (À l'eau, Touché, Coulé).");
    println!("  \x1b[1;32m3.\x1b[0m L'adversaire voit votre tir s'abattre sur sa propre grille et encaisse les dégâts.");
    println!("  \x1b[1;32m4.\x1b[0m Les rôles s'inversent ! Le suspense est total.\n");

    println!("\x1b[1;34m --- LÉGENDE DU RADAR ---\x1b[0m");
    println!("  [\x1b[34m~\x1b[0m] : Eau inexplorée        [\x1b[90mO\x1b[0m] : Tir raté (Plouf !)");
    println!("  [\x1b[32mB\x1b[0m] : Vos navires           [\x1b[31mX\x1b[0m] : Navire touché !");

    println!("\n\x1b[1;36m===========================================================\x1b[0m");
    println!("Appuyez sur \x1b[1;33mENTRÉE\x1b[0m pour retourner au Centre de Commandement...");
    
    let mut attente = String::new();
    io::stdin().read_line(&mut attente).unwrap();
}

fn afficher_plateau_double(ma_grille: &Grille, radar: &Grille, curseur_radar: Option<Coordonnee>) {
    let lignes_flotte = ma_grille.vers_lignes(false, None, None);
    let lignes_radar = radar.vers_lignes(true, curseur_radar, None);

    println!("        ÉTAT DE VOTRE FLOTTE                       RADAR TACTIQUE        ");
    println!("=========================================================================");
    
    for (g, d) in lignes_flotte.iter().zip(lignes_radar.iter()) {
        println!("{}   |   {}", g, d);
    }
}

fn choisir_coordonnee_interactive(ma_grille: &Grille, radar: &Grille) -> Coordonnee {
    let mut curseur = Coordonnee { x: 0, y: 0 };
    let mut premiere_fois = true;

    loop {
        disable_raw_mode().unwrap();
        let mut terminal = stdout();
        
        if premiere_fois {
            premiere_fois = false;
        } else {
            execute!(
                terminal, 
                cursor::MoveUp(18), 
                cursor::MoveToColumn(0), 
                Clear(ClearType::FromCursorDown)
            ).unwrap();
        }

        println!("=========================================================================");
        println!("                              À VOTRE TOUR!                              ");
        println!("                DÉPLACEZ LE CURSEUR ET APPUYEZ SUR ENTRÉE                ");
        println!("=========================================================================\n");
        
        // On affiche le double tableau avec le curseur projete sur le radar
        afficher_plateau_double(ma_grille, radar, Some(curseur));
        
        enable_raw_mode().unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Up => { if curseur.y > 0 { curseur.y -= 1; } }
                    KeyCode::Down => { if curseur.y < TAILLE_GRILLE - 1 { curseur.y += 1; } }
                    KeyCode::Left => { if curseur.x > 0 { curseur.x -= 1; } }
                    KeyCode::Right => { if curseur.x < TAILLE_GRILLE - 1 { curseur.x += 1; } }
                    KeyCode::Enter => {
                        let etat_case = &radar.cases[curseur.y][curseur.x].etat;
                        if *etat_case == modele::EtatCase::Touche || *etat_case == modele::EtatCase::Aleau {
                            continue;
                        }
                        disable_raw_mode().unwrap();
                        return curseur;
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        disable_raw_mode().unwrap();
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn placer_navire_interactif(grille: &mut Grille, nom: &str, taille: usize) {
    let mut curseur = Coordonnee { x: 0, y: 0 };
    let mut est_horizontal = true; // On gere l'orientation avec un booleen
    let mut message_erreur = String::new(); // Pour afficher si on place mal le bateau

    loop {
        disable_raw_mode().unwrap();
        let mut terminal = stdout();
        
        // On se replace tout en haut a gauche et on nettoie vers le bas
        execute!(terminal, cursor::MoveTo(0, 0), Clear(ClearType::FromCursorDown)).unwrap();
        
        println!("=================================================");
        println!(" DÉPLOIEMENT : {} (Taille : {})", nom.to_uppercase(), taille);
        println!(" Flèches : Déplacer | 'R' : Pivoter | Entrée : Valider");
        
        let texte_orientation = if est_horizontal { "Horizontale" } else { "Verticale" };
        println!(" Orientation actuelle : {}", texte_orientation);
        println!("=================================================\n");

        // Affichage dynamique des erreurs
        if !message_erreur.is_empty() {
            println!("ERREUR : {}\n", message_erreur);
        } else {
            println!("\n"); // Pour garder la grille a la meme hauteur
        }

        // 1. On traduit l'orientation actuelle
        let orientation_fantome = if est_horizontal { 
            Orientation::Horizontal 
        } else { 
            Orientation::Vertical 
        };
        
        // 2. On cree le navire fantome (il n'est pas encore dans la grille c'est juste un modele)
        let navire_fantome = Navire::new(nom, taille, curseur, orientation_fantome);

        // 3. On l'affiche (On met None pour le curseur simple et Some pour le fantome)
        grille.afficher(false, None, Some(&navire_fantome));

        enable_raw_mode().unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Up => { if curseur.y > 0 { curseur.y -= 1; } }
                    KeyCode::Down => { curseur.y += 1; } 
                    KeyCode::Left => { if curseur.x > 0 { curseur.x -= 1; } }
                    KeyCode::Right => { curseur.x += 1; }
                    
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        est_horizontal = !est_horizontal; 
                        message_erreur.clear();
                    }
                    
                    KeyCode::Enter => {
                        let orientation = if est_horizontal { Orientation::Horizontal } else { Orientation::Vertical };
                        let nouveau_navire = Navire::new(nom, taille, curseur, orientation);

                        match grille.placer_navire(nouveau_navire) {
                            Ok(_) => {
                                disable_raw_mode().unwrap();
                                return; 
                            }
                            Err(msg) => {
                                message_erreur = msg.to_string(); 
                            }
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        disable_raw_mode().unwrap();
                        std::process::exit(0);
                    }
                    _ => {}
                }

                // 1. On calcule la bordure maximum selon l'orientation et la taille du bateau
                let limite_x = if est_horizontal { TAILLE_GRILLE - taille } else { TAILLE_GRILLE - 1 };
                let limite_y = if est_horizontal { TAILLE_GRILLE - 1 } else { TAILLE_GRILLE - taille };

                // 2. Si le curseur depasse cette bordure on le force a rester dedans
                if curseur.x > limite_x { curseur.x = limite_x; }
                if curseur.y > limite_y { curseur.y = limite_y; }
            }
        }
    }
}

fn placer_flotte_aleatoire(grille: &mut Grille) {
    let flotte_a_placer = [
        ("Porte-avions", 5),
        ("Croiseur", 4),
        ("Contre-torpilleur", 3),
        ("Sous-marin", 3),
        ("Torpilleur", 2),
    ];
    
    let mut rng = rand::rng();

    for (nom, taille) in flotte_a_placer.iter() {
        loop {
            let x = rng.random_range(0..TAILLE_GRILLE);
            let y = rng.random_range(0..TAILLE_GRILLE);
            
            let orientation = if rng.random_bool(0.5) {
                Orientation::Horizontal
            } else {
                Orientation::Vertical
            };

            let navire = Navire::new(nom, *taille, Coordonnee { x, y }, orientation);
            
            if grille.placer_navire(navire).is_ok() {
                break;
            }
        }
    }
}

fn phase_placement(grille: &mut Grille, nom_joueur: &str) {
    let mut terminal = stdout();
    disable_raw_mode().unwrap();
    execute!(terminal, cursor::MoveTo(0, 0), Clear(ClearType::All)).unwrap();
    
    println!("=====================================");
    println!("   DÉPLOIEMENT : AMIRAL {}   ", nom_joueur.to_uppercase());
    println!("=====================================\n");
    
    // Le menu de choix
    println!("Comment souhaitez-vous déployer votre flotte ?");
    println!("1. Manuellement (Flèches du clavier)");
    println!("2. Aléatoirement (Placement express)");
    print!("Votre choix (1 ou 2) : ");
    io::stdout().flush().unwrap();
    
    let mut choix = String::new();
    io::stdin().read_line(&mut choix).unwrap();
    
    if choix.trim() == "2" {
        // Deploiement aleatoire
        placer_flotte_aleatoire(grille);
    } else {
        // Deploiement manuel interactif
        let flotte_a_placer = [
            ("Porte-avions", 5),
            ("Croiseur", 4),
            ("Contre-torpilleur", 3),
            ("Sous-marin", 3),
            ("Torpilleur", 2),
        ];
        for (nom, taille) in flotte_a_placer.iter() {
            placer_navire_interactif(grille, nom, *taille);
        }
    }

    // Affichage final de la grille
    execute!(terminal, cursor::MoveTo(0, 0), Clear(ClearType::All)).unwrap();
    
    println!("\n=====================================");
    println!("   FLOTTE DE {} DÉPLOYÉE !   ", nom_joueur.to_uppercase());
    println!("=====================================\n");
    
    grille.afficher(false, None, None); 
    
    println!("\nTous les navires sont en position ! Appuyez sur Entrée pour continuer...");
    let mut attente = String::new();
    io::stdin().read_line(&mut attente).unwrap();
}

fn nettoyer_ecran() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}