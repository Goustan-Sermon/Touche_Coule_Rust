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
                print!("Adresse IP du serveur : ");
                io::stdout().flush().unwrap();
                let mut ip = String::new();
                io::stdin().read_line(&mut ip).unwrap();
                ip_serveur = ip.trim().to_string(); // On sauvegarde l'IP
                
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
                println!("Fermeture du Centre de Commandement. Au revoir Amiral {} !\n", mon_nom.to_uppercase());
                std::process::exit(0);
            }
            _ => {
                println!("Choix invalide. Appuyez sur Entrée pour réessayer...");
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
                println!("\n[ERREUR] {}", msg);
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
                    println!("\n[ERREUR] Impossible de créer le salon.");
                } else {
                    println!("\n[ERREUR] Impossible de joindre le serveur. Vérifiez l'IP.");
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
                    println!("[BAN] Tentative de connexion bloquée pour {}", ip_client);
                    let _ = envoyer_message(&mut *flux, &MessageReseau::RepAuthFail);
                    continue; // On refuse la connexion et on retourne au menu d'attente pour le prochain client
                }
            }

            println!("[AUTH] En attente de l'authentification de {}...", ip_client);
            
            match recevoir_message(&mut *flux) {
                Some(MessageReseau::Hello(nom_client, code_client)) => {
                    if code_client == code_secret {
                        println!("[SUCCÈS] Authentification réussie pour {}.", nom_client);
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
                        println!("[ALERTE] Mauvais code ({}/3) de {}", *n, ip_client);
                        let _ = envoyer_message(&mut *flux, &MessageReseau::RepAuthFail);
                    }
                }
                _ => println!("[ALERTE] Déconnexion inattendue pendant l'authentification."),
            }
        } else {
            // Logique du Client
            let mon_hello = MessageReseau::Hello(mon_nom.clone(), code_secret.clone());
            envoyer_message(&mut *flux, &mon_hello).unwrap();
            
            match recevoir_message(&mut *flux) {
                Some(MessageReseau::RepAuthOk) => {
                    println!("[SUCCÈS] Accès autorisé !");
                    if let Some(MessageReseau::Hello(nom_hote, _)) = recevoir_message(&mut *flux) {
                        break (flux, nom_hote); // Succes pour le client aussi
                    }
                }
                _ => {
                    println!("[BAN] Le code de salon est incorrect ou vous êtes banni.");
                    std::process::exit(1); // Le client ferme son jeu
                }
            }
        }
    };

    println!("\n[ALLIANCE] Connexion sécurisée avec l'Amiral {} !\n", nom_adversaire.to_uppercase());

    // 4. La phase de placement (Chacun le fait de son cote localement)
    let mut ma_grille = Grille::new();
    let mut radar = Grille::new(); // Grille vide pour noter nos tirs
    
    phase_placement(&mut ma_grille, &mon_nom);

    println!("\nEn attente que l'Amiral {} termine son déploiement...", nom_adversaire);
    
    let mut mon_tour = est_hote; 

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
            println!("\n[CIBLE] Verrouillage des missiles sur {}{}...", lettre, chiffre);

            // 1. On envoie la coordonnee a l'adversaire
            let _ = envoyer_message(&mut flux_tcp, &MessageReseau::Tir(cible));
            println!("[RÉSEAU] Tir envoyé ! En attente du rapport de dégâts...");

            // 2. On attend sa reponse pour mettre a jour notre radar
            match recevoir_message(&mut flux_tcp) {
                Some(MessageReseau::RepAleau) => {
                    println!("[RÉSULTAT] \x1b[90mPlouf... C'est dans l'eau.\x1b[0m\n");
                    radar.cases[cible.y][cible.x].etat = modele::EtatCase::Aleau;
                }
                Some(MessageReseau::RepTouche) => {
                    println!("[RÉSULTAT] \x1b[31mBOUM ! Vous avez touché un navire !\x1b[0m\n");
                    radar.cases[cible.y][cible.x].etat = modele::EtatCase::Touche;
                }
                Some(MessageReseau::RepCoule(nom)) => {
                    println!("[RÉSULTAT] \x1b[31mTOUCHÉ ET COULÉ ! Vous avez détruit le {} !\x1b[0m\n", nom);
                    radar.cases[cible.y][cible.x].etat = modele::EtatCase::Touche;
                }
                Some(MessageReseau::RepFin) => {
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
            println!("\n>>> En attente de l'attaque de {}...", nom_adversaire);

            match recevoir_message(&mut flux_tcp) {
                Some(MessageReseau::Tir(coord)) => {
                    nettoyer_ecran();
                    let lettre = (b'A' + coord.x as u8) as char;
                    println!("\n[ALERTE] Tir ennemi détecté en {}{} !", lettre, coord.y + 1);

                    let resultat = ma_grille.tirer(coord);

                    if ma_grille.flotte_coulee() {
                        let _ = envoyer_message(&mut flux_tcp, &MessageReseau::RepFin);
                        println!("\n\x1b[1;31m=========================================================================\x1b[0m");
                        println!("\x1b[1;31m              DÉFAITE... Toute votre flotte a été anéantie.              \x1b[0m");
                        println!("\x1b[1;31m=========================================================================\x1b[0m\n");
                        afficher_plateau_double(&ma_grille, &radar, None);
                        break; 
                    }

                    // On affiche le resultat de l'impact en couleur et on prepare la reponse
                    let reponse = match resultat {
                        ResultatTir::Aleau => {
                            println!("[RÉSULTAT] \x1b[90mPlouf... C'est dans l'eau.\x1b[0m\n");
                            MessageReseau::RepAleau
                        },
                        ResultatTir::Touche => {
                            println!("[RÉSULTAT] \x1b[31mBOUM ! Un de vos navires a été touché !\x1b[0m\n");
                            MessageReseau::RepTouche
                        },
                        ResultatTir::Coule(nom) => {
                            println!("[RÉSULTAT] \x1b[31mATTAQUE DÉVASTATRICE ! Votre {} a été coulé !\x1b[0m\n", nom);
                            MessageReseau::RepCoule(nom)
                        },
                        _ => MessageReseau::RepAleau, 
                    };

                    let _ = envoyer_message(&mut flux_tcp, &reponse);
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

fn afficher_guide() {
    let mut terminal = io::stdout();
    execute!(terminal, cursor::MoveTo(0, 0), Clear(ClearType::All)).unwrap();

    println!("===========================================================");
    println!("                  GUIDE DE L'AMIRAL                        ");
    println!("===========================================================\n");
    
    println!(" --- CONNEXION RÉSEAU & ADRESSE IP ---");
    println!("L'Hôte doit communiquer son adresse IP locale au joueur qui le rejoint.");
    println!("  - Sous Windows : Ouvrez l'invite de commande et tapez 'ipconfig' (cherchez l'adresse IPv4).");
    println!("  - Sous Linux/Mac (ou WSL) : Ouvrez le terminal et tapez 'hostname -I' ou 'ip a'.");
    println!("  (Astuce : Tapez '127.0.0.1' pour jouer contre vous-même sur le même PC !)\n");

    println!(" --- COMMANDES DE JEU ---");
    println!("  - FLÈCHES : Déplacer le curseur de ciblage ou votre bateau.");
    println!("  - TOUCHE 'R' : Faire pivoter le navire lors du déploiement.");
    println!("  - ENTRÉE : Valider un tir ou confirmer le placement d'un navire.\n");

    println!(" --- DÉROULEMENT D'UN TOUR ---");
    println!("  1. C'est votre tour : Vous visez sur le radar et tirez.");
    println!("  2. Le jeu vous informe immédiatement du résultat (À l'eau, Touché, Coulé).");
    println!("  3. L'adversaire voit votre tir s'abattre sur sa propre grille et encaisse les dégâts.");
    println!("  4. Les rôles s'inversent ! Le suspense est total.\n");

    println!(" --- LÉGENDE DU RADAR ---");
    println!("  [\x1b[34m~\x1b[0m] : Eau inexplorée        [\x1b[90mO\x1b[0m] : Tir raté (Plouf !)");
    println!("  [\x1b[32mB\x1b[0m] : Vos navires           [\x1b[31mX\x1b[0m] : Navire touché !");

    println!("\n===========================================================");
    println!("Appuyez sur ENTRÉE pour retourner au Centre de Commandement...");
    
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