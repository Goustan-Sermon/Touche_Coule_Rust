// src/affichage.rs

use crate::modele::{Coordonnee, Grille, Navire, Orientation, TAILLE_GRILLE, EtatCase};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::{self, stdout, Write};
use rand::RngExt;

pub enum ActionTour {
    Tir(Coordonnee),
    Chat(String),
    Quitter,
}

pub fn afficher_guide() {
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

pub fn afficher_plateau_double(ma_grille: &Grille, radar: &Grille, curseur_radar: Option<Coordonnee>) {
    let lignes_flotte = ma_grille.vers_lignes(false, None, None);
    let lignes_radar = radar.vers_lignes(true, curseur_radar, None);

    println!("        ÉTAT DE VOTRE FLOTTE                       RADAR TACTIQUE        ");
    println!("=========================================================================");
    
    for (g, d) in lignes_flotte.iter().zip(lignes_radar.iter()) {
        println!("{}   |   {}", g, d);
    }
}

pub fn choisir_action_interactive(ma_grille: &Grille, radar: &Grille) -> ActionTour {
    // Securite anti ghosting
    enable_raw_mode().unwrap();
    
    while crossterm::event::poll(std::time::Duration::from_millis(50)).unwrap() {
        let _ = crossterm::event::read().unwrap();
    }
    
    disable_raw_mode().unwrap();

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
                cursor::MoveUp(19), 
                cursor::MoveToColumn(0), 
                Clear(ClearType::FromCursorDown)
            ).unwrap();
        }

        println!("\n=========================================================================");
        println!("                              À VOTRE TOUR!                              ");
        println!("   FLÈCHES : Déplacer | ENTRÉE : Tirer | 'C' : Message | 'Q' : Quitter   ");
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
                        if *etat_case == EtatCase::Touche || *etat_case == EtatCase::Aleau {
                            continue;
                        }
                        disable_raw_mode().unwrap();
                        return ActionTour::Tir(curseur);
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        disable_raw_mode().unwrap();
                        print!("\n\x1b[1;35m[CANAL RADIO]\x1b[0m Transmission : ");
                        io::stdout().flush().unwrap();
                        let mut msg = String::new();
                        match io::stdin().read_line(&mut msg) {
                            Ok(_) => {
                                if !msg.trim().is_empty() {
                                    return ActionTour::Chat(msg.trim().to_string());
                                }
                            }
                            Err(_) => {
                                println!("\x1b[1;31m[ERREUR]\x1b[0m Saisie invalide ou caractère non supporté.");
                                std::thread::sleep(std::time::Duration::from_secs(2));
                            }
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                        disable_raw_mode().unwrap();
                        return ActionTour::Quitter;
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn placer_navire_interactif(grille: &mut Grille, nom: &str, taille: usize) {
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

pub fn placer_flotte_aleatoire(grille: &mut Grille) {
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

pub fn phase_placement(grille: &mut Grille, nom_joueur: &str) {
    let mut terminal = stdout();
    disable_raw_mode().unwrap();
    
    let mut message_erreur = String::new();

    let choix = loop {
        execute!(terminal, cursor::MoveTo(0, 0), Clear(ClearType::All)).unwrap();
        
        println!("=====================================");
        println!("   DÉPLOIEMENT : AMIRAL {}   ", nom_joueur.to_uppercase());
        println!("=====================================\n");
        
        if !message_erreur.is_empty() {
            println!("\x1b[1;31m[ERREUR]\x1b[0m {}\n", message_erreur);
        }
        
        println!("Comment souhaitez-vous déployer votre flotte ?");
        println!("1. Manuellement (Flèches du clavier)");
        println!("2. Aléatoirement (Placement express)");
        print!("Votre choix (1 ou 2) : ");
        io::stdout().flush().unwrap();
        
        let mut saisie = String::new();
        io::stdin().read_line(&mut saisie).unwrap();
        
        match saisie.trim() {
            "1" => break "1",
            "2" => break "2",
            _ => message_erreur = "Choix invalide. Veuillez saisir 1 ou 2.".to_string(),
        }
    };
    
    if choix == "2" {
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

pub fn nettoyer_ecran() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}