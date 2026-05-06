// src/main.rs

mod modele;
mod reseau;
mod affichage;

use modele::{Coordonnee, Grille, Navire, Orientation, ResultatTir};
use reseau::{attendre_port_knocking, envoyer_message, heberger_partie, recevoir_message, rejoindre_partie, MessageReseau, FluxJeu};
use affichage::{nettoyer_ecran, afficher_guide, afficher_plateau_double, choisir_action_interactive, phase_placement, ActionTour, C};

use rand::RngExt; 
use crossterm::{cursor, execute, terminal::{Clear, ClearType}};
use std::io::{self, Write};
use std::collections::HashMap;
use std::net::IpAddr;

fn main() {
    nettoyer_ecran();
    println!("=====================================");
    println!("        BATAILLE NAVALE RÉSEAU       ");
    println!("=====================================\n");

    print!("Entrez votre nom de Commandant : ");
    io::stdout().flush().unwrap();
    let mut mon_nom = String::new();
    io::stdin().read_line(&mut mon_nom).unwrap();
    let mut mon_nom = mon_nom.trim().to_string();

    if mon_nom.is_empty() {
        mon_nom = "Anonyme".to_string();
    }

    loop {
        let (est_hote, code_secret, ip_serveur) = menu_principal(&mon_nom);
        let (flux_tcp, nom_adversaire) = etablir_connexion(est_hote, &ip_serveur, &code_secret, &mon_nom);
        lancer_combat(flux_tcp, est_hote, &mon_nom, &nom_adversaire);
    }
}

// =====================================================================

fn menu_principal(mon_nom: &str) -> (bool, String, String) {
    let est_hote: bool;
    let code_secret: String;
    let mut ip_serveur = String::new();

    loop {
        let mut terminal = io::stdout();
        execute!(terminal, cursor::MoveTo(0, 0), Clear(ClearType::All)).unwrap();

        println!("=====================================");
        println!("        BATAILLE NAVALE RÉSEAU       ");
        println!("=====================================\n");
        println!("Bienvenue, Amiral {} !\n", mon_nom);

        println!("1. Héberger une partie");
        println!("2. Rejoindre une partie");
        println!("3. Jouer contre l'IA (Bot Python)");
        println!("4. Guide pratique et règles du jeu");
        println!("5. Quitter le jeu");
        print!("\nVotre choix (1, 2, 3, 4 ou 5) : ");
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
                loop {
                    print!("Adresse IP du serveur : ");
                    io::stdout().flush().unwrap();
                    let mut ip = String::new();
                    io::stdin().read_line(&mut ip).unwrap();
                    
                    if ip.trim().parse::<IpAddr>().is_ok() {
                        ip_serveur = ip.trim().to_string();
                        break;
                    } else {
                        println!("{}[ERREUR]{} Format invalide. Veuillez entrer une adresse IPv4 ou IPv6 (ex: 127.0.0.1).\n", C::ROUGE, C::RESET);
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
                let pin = (rand::random::<u16>() % 9000) + 1000;
                code_secret = pin.to_string();
                
                println!("\n===================================================");
                println!("       DÉPLOIEMENT DU BOT AMIRAL EN COURS...");
                println!(" Initialisation de l'IA avec le code secret : {}", code_secret);
                println!("===================================================\n");

                est_hote = true;
                let pin_clone = code_secret.clone();
                
                #[cfg(target_os = "windows")]
                let commande_python = "python";
                #[cfg(not(target_os = "windows"))]
                let commande_python = "python3";

                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    
                    let _ = std::process::Command::new(commande_python)
                        .arg("bot_touche_coule/main.py")
                        .arg(pin_clone)
                        .stdout(std::process::Stdio::null()) 
                        .stderr(std::process::Stdio::null())
                        .spawn();
                });

                break;
            }
            "4" => {
                afficher_guide();
            }
            "5" => {
                let mut terminal = io::stdout();
                execute!(terminal, Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();
                println!("{}[SYSTÈME]{} Fermeture du Centre de Commandement. Au revoir Amiral {}{}{} !\n", C::CYAN, C::RESET, C::GRAS, mon_nom.to_uppercase(), C::RESET);
                std::process::exit(0);
            }
            _ => {
                println!("{}[ERREUR]{} Choix invalide. Appuyez sur Entrée pour réessayer...", C::ROUGE, C::RESET);
                let mut attente = String::new();
                io::stdin().read_line(&mut attente).unwrap();
            }
        }
    }

    (est_hote, code_secret, ip_serveur)
}

