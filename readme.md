# ⚓ Touché-Coulé : Bataille Navale TCP en Rust

[![Ask DeepWiki](https://devin.ai/assets/askdeepwiki.png)](https://deepwiki.com/Goustan-Sermon/Touche_Coule_Rust)

Ce dépôt contient le code source d'un jeu classique de Bataille Navale implémenté en Rust. Le jeu se joue à deux en réseau (client-serveur TCP), directement dans le terminal, grâce à une interface interactive contrôlée au curseur.

## ✨ Fonctionnalités

* **Multijoueur en réseau (TCP) :** Jouez contre un adversaire sur un réseau local. Il suffit de choisir d'héberger la partie ou de la rejoindre en entrant l'adresse IP de l'hôte.
* **Interface Terminal Interactive (TUI) :** Utilisation de `crossterm` pour offrir une expérience riche directement dans le terminal. Fini la saisie manuelle des coordonnées !
* **Contrôles fluides au curseur :** Utilisez les flèches directionnelles pour positionner vos navires et cibler la grille ennemie.
* **Déploiement dynamique & Hologramme :** Placez votre flotte manuellement. Un navire "fantôme" vous permet de visualiser le placement et la rotation avant de valider. Une sécurité anti-débordement (clamping) empêche les placements invalides.
* **Génération aléatoire de la flotte :** Envie de plonger directement dans l'action ? Optez pour le placement automatique pour déployer votre flotte instantanément et sans collision.
* **Temps réel & Retours visuels :** Obtenez un retour immédiat sur vos tirs (`Plouf`, `Touché` ou `Coulé !`) avec une mise à jour dynamique de votre radar tactique.

## 🚀 Comment y jouer ?

### Option 1 : Jouer directement (Recommandé)
Allez dans l'onglet **Releases** de ce dépôt GitHub et téléchargez l'exécutable correspondant à votre système (`.exe` pour Windows, ou le binaire pour Linux). Double-cliquez pour lancer le Centre de Commandement !

### Option 2 : Compiler depuis les sources
Si vous préférez compiler le jeu vous-même, assurez-vous d'avoir [Rust et Cargo](https://www.rust-lang.org/tools/install) installés.
1. Clonez le dépôt :
   ```sh
   git clone [https://github.com/goustan-sermon/touche_coule_rust.git](https://github.com/goustan-sermon/touche_coule_rust.git)
    ```
2.  Naviguez dans le dossier du projet :
    ```sh
    cd touche_coule_rust
    ```
3.  Lancez le jeu :
    ```sh
    cargo run --release
    ```
## 🗺️ Déroulement d'une partie

1.  **Lancement & Identification :** Lancez le jeu et entrez votre nom d'Amiral.
2.  **Connexion :**
    * **Hôte :** Tapez `1` pour héberger. Le jeu attendra la connexion d'un adversaire.
    * **Client :** Tapez `2` pour rejoindre, puis saisissez l'adresse IP de l'hôte (tapez `127.0.0.1` pour jouer en local sur la même machine).
3.  **Déploiement de la flotte :**
    * Choix `1` (**Manuel**) : Utilisez les **Flèches** pour déplacer le navire, **'R'** pour le faire pivoter, et **Entrée** pour valider.
    * Choix `2` (**Aléatoire**) : L'ordinateur déploie vos 5 navires de manière stratégique.
4.  **Phase de Combat :**
    * L'Hôte tire en premier.
    * À votre tour, déplacez le curseur sur le radar avec les flèches et appuyez sur **Entrée** pour faire feu.
    * La victoire est déclarée lorsqu'un joueur coule l'intégralité de la flotte adverse.

## 🏗️ Architecture du Projet

Le code source est organisé en trois modules principaux pour garantir une séparation claire des responsabilités :

* `src/main.rs` : Point d'entrée de l'application. Gère la boucle de jeu principale, les entrées utilisateur (clavier en mode brut) et l'orchestration des différentes phases (menu, placement, combat).
* `src/modele.rs` : Définit les structures de données fondamentales et la logique métier (`Grille`, `Navire`, `Coordonnee`). Gère les règles de collision et le traitement des tirs.
* `src/reseau.rs` : Implémente la couche réseau. Définit un protocole de communication textuel (`MessageReseau`) avec son parser, et gère l'ouverture des sockets TCP (`TcpListener` et `TcpStream`).

## 🛡️ Aspect Technique & Sécurité
Ce projet démontre l'utilisation sécurisée de la mémoire en Rust, la gestion de flux TCP asynchrones, et la prévention des comportements indésirables (le parser réseau rejette les commandes malformées, et l'interface empêche physiquement l'utilisateur de sortir des limites de la grille ou de tirer deux fois au même endroit).

## 📦 Dépendances

* [`crossterm`](https://github.com/crossterm-rs/crossterm) : Pour la manipulation multiplateforme du terminal, l'activation du mode brut (Raw Mode), le contrôle du curseur et le nettoyage de l'écran.
* [`rand`](https://github.com/rust-random/rand) : Utilisé pour l'algorithme de placement procédural de la flotte.
