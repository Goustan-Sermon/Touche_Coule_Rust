pub const TAILLE_GRILLE: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EtatCase {
    Vide,
    Bateau,
    Touche,
    Aleau,
}

#[derive(Debug, PartialEq)]
pub enum ResultatTir {
    Aleau,
    Touche,
    Coule(String),
    DejaJoue,
    HorsLimite,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordonnee {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Case {
    pub coord: Coordonnee,
    pub etat: EtatCase,
}

#[derive(Debug)]
pub struct Grille {
    pub cases: [[Case; TAILLE_GRILLE]; TAILLE_GRILLE],
    pub navires: Vec<Navire>,
}

impl Grille {

    pub fn new() -> Self {
        let mut cases = [[Case {
            coord: Coordonnee { x: 0, y: 0 },
            etat: EtatCase::Vide,
        }; TAILLE_GRILLE]; TAILLE_GRILLE];

        for y in 0..TAILLE_GRILLE {
            for x in 0..TAILLE_GRILLE {
                cases[y][x].coord = Coordonnee { x, y };
            }
        }

        Grille { 
            cases,
            navires: Vec::new(), // On initialise une liste de navires vide
        }
    }

    pub fn afficher(&self, cacher_bateaux: bool) {
        print!("   "); 
        for x in 0..TAILLE_GRILLE {
            let lettre = (b'A' + x as u8) as char;
            print!("{} ", lettre);
        }
        println!(); 

        for y in 0..TAILLE_GRILLE {
            print!("{:2} ", y + 1);

            for x in 0..TAILLE_GRILLE {
                let symbole = match self.cases[y][x].etat {
                    EtatCase::Vide => '~',
                    EtatCase::Bateau => {
                        if cacher_bateaux {
                            '~' // On le camoufle en eau
                        } else {
                            'B' // On l'affiche si on a les droits
                        }
                    }
                    EtatCase::Touche => 'X',
                    EtatCase::Aleau => 'O',
                };
                print!("{} ", symbole);
            }
            println!();
        }
    }

    pub fn placer_navire(&mut self, navire: Navire) -> Result<(), &'static str> {
        let mut x = navire.coord_depart.x;
        let mut y = navire.coord_depart.y;

        // ETAPE 1 : Verifier si le bateau sort de la grille
        match navire.orientation {
            Orientation::Horizontal => {
                if x + navire.taille > TAILLE_GRILLE {
                    return Err("Le navire sort de la grille à l'horizontale !");
                }
            }
            Orientation::Vertical => {
                if y + navire.taille > TAILLE_GRILLE {
                    return Err("Le navire sort de la grille à la verticale !");
                }
            }
        }

        // ETAPE 2 : Verifier les collisions (case deja prise)
        let mut check_x = x;
        let mut check_y = y;
        for _ in 0..navire.taille {
            if self.cases[check_y][check_x].etat != EtatCase::Vide {
                return Err("Le navire chevauche un autre bateau !");
            }
            // On avance pour verifier la case suivante
            match navire.orientation {
                Orientation::Horizontal => check_x += 1,
                Orientation::Vertical => check_y += 1,
            }
        }

        // ETAPE 3 : Tout est bon, on place le bateau !
        for _ in 0..navire.taille {
            self.cases[y][x].etat = EtatCase::Bateau;
            match navire.orientation {
                Orientation::Horizontal => x += 1,
                Orientation::Vertical => y += 1,
            }
        }

        // ETAPE 4 : On stocke le navire dans la liste de la grille
        self.navires.push(navire);

        Ok(())
    }

    pub fn tirer(&mut self, coord: Coordonnee) -> ResultatTir {
        if coord.x >= TAILLE_GRILLE || coord.y >= TAILLE_GRILLE {
            return ResultatTir::HorsLimite;
        }

        match self.cases[coord.y][coord.x].etat {
            EtatCase::Vide => {
                self.cases[coord.y][coord.x].etat = EtatCase::Aleau;
                ResultatTir::Aleau
            }
            EtatCase::Bateau => {
                self.cases[coord.y][coord.x].etat = EtatCase::Touche;
                
                // On cherche quel bateau a ete touche
                for navire in &mut self.navires {
                    if navire.occupe(coord) {
                        navire.touches += 1; // On ajoute un degat
                        
                        if navire.est_coule() {
                            // Si il coule on renvoie "Coule" avec une copie de son nom
                            return ResultatTir::Coule(navire.nom.clone());
                        } else {
                            return ResultatTir::Touche;
                        }
                    }
                }
                ResultatTir::Touche // Securite au cas ou
            }
            EtatCase::Touche | EtatCase::Aleau => ResultatTir::DejaJoue,
        }
    }

    pub fn flotte_coulee(&self) -> bool {
        self.navires.iter().all(|navire| navire.est_coule())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct Navire {
    pub nom: String,
    pub taille: usize,
    pub coord_depart: Coordonnee,
    pub orientation: Orientation,
    pub touches: usize, // Compteur pour savoir combien de ses cases sont touchees
}

impl Navire {

    pub fn new(nom: &str, taille: usize, coord_depart: Coordonnee, orientation: Orientation) -> Self {
        Navire {
            nom: nom.to_string(), // On convertit le texte statique en String dynamique
            taille,
            coord_depart,
            orientation,
            touches: 0, // Un bateau neuf n'a aucun dégât
        }
    }

    pub fn est_coule(&self) -> bool {
        self.touches >= self.taille
    }

    pub fn occupe(&self, cible: Coordonnee) -> bool {
        let mut cx = self.coord_depart.x;
        let mut cy = self.coord_depart.y;

        for _ in 0..self.taille {
            if cx == cible.x && cy == cible.y {
                return true;
            }
            match self.orientation {
                Orientation::Horizontal => cx += 1,
                Orientation::Vertical => cy += 1,
            }
        }
        false
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Joueur {
    J1,
    J2,
}

impl Joueur {

    pub fn adversaire(&self) -> Joueur {
        match self {
            Joueur::J1 => Joueur::J2,
            Joueur::J2 => Joueur::J1,
        }
    }
}

pub struct Partie {
    pub grille_j1: Grille,
    pub grille_j2: Grille,
    pub tour_actuel: Joueur,
    pub nom_j1: String,
    pub nom_j2: String,
}

impl Partie {
    pub fn new(nom_j1: &str, nom_j2: &str) -> Self {
        Partie {
            grille_j1: Grille::new(),
            grille_j2: Grille::new(),
            tour_actuel: Joueur::J1,
            nom_j1: nom_j1.to_string(),
            nom_j2: nom_j2.to_string(),
        }
    }

    pub fn nom_joueur_actuel(&self) -> &str {
        match self.tour_actuel {
            Joueur::J1 => &self.nom_j1,
            Joueur::J2 => &self.nom_j2,
        }
    }

    // Renvoie la grille sur laquelle on tire
    pub fn grille_cible(&mut self) -> &mut Grille {
        match self.tour_actuel {
            Joueur::J1 => &mut self.grille_j2, // Si c'est au J1 de jouer, il tire sur la grille du J2
            Joueur::J2 => &mut self.grille_j1,
        }
    }

    pub fn changer_tour(&mut self) {
        self.tour_actuel = self.tour_actuel.adversaire();
    }
}

pub fn analyser_saisie(entree: &str) -> Option<Coordonnee> {
    // On enleve les espaces et les retours a la ligne, et on met tout en majuscules
    let entree_propre = entree.trim().to_uppercase(); 

    // Invalide si trop court
    if entree_propre.len() < 2 {
        return None;
    }

    // On extrait la premiere lettre
    let lettre = entree_propre.chars().next()?; // Le '?' retourne None direct si ça echoue
    
    // On verifie que c'est bien une lettre entre A et J
    if lettre < 'A' || lettre > 'J' {
        return None;
    }
    
    // On transforme 'A' en 0, 'B' en 1, etc.
    let x = (lettre as u8 - b'A') as usize;

    // On prend le reste de la chaine (de l'index 1 jusqu'à la fin) pour le chiffre
    let reste = &entree_propre[1..];
    
    // On essaie de convertir ce texte en nombre entier
    let ligne: usize = reste.parse().ok()?;

    // On vérifie que le chiffre est entre 1 et 10
    if ligne < 1 || ligne > 10 {
        return None;
    }

    // On fait -1 car la ligne 1 correspond à l'index 0
    Some(Coordonnee { x, y: ligne - 1 })
}