fn etablir_connexion(est_hote: bool, ip_serveur: &str, code_secret: &str, mon_nom: &str) -> (Box<dyn FluxJeu>, String) {
    let mut tentatives_echouees: HashMap<IpAddr, u32> = HashMap::new();

    let (flux_tcp, nom_adversaire) = loop {
        let resultat_connexion = if est_hote {
            if let Err(msg) = attendre_port_knocking() {
                println!("\n{}[ERREUR]{} {}", C::ROUGE, C::RESET, msg);
                std::process::exit(1);
            }
            heberger_partie("3333")
        } else {
            rejoindre_partie(ip_serveur, "3333")
        };

        let mut flux = match resultat_connexion {
            Some(f) => f,
            None => {
                if est_hote {
                    println!("\n{}[ERREUR]{} Impossible de créer le salon.", C::ROUGE, C::RESET);
                } else {
                    println!("\n{}[ERREUR]{} Impossible de joindre le serveur. Vérifiez l'IP.", C::ROUGE, C::RESET);
                }
                std::process::exit(1);
            }
        };

        if est_hote {
            let ip_client = flux.adresse_ip();

            if let Some(&nb_echecs) = tentatives_echouees.get(&ip_client) {
                if nb_echecs >= 3 {
                    println!("{}[BAN]{} Tentative de connexion bloquée pour {}", C::ROUGE, C::RESET, ip_client);
                    let _ = envoyer_message(&mut *flux, &MessageReseau::RepAuthFail);
                    continue; 
                }
            }

            println!("{}[AUTH]{} En attente de l'authentification de {}...", C::BLEU, C::RESET, ip_client);
            
            match recevoir_message(&mut *flux) {
                Some(MessageReseau::Hello(nom_client, code_client)) => {
                    if code_client == code_secret {
                        println!("{}[SUCCÈS]{} Authentification réussie pour {}.", C::VERT, C::RESET, nom_client);
                        tentatives_echouees.remove(&ip_client); 
                        
                        envoyer_message(&mut *flux, &MessageReseau::RepAuthOk).unwrap();
                        let mon_hello = MessageReseau::Hello(mon_nom.to_string(), "".to_string());
                        envoyer_message(&mut *flux, &mon_hello).unwrap();
                        
                        break (flux, nom_client); 
                    } else {
                        let n = tentatives_echouees.entry(ip_client).or_insert(0);
                        *n += 1;
                        println!("{}[ALERTE]{} Mauvais code ({}/3) de {}", C::ROUGE, C::RESET, *n, ip_client);
                        let _ = envoyer_message(&mut *flux, &MessageReseau::RepAuthFail);
                    }
                }
                _ => println!("{}[ALERTE]{} Déconnexion inattendue pendant l'authentification.", C::ROUGE, C::RESET),
            }
        } else {
            let mon_hello = MessageReseau::Hello(mon_nom.to_string(), code_secret.to_string());
            envoyer_message(&mut *flux, &mon_hello).unwrap();
            
            match recevoir_message(&mut *flux) {
                Some(MessageReseau::RepAuthOk) => {
                    println!("{}[SUCCÈS]{} Accès autorisé !", C::VERT, C::RESET);
                    if let Some(MessageReseau::Hello(nom_hote, _)) = recevoir_message(&mut *flux) {
                        break (flux, nom_hote); 
                    }
                }
                _ => {
                    println!("{}[BAN]{} Le code de salon est incorrect ou vous êtes banni.", C::ROUGE, C::RESET);
                    std::process::exit(1); 
                }
            }
        }
    };

    println!("\n{}[ALLIANCE]{} Connexion sécurisée avec l'Amiral {} !\n", C::VERT, C::RESET, nom_adversaire.to_uppercase());
    (flux_tcp, nom_adversaire)
}

