mod modele;
mod reseau;

use modele::{analyser_saisie, Coordonnee, Grille, Navire, Orientation, ResultatTir, TAILLE_GRILLE};
use reseau::{envoyer_message, heberger_partie, recevoir_message, rejoindre_partie, MessageReseau};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::{self, stdout, Write};

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

/// Nouvelle fonction remplacant totalement l'ancienne saisie textuelle ("B2")
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
                    KeyCode::Down => { if curseur.y < TAILLE_GRILLE - 1 { curseur.y += 1; } }
                    KeyCode::Left => { if curseur.x > 0 { curseur.x -= 1; } }
                    KeyCode::Right => { if curseur.x < TAILLE_GRILLE - 1 { curseur.x += 1; } }
                    
                    // La touche R pour pivoter 
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        est_horizontal = !est_horizontal; 
                        message_erreur.clear(); // On efface l'erreur precedente si on pivote
                    }
                    
                    KeyCode::Enter => {
                        // On traduit notre booleen en orientation du modele
                        let orientation = if est_horizontal { 
                            Orientation::Horizontal 
                        } else { 
                            Orientation::Vertical 
                        };
                        
                        // On cree le navire
                        let nouveau_navire = Navire::new(nom, taille, curseur, orientation);

                        // On tente de le placer
                        match grille.placer_navire(nouveau_navire) {
                            Ok(_) => {
                                // On sort du mode brut et on termine la fonction
                                disable_raw_mode().unwrap();
                                return; 
                            }
                            Err(msg) => {
                                // Échec : (débordement ou chevauchement) on stocke l'erreur pour l'afficher
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
            }
        }
    }
}

fn phase_placement(grille: &mut Grille, nom_joueur: &str) {
    let flotte_a_placer = [
        ("Porte-avions", 5),
        ("Croiseur", 4),
        ("Contre-torpilleur", 3),
        ("Sous-marin", 3),
        ("Torpilleur", 2),
    ];

    // Pour chaque bateau de la flotte on lance l'interface dediee
    for (nom, taille) in flotte_a_placer.iter() {
        placer_navire_interactif(grille, nom, *taille);
    }

    // Affichage final une fois tous les bateaux places
    let mut terminal = stdout();
    disable_raw_mode().unwrap(); 
    execute!(terminal, cursor::MoveTo(0, 0), Clear(ClearType::FromCursorDown)).unwrap();
    
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