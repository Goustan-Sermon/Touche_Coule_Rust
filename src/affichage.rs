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

// --- PALETTE DE COULEURS ANSI ---
pub struct C;
impl C {
    pub const CYAN: &'static str = "\x1b[1;36m";
    pub const VERT: &'static str = "\x1b[1;32m";
    pub const ROUGE: &'static str = "\x1b[1;31m";
    pub const JAUNE: &'static str = "\x1b[1;33m";
    pub const MAGENTA: &'static str = "\x1b[1;35m";
    pub const BLEU: &'static str = "\x1b[1;34m";
    pub const GRIS: &'static str = "\x1b[90m";
    pub const GRAS: &'static str = "\x1b[1m";
    pub const ITALIQUE: &'static str = "\x1b[3m";
    pub const RESET: &'static str = "\x1b[0m";
}

pub enum ActionTour {
    Tir(Coordonnee),
    Chat(String),
    Quitter,
}

pub fn afficher_guide() {
    let mut terminal = io::stdout();
    execute!(terminal, cursor::MoveTo(0, 0), Clear(ClearType::All)).unwrap();

    println!("{}==========================================================={}", C::CYAN, C::RESET);
    println!("{}                  GUIDE DE L'AMIRAL                        {}", C::CYAN, C::RESET);
    println!("{}==========================================================={}\n", C::CYAN, C::RESET);
    
    println!("{} --- CONNEXION RÉSEAU & ADRESSE IP ---{}", C::BLEU, C::RESET);
    println!("L'Hôte doit communiquer son adresse IP locale au joueur qui le rejoint.");
    println!("  - Sous {}Windows{} : Ouvrez l'invite de commande et tapez '{}ipconfig{}' (cherchez l'adresse IPv4).", C::GRAS, C::RESET, C::JAUNE, C::RESET);
    println!("  - Sous {}Linux/Mac{} : Ouvrez le terminal et tapez '{}hostname -I{}' ou '{}ip a{}'.", C::GRAS, C::RESET, C::JAUNE, C::RESET, C::JAUNE, C::RESET);
    println!("  ({}Astuce{} : Tapez '{}127.0.0.1{}' pour jouer contre vous-même sur le même PC !)\n", C::MAGENTA, C::RESET, C::JAUNE, C::RESET);

    println!("{} --- COMMANDES DE JEU ---{}", C::BLEU, C::RESET);
    println!("  - {}FLÈCHES{} : Déplacer le curseur de ciblage ou votre bateau.", C::JAUNE, C::RESET);
    println!("  - {}TOUCHE 'R'{} : Faire pivoter le navire lors du déploiement.", C::JAUNE, C::RESET);
    println!("  - {}ENTRÉE{} : Valider un tir ou confirmer le placement d'un navire.\n", C::JAUNE, C::RESET);

    println!("{} --- DÉROULEMENT D'UN TOUR ---{}", C::BLEU, C::RESET);
    println!("  {}1.{} C'est votre tour : Vous visez sur le radar et tirez.", C::VERT, C::RESET);
    println!("  {}2.{} Le jeu vous informe immédiatement du résultat (À l'eau, Touché, Coulé).", C::VERT, C::RESET);
    println!("  {}3.{} L'adversaire voit votre tir s'abattre sur sa propre grille et encaisse les dégâts.", C::VERT, C::RESET);
    println!("  {}4.{} Les rôles s'inversent ! Le suspense est total.\n", C::VERT, C::RESET);

    println!("{} --- LÉGENDE DU RADAR ---{}", C::BLEU, C::RESET);
    println!("  [{}~{}] : Eau inexplorée        [{}O{}] : Tir raté (Plouf !)", C::BLEU, C::RESET, C::GRIS, C::RESET);
    println!("  [{}B{}] : Vos navires           [{}X{}] : Navire touché !", C::VERT, C::RESET, C::ROUGE, C::RESET);

    println!("\n{}==========================================================={}", C::CYAN, C::RESET);
    println!("Appuyez sur {}ENTRÉE{} pour retourner au Centre de Commandement...", C::JAUNE, C::RESET);
    
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
                        print!("\n{}[CANAL RADIO]{} Transmission : ", C::MAGENTA, C::RESET);
                        io::stdout().flush().unwrap();
                        let mut msg = String::new();
                        match io::stdin().read_line(&mut msg) {
                            Ok(_) => {
                                if !msg.trim().is_empty() {
                                    return ActionTour::Chat(msg.trim().to_string());
                                }
                            }
                            Err(_) => {
                                println!("{}[ERREUR]{} Saisie invalide ou caractère non supporté.", C::ROUGE, C::RESET);
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
    let mut est_horizontal = true;
    let mut message_erreur = String::new(); 

    loop {
        disable_raw_mode().unwrap();
        let mut terminal = stdout();
        
        execute!(terminal, cursor::MoveTo(0, 0), Clear(ClearType::FromCursorDown)).unwrap();
        
        println!("=================================================");
        println!(" DÉPLOIEMENT : {} (Taille : {})", nom.to_uppercase(), taille);
        println!(" Flèches : Déplacer | 'R' : Pivoter | Entrée : Valider");
        
        let texte_orientation = if est_horizontal { "Horizontale" } else { "Verticale" };
        println!(" Orientation actuelle : {}", texte_orientation);
        println!("=================================================\n");

        if !message_erreur.is_empty() {
            println!("{}[ERREUR]{} {}\n", C::ROUGE, C::RESET, message_erreur);
        } else {
            println!("\n"); 
        }

        let orientation_fantome = if est_horizontal { 
            Orientation::Horizontal 
        } else { 
            Orientation::Vertical 
        };
        
        let navire_fantome = Navire::new(nom, taille, curseur, orientation_fantome);
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

                let limite_x = if est_horizontal { TAILLE_GRILLE - taille } else { TAILLE_GRILLE - 1 };
                let limite_y = if est_horizontal { TAILLE_GRILLE - 1 } else { TAILLE_GRILLE - taille };

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
            println!("{}[ERREUR]{} {}\n", C::ROUGE, C::RESET, message_erreur);
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
        placer_flotte_aleatoire(grille);
    } else {
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