fn lancer_combat(mut flux_tcp: Box<dyn FluxJeu>, est_hote: bool, mon_nom: &str, nom_adversaire: &str) {
    loop {
        let mut ma_grille = Grille::new();
        let mut radar = Grille::new(); 
        
        phase_placement(&mut ma_grille, mon_nom);

        println!("\n{}[RÉSEAU]{} En attente que l'Amiral {}{}{} termine son déploiement...", C::CYAN, C::RESET, C::GRAS, nom_adversaire, C::RESET);
        
        let mut grille_adversaire = Grille::new();

        if est_hote {
            println!("{}[RÉSEAU]{} Synchronisation : Réception de la flotte ennemie...", C::CYAN, C::RESET);
            loop {
                match recevoir_message(&mut *flux_tcp) {
                    Some(MessageReseau::EnvoiNavire(nom, taille, x, y, ori)) => {
                        let orientation = if ori == "H" { Orientation::Horizontal } else { Orientation::Vertical };
                        let navire = Navire::new(&nom, taille, Coordonnee { x, y }, orientation);
                        
                        if let Err(msg_erreur) = grille_adversaire.placer_navire(navire) {
                            println!("\n{}[ALERTE SÉCURITÉ]{} Déploiement adverse invalide : {}", C::ROUGE, C::RESET, msg_erreur);
                            println!("{}[DÉCONNEXION]{} Le client a été expulsé pour cause de triche ou corruption des données.", C::ROUGE, C::RESET);
                            std::thread::sleep(std::time::Duration::from_secs(4));
                            return; 
                        }
                    }
                    Some(MessageReseau::FlotteOk) => {
                        println!("{}[SUCCÈS]{} Flotte adverse verrouillée sur le serveur !", C::VERT, C::RESET);
                        break;
                    }
                    _ => {}
                }
            }
        } else {
            println!("{}[RÉSEAU]{} Synchronisation : Envoi sécurisé de votre flotte au serveur...", C::CYAN, C::RESET);
            for navire in &ma_grille.navires {
                let ori = match navire.orientation {
                    Orientation::Horizontal => "H",
                    Orientation::Vertical => "V",
                };
                let msg = MessageReseau::EnvoiNavire(navire.nom.clone(), navire.taille, navire.coord_depart.x, navire.coord_depart.y, ori.to_string());
                envoyer_message(&mut *flux_tcp, &msg).unwrap();
                
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            envoyer_message(&mut *flux_tcp, &MessageReseau::FlotteOk).unwrap();
        }

        std::thread::sleep(std::time::Duration::from_millis(500));
        nettoyer_ecran();

        let mut mon_tour;
        if est_hote {
            mon_tour = rand::rng().random_bool(0.5);
            
            let msg_client = MessageReseau::InfoTour(!mon_tour); 
            envoyer_message(&mut *flux_tcp, &msg_client).unwrap();
        } else {
            println!("{}[RÉSEAU]{} En attente du tirage au sort de l'arbitre...", C::CYAN, C::RESET);
            match recevoir_message(&mut *flux_tcp) {
                Some(MessageReseau::InfoTour(a_moi)) => mon_tour = a_moi,
                _ => mon_tour = false, 
            }
        }

        let nom_qui_commence = if mon_tour { mon_nom.to_string() } else { nom_adversaire.to_string() };
        println!("\n{}[ARBITRE]{} Le sort a désigné l'Amiral {} pour lancer la première offensive !", C::MAGENTA, C::RESET, nom_qui_commence.to_uppercase());
        
        std::thread::sleep(std::time::Duration::from_secs(4));

        nettoyer_ecran();

        println!("\n=========================================================================");
        println!("                            DÉBUT DU COMBAT !                            ");
        println!("=========================================================================");

        loop {
            if mon_tour {
                let cible = loop {
                    match choisir_action_interactive(&ma_grille, &radar) {
                        ActionTour::Tir(coord) => break coord,
                        ActionTour::Chat(msg) => {
                            if envoyer_message(&mut *flux_tcp, &MessageReseau::Chat(msg.clone())).is_err() {
                                println!("\n{}[DÉCONNEXION]{} L'Amiral ennemi a fui la bataille !", C::ROUGE, C::RESET);
                                return;
                            }
                            println!("{}[Message Envoyé]{}", C::VERT, C::RESET);
                            std::thread::sleep(std::time::Duration::from_millis(500));
                        }
                        ActionTour::Quitter => {
                            return; 
                        }
                    }
                };

                let lettre = (b'A' + cible.x as u8) as char;
                let chiffre = cible.y + 1;
                println!("\n{}[CIBLE]{} Verrouillage des missiles sur {}{}...", C::JAUNE, C::RESET, lettre, chiffre);

                if envoyer_message(&mut *flux_tcp, &MessageReseau::Tir(cible)).is_err() {
                    println!("\n{}[DÉCONNEXION]{} L'Amiral ennemi a déserté le champ de bataille !", C::ROUGE, C::RESET);
                    return;
                }
                println!("{}[RÉSEAU]{} Tir envoyé ! En attente du rapport de dégâts...", C::CYAN, C::RESET);

                let reponse_serveur = if est_hote {
                    let resultat = grille_adversaire.tirer(cible);
                    let rep = match resultat {
                        ResultatTir::Aleau => MessageReseau::RepAleau,
                        ResultatTir::Touche => MessageReseau::RepTouche,
                        ResultatTir::Coule(nom) => MessageReseau::RepCoule(nom),
                        _ => MessageReseau::RepAleau,
                    };
                    
                    let rep_finale = if grille_adversaire.flotte_coulee() { MessageReseau::RepFin } else { rep };
                    
                    if envoyer_message(&mut *flux_tcp, &rep_finale).is_err() {
                        println!("\n{}[DÉCONNEXION]{} L'Amiral ennemi a déserté le champ de bataille !", C::ROUGE, C::RESET);
                        return;
                    }
                    rep_finale 
                } else {
                    recevoir_message(&mut *flux_tcp).unwrap_or(MessageReseau::RepAleau)
                };

                match reponse_serveur {
                    MessageReseau::RepAleau => {
                        println!("{}[RÉSULTAT]{} {}Plouf... C'est dans l'eau.{}", C::JAUNE, C::RESET, C::GRIS, C::RESET);
                        radar.cases[cible.y][cible.x].etat = modele::EtatCase::Aleau;
                    }
                    MessageReseau::RepTouche => {
                        println!("{}[RÉSULTAT]{} {}BOUM ! Vous avez touché un navire !{}", C::JAUNE, C::RESET, C::ROUGE, C::RESET);
                        radar.cases[cible.y][cible.x].etat = modele::EtatCase::Touche;
                    }
                    MessageReseau::RepCoule(nom) => {
                        println!("{}[RÉSULTAT]{} {}TOUCHÉ ET COULÉ ! Vous avez détruit le {} !{}", C::JAUNE, C::RESET, C::ROUGE, nom, C::RESET);
                        radar.cases[cible.y][cible.x].etat = modele::EtatCase::Touche;
                    }
                    MessageReseau::RepFin => {
                        println!("\n{}========================================================================={}", C::VERT, C::RESET);
                        println!("{}           VICTOIRE TOTALE ! La flotte ennemie est détruite !            {}", C::VERT, C::RESET);
                        println!("{}========================================================================={}\n", C::VERT, C::RESET);
                        radar.cases[cible.y][cible.x].etat = modele::EtatCase::Touche;
                        afficher_plateau_double(&ma_grille, &radar, None);
                        break; 
                    }
                    _ => println!("Erreur réseau inattendue."),
                }
                
                println!("=========================================================================");
                println!("                            RADAR MIS À JOUR                            ");
                println!("=========================================================================\n");
                afficher_plateau_double(&ma_grille, &radar, None);
                
                mon_tour = false; 

            } else {
                println!("\n{}[RÉSEAU]{} En attente de l'attaque de {}{}{}...", C::CYAN, C::RESET, C::GRAS, nom_adversaire, C::RESET);

                let coord_ennemie = loop {
                    match recevoir_message(&mut *flux_tcp) {
                        Some(MessageReseau::Chat(msg)) => {
                            println!("{}[{}]{} {}{}{}", C::MAGENTA, nom_adversaire.to_uppercase(), C::RESET, C::ITALIQUE, msg, C::RESET);
                        }
                        Some(MessageReseau::Tir(coord)) => break Some(coord), 
                        None => break None,
                        _ => println!("{}[ALERTE]{} Message inattendu pendant le tour adverse.", C::ROUGE, C::RESET),
                    }
                };

                if let Some(coord) = coord_ennemie {
                    nettoyer_ecran();
                    let lettre = (b'A' + coord.x as u8) as char;
                    println!("\n{}[ALERTE]{} Tir ennemi détecté en {}{}{}{} !", C::ROUGE, C::RESET, C::GRAS, lettre, coord.y + 1, C::RESET);

                    if est_hote {
                        let resultat = ma_grille.tirer(coord);
                        if ma_grille.flotte_coulee() {
                            if envoyer_message(&mut *flux_tcp, &MessageReseau::RepFin).is_err() {
                                println!("\n{}[DÉCONNEXION]{} L'Amiral ennemi a fui la bataille !", C::ROUGE, C::RESET);
                                return;
                            }
                            println!("\n{}========================================================================={}", C::ROUGE, C::RESET);
                            println!("{}              DÉFAITE... Toute votre flotte a été anéantie.              {}", C::ROUGE, C::RESET);
                            println!("{}========================================================================={}\n", C::ROUGE, C::RESET);
                            afficher_plateau_double(&ma_grille, &radar, None);
                            break;
                        } else {
                            let reponse = match resultat { 
                                ResultatTir::Aleau => {
                                    println!("{}[RÉSULTAT]{} {}Plouf... C'est dans l'eau.{}", C::JAUNE, C::RESET, C::GRIS, C::RESET);
                                    MessageReseau::RepAleau
                                },
                                ResultatTir::Touche => {
                                    println!("{}[RÉSULTAT]{} {}BOUM ! Un de vos navires a été touché !{}", C::JAUNE, C::RESET, C::ROUGE, C::RESET);
                                    MessageReseau::RepTouche
                                },
                                ResultatTir::Coule(nom) => {
                                    println!("{}[RÉSULTAT]{} {}ATTAQUE DÉVASTATRICE ! Votre {} a été coulé !{}", C::JAUNE, C::RESET, C::ROUGE, nom, C::RESET);
                                    MessageReseau::RepCoule(nom)
                                },
                                _ => MessageReseau::RepAleau,  
                            };
                            if envoyer_message(&mut *flux_tcp, &reponse).is_err() {
                                println!("\n{}[DÉCONNEXION]{} L'Amiral ennemi a déserté le champ de bataille !", C::ROUGE, C::RESET);
                                return;
                            }
                        }
                    } else {
                        match recevoir_message(&mut *flux_tcp) {
                            Some(MessageReseau::RepAleau) => {
                                println!("{}[RÉSULTAT]{} {}Plouf... L'ennemi a raté.{}", C::JAUNE, C::RESET, C::GRIS, C::RESET);
                                ma_grille.cases[coord.y][coord.x].etat = modele::EtatCase::Aleau;
                            }
                            Some(MessageReseau::RepTouche) => {
                                println!("{}[RÉSULTAT]{} {}BOUM ! Vous avez été touché !{}", C::JAUNE, C::RESET, C::ROUGE, C::RESET);
                                ma_grille.cases[coord.y][coord.x].etat = modele::EtatCase::Touche;
                            }
                            Some(MessageReseau::RepCoule(nom)) => {
                                println!("{}[RÉSULTAT]{} {}DÉSASTRE ! Votre {} a été coulé !{}", C::JAUNE, C::RESET, C::ROUGE, nom, C::RESET);
                                ma_grille.cases[coord.y][coord.x].etat = modele::EtatCase::Touche;
                            }
                            Some(MessageReseau::RepFin) => {
                                ma_grille.cases[coord.y][coord.x].etat = modele::EtatCase::Touche;
                                println!("\n{}========================================================================={}", C::ROUGE, C::RESET);
                                println!("{}              DÉFAITE... Toute votre flotte a été anéantie.              {}", C::ROUGE, C::RESET);
                                println!("{}========================================================================={}\n", C::ROUGE, C::RESET);
                                afficher_plateau_double(&ma_grille, &radar, None);
                                break;
                            }
                            _ => {}
                        }
                    }
                    afficher_plateau_double(&ma_grille, &radar, None);
                } else {
                    println!("\n{}[DÉCONNEXION]{} L'Amiral ennemi a déserté le champ de bataille !", C::ROUGE, C::RESET);
                    return;
                }
                
                mon_tour = true; 
            }
        } 

        println!("\n{} --- FIN DES HOSTILITÉS ---{}", C::BLEU, C::RESET);
        print!("Voulez-vous proposer une revanche ? (O/N) : ");
        io::stdout().flush().unwrap();
        
        let mut choix = String::new();
        io::stdin().read_line(&mut choix).unwrap();
        let veut_rejouer = choix.trim().eq_ignore_ascii_case("o");

        if envoyer_message(&mut *flux_tcp, &MessageReseau::Revanche(veut_rejouer)).is_err() {
            println!("\n{}[DÉCONNEXION]{} Impossible d'envoyer la demande de revanche.", C::ROUGE, C::RESET);
            return; 
        }

        println!("{}[RÉSEAU]{} En attente de la décision de l'Amiral {}...", C::CYAN, C::RESET, nom_adversaire);

        match recevoir_message(&mut *flux_tcp) {
            Some(MessageReseau::Revanche(adversaire_veut_rejouer)) => {
                if veut_rejouer && adversaire_veut_rejouer {
                    println!("\n{}[SUCCÈS]{} La revanche est acceptée ! Nettoyage du pont...", C::VERT, C::RESET);
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    nettoyer_ecran();
                    continue; 
                } else {
                    println!("\n{}[DÉCONNEXION]{} La revanche a été refusée. Fin des transmissions.", C::ROUGE, C::RESET);
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    return;
                }
            }
            _ => {
                println!("\n{}[DÉCONNEXION]{} La communication a été rompue.", C::ROUGE, C::RESET);
                std::thread::sleep(std::time::Duration::from_secs(3));
                return;
            }
        }
    } 
}