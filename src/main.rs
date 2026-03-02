mod modele;
mod reseau;

use modele::{analyser_saisie, Coordonnee, Grille, Navire, Orientation, Partie, ResultatTir};
use reseau::{heberger_partie, rejoindre_partie, MessageReseau};
use std::io::{self, Write};

fn main() {
    println!("=====================================");
    println!("        BATAILLE NAVALE RÉSEAU       ");
    println!("=====================================\n");

    let flux_tcp; // Notre fameux "tuyau" réseau

    loop {
        println!("1. Héberger une partie (Attendre un adversaire)");
        println!("2. Rejoindre une partie (Se connecter à une adresse IP)");
        print!("Votre choix (1 ou 2) : ");
        io::stdout().flush().unwrap();

        let mut choix = String::new();
        io::stdin().read_line(&mut choix).unwrap();

        match choix.trim() {
            "1" => {
                // On lance le serveur
                if let Some(flux) = heberger_partie("3333") {
                    flux_tcp = flux;
                    break;
                }
            }
            "2" => {
                // On lance le client
                print!("Entrez l'adresse IP du serveur (ex: 127.0.0.1 pour jouer sur le même PC) : ");
                io::stdout().flush().unwrap();
                let mut ip = String::new();
                io::stdin().read_line(&mut ip).unwrap();
                
                if let Some(flux) = rejoindre_partie(ip.trim(), "3333") {
                    flux_tcp = flux;
                    break;
                }
            }
            _ => println!("Choix invalide, veuillez taper 1 ou 2.\n"),
        }
    }

    println!("\n>>> TEST RÉSEAU TERMINÉ AVEC SUCCÈS ! <<<");
    // On garde le "return" ici pour stopper le programme avant de lancer le "vrai" jeu pour l'instant
    return;

    // Initialisation de la partie avec les noms des deux commandants
    let mut partie = Partie::new("Goustan", "Appoline");

    // --- PHASE DE PRÉPARATION ---
    
    // Placement Joueur 1
    phase_placement(&mut partie.grille_j1, &partie.nom_j1);
    cacher_ecran();

    // Placement Joueur 2
    phase_placement(&mut partie.grille_j2, &partie.nom_j2);
    cacher_ecran();

    // --- PHASE DE COMBAT ---

    println!("=====================================");
    println!("          DÉBUT DU COMBAT !          ");
    println!("=====================================");

    loop {
        // On récupère le nom du joueur actuel (on en fait une String autonome pour éviter 
        // les conflits d'emprunt de mémoire avec le "borrow checker" de Rust)
        let nom_actuel = partie.nom_joueur_actuel().to_string(); 
        
        println!("\n=====================================");
        println!("      TOUR DE L'AMIRAL {}      ", nom_actuel.to_uppercase());
        println!("=====================================");

        println!("\n--- CARTE TACTIQUE ENNEMIE ---");
        // On affiche la grille adverse avec le brouillard de guerre activé (true)
        partie.grille_cible().afficher(true);

        // Tir
        print!("\nAmiral {}, entrez les coordonnées de tir (ex: B2) : ", nom_actuel);
        io::stdout().flush().unwrap();
        let mut saisie = String::new();
        io::stdin().read_line(&mut saisie).unwrap();

        let cible = match analyser_saisie(&saisie) {
            Some(c) => c,
            None => {
                println!("Coordonnées invalides ! Recommencez.");
                continue; // On ne change pas de tour
            }
        };

        // Exécution du tir sur la grille adverse
        println!(">>> Tir en cours...");
        match partie.grille_cible().tirer(cible) {
            ResultatTir::Aleau => println!("Résultat : Plouf... C'est dans l'eau."),
            ResultatTir::Touche => println!("Résultat : BOUM ! Navire touché !"),
            ResultatTir::Coule(nom) => println!("Résultat : TOUCHÉ ET COULÉ ! Le {} est détruit !", nom),
            ResultatTir::DejaJoue => {
                println!("Résultat : Inutile, vous avez déjà tiré ici. Recommencez.");
                continue; // On ne change pas de tour si le joueur s'est trompé de case
            },
            ResultatTir::HorsLimite => continue,
        }

        println!("\n--- RÉSULTAT DE L'ATTAQUE ---");
        partie.grille_cible().afficher(true); // Toujours en "true" pour garder le brouillard de guerre !

        // Vérification de la victoire
        if partie.grille_cible().flotte_coulee() {
            println!("\n=================================================");
            println!("VICTOIRE ! L'Amiral {} a remporté la bataille !", nom_actuel.to_uppercase());
            println!("=================================================");
            
            println!("\n--- CARTE FINALE DE L'ENNEMI ---");
            partie.grille_cible().afficher(false); // On dévoile tout !
            break;
        }

        // Fin du tour : on bascule l'état du joueur et on cache l'écran pour le suivant
        partie.changer_tour();
        cacher_ecran();
    }
}

fn demander_orientation() -> Orientation {
    loop {
        print!("Orientation (H pour Horizontal, V pour Vertical) : ");
        io::stdout().flush().unwrap();
        
        let mut saisie = String::new();
        io::stdin().read_line(&mut saisie).expect("Erreur de lecture");

        // On nettoie la saisie et on vérifie
        match saisie.trim().to_uppercase().as_str() {
            "H" => return Orientation::Horizontal,
            "V" => return Orientation::Vertical,
            _ => println!("Saisie invalide. Veuillez taper 'H' ou 'V'."),
        }
    }
}

fn phase_placement(grille: &mut Grille, nom_joueur: &str) {
    // On définit la flotte standard du Touché Coulé (Nom, Taille)
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
            grille.afficher(false); // On met "false" car le joueur doit voir ses propres bateaux !
            
            println!("\nAmiral, où voulez-vous placer le {} (Taille : {}) ?", nom, taille);
            
            // 1. Demander les coordonnées
            print!("Coordonnées de la proue (ex: A1) : ");
            io::stdout().flush().unwrap();
            let mut saisie_coord = String::new();
            io::stdin().read_line(&mut saisie_coord).expect("Erreur de lecture");

            let coord = match analyser_saisie(&saisie_coord) {
                Some(c) => c,
                None => {
                    println!("Coordonnées invalides ! Recommencez.");
                    continue; // On relance la boucle pour ce même bateau
                }
            };

            // 2. Demander l'orientation
            let orientation = demander_orientation();

            // 3. Créer le navire et tenter de le placer
            // On déréférence la taille avec *taille car c'est une référence (&usize) issue de l'itérateur
            let nouveau_navire = Navire::new(nom, *taille, coord, orientation);
            
            match grille.placer_navire(nouveau_navire) {
                Ok(_) => {
                    println!(">>> {} positionné avec succès !", nom);
                    break; // Le bateau est placé, on casse cette boucle pour passer au bateau suivant !
                }
                Err(message) => {
                    // Si ça déborde ou chevauche, on affiche l'erreur et on laisse la boucle recommencer
                    println!("ERREUR : {}", message);
                    println!("Veuillez choisir un autre emplacement.");
                }
            }
        }
    }
    println!("\n--- VOTRE CARTE ACTUELLE ---");        
    grille.afficher(false); // On met "false" car le joueur doit voir ses propres bateaux !
    println!("\nTous les navires sont en position !");
}

fn cacher_ecran() {
    println!("\nAppuyez sur Entrée pour cacher l'écran et passer le tour...");
    let mut attente = String::new();
    io::stdin().read_line(&mut attente).unwrap();
    
    // On imprime 50 sauts de ligne pour "nettoyer" le terminal
    for _ in 0..50 { 
        println!(); 
    }
}