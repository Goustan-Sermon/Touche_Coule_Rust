# ⚓ Touché-Coulé : Bataille Navale TCP en Rust

[![Ask DeepWiki](https://devin.ai/assets/askdeepwiki.png)](https://deepwiki.com/Goustan-Sermon/Touche_Coule_Rust)

Ce dépôt contient le code source d'un jeu classique de Bataille Navale implémenté en Rust. Le jeu se joue à deux en réseau (client-serveur TCP), directement dans le terminal, grâce à une interface interactive contrôlée au curseur. Il inclut également une **Intelligence Artificielle en Python** intégrée pour jouer en solo.

## ✨ Fonctionnalités

* **Multijoueur en réseau (TCP) :** Jouez contre un adversaire sur un réseau local. Il suffit de choisir d'héberger la partie ou de la rejoindre en entrant l'adresse IP de l'hôte.
* **Mode Solo contre l'IA (Hunt & Target) :** Affrontez le "Bot Amiral", un client Python autonome exécuté en arrière-plan, doté d'un algorithme de probabilité pour traquer et couler votre flotte sans pitié.
* **Interface Terminal Interactive (TUI) :** Utilisation de `crossterm` pour offrir une expérience riche directement dans le terminal. Fini la saisie manuelle des coordonnées !
* **Contrôles fluides au curseur :** Utilisez les flèches directionnelles pour positionner vos navires et cibler la grille ennemie.
* **Déploiement dynamique & Hologramme :** Placez votre flotte manuellement. Un navire "fantôme" vous permet de visualiser le placement et la rotation avant de valider. Une sécurité anti-débordement (clamping) empêche les placements invalides.
* **Génération aléatoire de la flotte :** Envie de plonger directement dans l'action ? Optez pour le placement automatique (côté Rust ou Python) pour déployer la flotte instantanément et sans collision.
* **Tableau de Bord Stratégique (TUI Colorée) :** Une refonte visuelle complète utilisant des balises ANSI. Observez l'état de votre flotte et votre radar tactique de frappe **côte à côte** pour une immersion totale et une lisibilité instantanée des actions.
* **Canal Radio (Chat In-Game) :** Envoyez des messages en temps réel à votre adversaire pendant votre tour pour un peu de *trash-talk* tactique !
* **Rejouabilité (Revanche) :** Enchaînez les parties avec le même adversaire (ou l'IA) sans avoir à recréer le salon ou relancer la connexion sécurisée.

## 🚀 Comment y jouer ?

Assurez-vous d'avoir [Rust et Cargo](https://www.rust-lang.org/tools/install) ainsi que **Python 3** installés sur votre machine.

1. Clonez le dépôt :
   ```sh
   git clone [https://github.com/Goustan-Sermon/Touche_Coule_Rust.git](https://github.com/Goustan-Sermon/Touche_Coule_Rust.git)
   ```
2. Naviguez dans le dossier du projet :
   ```sh
   cd touche_coule_rust
   ```
3. Lancez le jeu :
   ```sh
   cargo run --release
   ```

## 🗺️ Déroulement d'une partie

1.  **Lancement & Identification :** Lancez le jeu et entrez votre nom d'Amiral.
2.  **Connexion :**
    * **Hôte :** Tapez `1` pour héberger. Le jeu attendra la connexion d'un adversaire.
    * **Client :** Tapez `2` pour rejoindre, puis saisissez l'adresse IP de l'hôte (tapez `127.0.0.1` pour jouer en local sur la même machine).
    * **Solo :** Tapez `3` pour lancer l'IA Python en arrière-plan de manière totalement transparente.
3.  **Déploiement de la flotte :**
    * Choix `1` (**Manuel**) : Utilisez les **Flèches** pour déplacer le navire, **'R'** pour le faire pivoter, et **Entrée** pour valider.
    * Choix `2` (**Aléatoire**) : L'ordinateur déploie vos 5 navires de manière stratégique.
4.  **Phase de Combat :**
    * L'arbitre (le serveur) effectue un tirage au sort sécurisé pour déterminer aléatoirement quel Amiral a l'honneur de tirer en premier.
    * À votre tour, utilisez les **Flèches** pour cibler, **Entrée** pour faire feu, **'C'** pour envoyer un message radio, ou **'Q'** pour quitter proprement.
    * La victoire est déclarée lorsqu'un joueur coule l'intégralité de la flotte adverse. En cas de déconnexion inattendue ou de triche avérée, le serveur expulse le joueur.

## 🏗️ Architecture du Projet

### Le Cœur du Jeu (Rust)
Le code source Rust est organisé selon le modèle **MVC (Modèle-Vue-Contrôleur)** :
* `src/main.rs` *(Contrôleur)* : Point d'entrée. Orchestre le menu, la connexion réseau et la boucle de jeu. En mode Hôte, il agit comme **Serveur Autoritaire**, retenant les grilles en mémoire et arbitrant les tirs.
* `src/modele.rs` *(Modèle)* : Définit la logique métier pure (`Grille`, `Navire`). Gère les règles de collision et le traitement des dégâts de manière isolée.
* `src/affichage.rs` *(Vue)* : Gère le dessin des interfaces dans le terminal via `crossterm` et la capture des saisies utilisateur.
* `src/reseau.rs` *(Infrastructure)* : Implémente la couche réseau (Port Knocking, TCP, tunnel TLS) via le trait abstrait `FluxJeu`.

### L'Intelligence Artificielle (Python)
Le sous-dossier `bot_touche_coule/` contient l'IA, découpée de manière modulaire :
* `ia.py` : Cerveau du Bot basé sur l'algorithme **Hunt & Target** (damier mathématique pour la recherche, ciblage en croix une fois touché).
* `flotte.py` : Génération mathématique anti-collision des navires (Horizontal/Vertical).
* `reseau.py` & `main.py` : S'occupent de contourner les défenses du serveur Rust (Knocking, Handshake TLS) pour s'y infiltrer légitimement.

## 📡 Protocole de Communication (TCP / TLS)

Le serveur et les clients communiquent via un protocole textuel strict conçu pour ce projet, avec des commandes séparées par `:`.

| Phase | Commande Client (Bot/Joueur) | Commande Serveur (Hôte) | Description |
| :--- | :--- | :--- | :--- |
| **Authentification** | `HELLO:<Nom>:<PIN>\n` | `REP:AUTH_OK\n` ou `REP:AUTH_FAIL\n` | Vérification du code d'accès au salon. |
| **Authentification** | - | `HELLO:<NomHôte>:\n` | Après succès, l'hôte transmet son nom. |
| **Déploiement** | `NAV:<Nom>:<Taille>:<X>:<Y>:<H/V>\n` | - | Envoi sécurisé des coordonnées des navires. |
| **Déploiement** | `FLOTTE_OK:OK\n` | `FLOTTE_OK:OK\n` | Signal de fin de déploiement mutuel. |
| **Synchronisation** | - | `TOUR:OUI\n` ou `TOUR:NON\n` | L'arbitre informe de l'ordre de jeu. |
| **Combat (Tir)** | `TIR:<Case>\n` (ex: `TIR:A1\n`) | `TIR:<Case>\n` | Le joueur actif annonce sa cible. |
| **Combat (Résultat)**| - | `REP:ALEAU\n` <br> `REP:TOUCHE\n` <br> `REP:COULE:<Nom>\n` <br> `REP:FIN\n` | L'arbitre centralise et diffuse l'impact. |
| **Revanche** | `REV:OUI\n` ou `REV:NON\n` | `REV:OUI\n` ou `REV:NON\n` | Négociation de relance de partie. |
| **Social** | `CHAT:<Message>\n` | `CHAT:<Message>\n` | Canal de communication radio (texte). |

## 🛡️ Aspect Technique & Cybersécurité

Ce projet a été conçu avec une approche "Security by Design" :

* **Chiffrement de bout en bout (TLS 1.3) :** Utilisation de `rustls` pour empêcher la triche par écoute du réseau (Sniffing/Man-in-the-Middle).
* **Génération dynamique de certificats :** `rcgen` forge des certificats auto-signés à la volée.
* **Prévention du Déni de Service (DoS) :** Implémentation d'un mécanisme de *clamping* réseau (limite stricte à 512 octets par lecture) contre les Buffer Overflows.
* **Validation stricte et Anti-Triche :** Le *parser* réseau rejette les commandes malformées. Les clients doivent soumettre leurs grilles, et **seul le serveur valide les tirs**, rendant le *God Mode* impossible.
* **Port Knocking Applicatif (Anti-Reconnaissance) :** Le port principal du jeu (3333) est masqué. Un "Gardien" multi-threadé exige une séquence précise (7777, 8888, 9999) pour autoriser la connexion.
* **Fail2Ban (Anti-Bruteforce) :** Une `HashMap` agit comme un bouclier et bannit temporairement les adresses IP après 3 échecs d'authentification.
* **Résilience aux Déconnexions (Fail-Safe) :** Gestion gracieuse des fermetures inattendues de sockets (`Broken Pipe`) sans provoquer de `panic!` du serveur.

## 📦 Dépendances Principales (Rust)

* `crossterm` : Manipulation TUI, gestion du mode brut et clavier.
* `rand` : Génération procédurale, codes sécurisés et tirages au sort.
* `rustls` & `rcgen` : Implémentation du tunnel chiffré TCP (TLS 1.3).