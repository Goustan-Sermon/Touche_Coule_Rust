# flotte.py

import time
import random
from config import C

def generer_flotte_aleatoire():
    navires = [
        ("Porte-avions", 5),
        ("Croiseur", 4),
        ("Contre-torpilleur", 3),
        ("Sous-marin", 3),
        ("Torpilleur", 2)
    ]
    
    cases_occupees = set()
    flotte_str = []

    for nom, taille in navires:
        place = False
        while not place:
            orientation = random.choice(["H", "V"])
            
            if orientation == "H":
                x = random.randint(0, 10 - taille)
                y = random.randint(0, 9)
                cases_test = {(x + i, y) for i in range(taille)}
            else:
                x = random.randint(0, 9)
                y = random.randint(0, 10 - taille)
                cases_test = {(x, y + i) for i in range(taille)}

            if not cases_test.intersection(cases_occupees):
                cases_occupees.update(cases_test)
                flotte_str.append(f"NAV:{nom}:{taille}:{x}:{y}:{orientation}\n")
                place = True
                
    return flotte_str

def envoyer_flotte(sock):
    print(f"\n{C.CYAN}[BOT]{C.RESET} Génération et déploiement de la flotte tactique (Mode Aléatoire)...")
    flotte = generer_flotte_aleatoire()
    
    for navire in flotte:
        sock.sendall(navire.encode('utf-8'))
        time.sleep(0.1)
        
    print(f"{C.VERT}[BOT]{C.RESET} Navires positionnés sans collision.")
    sock.sendall("FLOTTE_OK:OK\n".encode('utf-8'))
    print(f"{C.CYAN}[BOT]{C.RESET} Signal FLOTTE_OK:OK envoyé !")