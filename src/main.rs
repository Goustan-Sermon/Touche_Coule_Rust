mod modele;
mod reseau;

use modele::{Coordonnee, Grille, Navire, Orientation, ResultatTir, TAILLE_GRILLE};
use reseau::{envoyer_message, heberger_partie, recevoir_message, rejoindre_partie, MessageReseau};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::{self, stdout, Write};
use rand::RngExt;

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
    let mut code_secret = String::new();

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
                
                print!("Code secret du salon : ");
                io::stdout().flush().unwrap();
                let mut saisie_code = String::new();
                io::stdin().read_line(&mut saisie_code).unwrap();
                code_secret = saisie_code.trim().to_string(); 
                
                if let Some(flux) = rejoindre_partie(ip.trim(), "3333") {
                    flux_tcp = flux;
                    est_hote = false;
                    break;
                }
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

    // --- 3. AUTHENTIFICATION ET ÉCHANGE DES PSEUDOS ---
    let nom_adversaire: String;

    if est_hote {
        // L'Hôte agit comme un videur de boîte de nuit
        println!("En attente de l'authentification du client...");
        
        match recevoir_message(&mut *flux_tcp) {
            Some(MessageReseau::Hello(nom_client, code_client)) => {
                // On vérifie si le code envoyé par le client correspond au nôtre !
                if code_client == code_secret {
                    println!(">>> Authentification réussie pour {}.", nom_client);
                    envoyer_message(&mut *flux_tcp, &MessageReseau::RepAuthOk).unwrap();
                    nom_adversaire = nom_client;
                    
                    // On envoie notre propre pseudo au client en retour (le code n'a plus d'importance ici)
                    let mon_hello = MessageReseau::Hello(mon_nom.clone(), "".to_string());
                    envoyer_message(&mut *flux_tcp, &mon_hello).unwrap();
                } else {
                    println!("ALERTE : Code incorrect ({}) fourni par {}. Fermeture de la connexion.", code_client, nom_client);
                    envoyer_message(&mut *flux_tcp, &MessageReseau::RepAuthFail).unwrap();
                    std::process::exit(1); // On coupe tout !
                }
            }
            _ => {
                println!("Erreur de protocole d'authentification.");
                std::process::exit(1);
            }
        }
    } else {
        // Le Client présente sa carte d'identité et son mot de passe
        let mon_hello = MessageReseau::Hello(mon_nom.clone(), code_secret.clone());
        envoyer_message(&mut *flux_tcp, &mon_hello).unwrap();
        
        // On attend le verdict du videur (le serveur)
        match recevoir_message(&mut *flux_tcp) {
            Some(MessageReseau::RepAuthOk) => {
                println!(">>> Accès autorisé !");
                // Le serveur nous accepte, on attend qu'il nous donne son pseudo
                match recevoir_message(&mut *flux_tcp) {
                    Some(MessageReseau::Hello(nom_hote, _)) => {
                        nom_adversaire = nom_hote;
                    }
                    _ => panic!("Le serveur n'a pas envoyé son pseudo."),
                }
            }
            Some(MessageReseau::RepAuthFail) => {
                println!("ACCÈS REFUSÉ : Le code de salon est incorrect.");
                std::process::exit(1); // On quitte le jeu
            }
            _ => {
                println!("Erreur de protocole inattendue.");
                std::process::exit(1);
            }
        }
    }

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
            // --- C'EST MON TOUR ---
            println!("\n=====================================");
            println!("           À VOTRE TOUR !            ");
            println!("=====================================");
            
            // Afficher la grille et gerer les fleches
            let cible = choisir_coordonnee_interactive(&radar, false);

            // Une fois qu'on a appuye sur Entree, on reaffiche proprement le radar pour voir oui on a tire 
            println!("\n--- TIR VERROUILLÉ EN {:?} ---", cible);
            radar.afficher(false, None, None);

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
                    println!("VICTOIRE TOTALE ! La flotte ennemie est détruite !");
                    println!("=================================================");
                    radar.cases[cible.y][cible.x].etat = modele::EtatCase::Touche;
                    radar.afficher(false, None, None);
                    break; // Fin du jeu 
                }
                _ => println!("Erreur réseau inattendue."),
            }
            
            println!("\n--- RADAR MIS À JOUR ---");
            radar.afficher(false, None, None); 
            
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
                        println!("  DÉFAITE... Toute votre flotte a été anéantie.  ");
                        println!("=================================================");
                        ma_grille.afficher(false, None, None);
                        break; // Fin du jeu 
                    }

                    // Sinon on renvoie le résultat normal a l'adversaire
                    let reponse = match resultat {
                        ResultatTir::Aleau => MessageReseau::RepAleau,
                        ResultatTir::Touche => MessageReseau::RepTouche,
                        ResultatTir::Coule(nom) => MessageReseau::RepCoule(nom),
                        // Si on tire deux fois au meme endroit ou hors limite on dit "A l'eau" pour simplifier la logique reseau
                        _ => MessageReseau::RepAleau, 
                    };

                    let _ = envoyer_message(&mut flux_tcp, &reponse);
                    
                    println!("\n--- ÉTAT DE VOTRE FLOTTE ---");
                    ma_grille.afficher(false, None, None); // On regarde les degats
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
    println!("  [~] : Eau inexplorée        [O] : Tir raté (Plouf !)");
    println!("  [B] : Vos navires           [X] : Navire touché !");

    println!("\n===========================================================");
    println!("Appuyez sur ENTRÉE pour retourner au Centre de Commandement...");
    
    let mut attente = String::new();
    io::stdin().read_line(&mut attente).unwrap();
}

/// Nouvelle fonction remplacant l'ancienne saisie textuelle ("B2")
fn choisir_coordonnee_interactive(grille: &Grille, cacher_bateaux: bool) -> Coordonnee {
    let mut curseur = Coordonnee { x: 0, y: 0 };
    let mut premiere_fois = true;

    loop {
        disable_raw_mode().unwrap();
        let mut terminal = stdout();
        
        if premiere_fois {
            premiere_fois = false; // La premiere fois on affiche normalement
        } else {
            // Les fois suivantes on remonte de 15 lignes et on efface juste vers le bas pour redessiner proprement
            execute!(
                terminal, 
                cursor::MoveUp(15), 
                cursor::MoveToColumn(0), 
                Clear(ClearType::FromCursorDown)
            ).unwrap();
        }
        
        println!("=================================================");
        println!("    DÉPLACEZ LE CURSEUR ET APPUYEZ SUR ENTRÉE    ");
        println!("=================================================\n");
        
        grille.afficher(cacher_bateaux, Some(curseur), None);
        
        enable_raw_mode().unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Up => { if curseur.y > 0 { curseur.y -= 1; } }
                    KeyCode::Down => { if curseur.y < TAILLE_GRILLE - 1 { curseur.y += 1; } }
                    KeyCode::Left => { if curseur.x > 0 { curseur.x -= 1; } }
                    KeyCode::Right => { if curseur.x < TAILLE_GRILLE - 1 { curseur.x += 1; } }
                    KeyCode::Enter => {
                        let etat_case = &grille.cases[curseur.y][curseur.x].etat;
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