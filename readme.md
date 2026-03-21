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
* **Tableau de Bord Stratégique (TUI Colorée) :** Une refonte visuelle complète utilisant des balises ANSI. Observez l'état de votre flotte et votre radar tactique de frappe **côte à côte** pour une immersion totale et une lisibilité instantanée des actions.
* **Canal Radio (Chat In-Game) :** Envoyez des messages en temps réel à votre adversaire pendant votre tour pour un peu de *trash-talk* tactique !
* **Rejouabilité (Revanche) :** Enchaînez les parties avec le même adversaire sans avoir à recréer le salon ou relancer la connexion sécurisée.

## 🚀 Comment y jouer ?

### Option 1 : Jouer directement (Recommandé)
Allez dans l'onglet **Releases** de ce dépôt GitHub et téléchargez l'exécutable correspondant à votre système (`.exe` pour Windows, ou le binaire pour Linux). Double-cliquez pour lancer le Centre de Commandement !

### Option 2 : Compiler depuis les sources
Si vous préférez compiler le jeu vous-même, assurez-vous d'avoir [Rust et Cargo](https://www.rust-lang.org/tools/install) installés.
1. Clonez le dépôt :
   ```sh
   git clone https://github.com/Goustan-Sermon/Touche_Coule_Rust.git
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
    * L'arbitre (le serveur) effectue un tirage au sort sécurisé pour déterminer aléatoirement quel Amiral a l'honneur de tirer en premier.
    * À votre tour, utilisez les **Flèches** pour cibler, **Entrée** pour faire feu, **'C'** pour envoyer un message radio, ou **'Q'** pour quitter proprement la partie.
    * La victoire est déclarée lorsqu'un joueur coule l'intégralité de la flotte adverse. En cas de déconnexion inattendue d'un joueur, la partie se termine proprement et signale la désertion.
    * À la fin des hostilités, proposez ou acceptez une revanche pour relancer la partie instantanément.

## 🏗️ Architecture du Projet

Le code source est organisé en trois modules principaux pour garantir une séparation claire des responsabilités :

* `src/main.rs` : Point d'entrée de l'application. Gère la boucle de jeu principale, l'affichage interactif du double tableau de bord et les entrées clavier. Surtout, en mode Hôte, il agit comme **Serveur Autoritaire**, retenant les grilles en mémoire et arbitrant les tirs pour empêcher toute triche.
* `src/modele.rs` : Définit les structures de données fondamentales et la logique métier (`Grille`, `Navire`, `Coordonnee`). Gère les règles de collision et le traitement des tirs.
* `src/reseau.rs` : Implémente la couche réseau. Définit un protocole de communication textuel (`MessageReseau`) avec son parser, et gère l'ouverture des sockets TCP (`TcpListener` et `TcpStream`).

## 🛡️ Aspect Technique & Cybersécurité

Ce projet a été conçu avec une approche "Security by Design", en traitant les vulnérabilités réseau courantes :

* **Chiffrement de bout en bout (TLS 1.3) :** Remplacement des flux TCP en clair par des tunnels sécurisés à l'aide de `rustls`. Impossibilité de tricher via "Sniffing" (Wireshark) ou de réaliser des attaques de type Man-in-the-Middle (MitM).
* **Génération dynamique de certificats :** Utilisation de `rcgen` pour forger à la volée des certificats auto-signés lors de la création du serveur, sans nécessiter de configuration externe complexe.
* **Prévention du Déni de Service (DoS) :** Implémentation d'un mécanisme de *clamping* réseau (limite stricte à 512 octets par lecture) pour bloquer les attaques par épuisement de ressources (Buffer Overflow) visant à saturer la RAM.
* **Validation stricte des paquets :** Le *parser* réseau rejette systématiquement les commandes malformées, garantissant la stabilité du serveur face à des injections de données corrompues. Traitement robuste de l'encodage (UTF-8 lossy) pour éviter les crashs liés aux saisies invalides.
* **Port Knocking Applicatif (Anti-Reconnaissance) :** Le port principal du jeu (3333) est masqué par défaut aux scanners réseau (comme `nmap`). Un "Gardien" multi-threadé écoute silencieusement une séquence de frappe spécifique sur des ports leurres (7777, 8888, 9999). Le véritable tunnel chiffré ne s'ouvre qu'après validation de cette séquence dynamique, empêchant toute tentative de connexion non sollicitée.
* **Contrôle d'Accès & Fail2Ban (Anti-Bruteforce) :** Chaque salon est protégé par une authentification applicative via un code PIN à 4 chiffres généré aléatoirement. Le serveur intègre un registre dynamique (`HashMap`) qui agit comme un bouclier Fail2Ban : il bannit et rejette automatiquement les adresses IP après 3 tentatives d'authentification infructueuses, tout en maintenant le serveur en ligne et résilient.
* **Serveur Autoritaire (Anti-Cheat) :** Abandon du modèle de confiance client ("Client-Trust"). En début de partie, les clients transfèrent de manière sécurisée la position de leur flotte à l'Hôte. C'est le serveur qui calcule les impacts et renvoie le verdict, rendant la falsification des dégâts (tricherie de type God Mode) mathématiquement impossible pour le client.
* **Validation Stricte des Entrées (Input Validation) :** Les saisies sensibles, telles que l'adresse IP du serveur, sont analysées et validées formellement avant toute interaction réseau. Cela empêche les envois de paquets aveugles ou les requêtes DNS inutiles liées à des saisies corrompues. Purge systématique des buffers clavier (Input Buffering) pour contrer les comportements asynchrones indésirables du système d'exploitation.
* **Résilience aux Déconnexions (Fail-Safe) :** Gestion gracieuse des fermetures inattendues de sockets TCP (`Broken Pipe` / Rage-quit). Le serveur intercepte les pertes de connexion sans déclencher de panique mémoire, libère les ports proprement et informe l'adversaire de la désertion.
## 📦 Dépendances

* [`crossterm`](https://github.com/crossterm-rs/crossterm) : Pour la manipulation multiplateforme du terminal, l'activation du mode brut (Raw Mode), le contrôle du curseur et le nettoyage de l'écran afin de créer l'interface interactive (TUI).
* [`rand`](https://github.com/rust-random/rand) : Utilisé pour la génération procédurale du placement de la flotte, la création aléatoire du code PIN sécurisé (4 chiffres) du salon, et le tirage au sort (pile ou face) de l'arbitre pour déterminer qui joue le premier tour.
* [`rustls`](https://github.com/rustls/rustls) & [`rcgen`](https://github.com/rustls/rcgen) : Pour la génération de certificats à la volée et la mise en place du tunnel chiffré TCP (TLS 1.3).