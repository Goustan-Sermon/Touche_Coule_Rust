# main.py

import sys
import time

from reseau import connexion_serveur
from flotte import envoyer_flotte
from ia import CerveauIA
from config import C

if __name__ == "__main__":
    print(f"{C.CYAN}=== DÉMARRAGE DU BOT AMIRAL (PYTHON) ==={C.RESET}")

    if len(sys.argv) < 2:
        print(f"{C.ROUGE}[ERREUR]{C.RESET} Aucun PIN secret fourni.")
        print(f"Usage : python3 main.py <PIN_SECRET>")
        sys.exit(1)

    pin_secret = sys.argv[1]
    print(f"{C.JAUNE}[BOT]{C.RESET} Configuration auto reçue. PIN : {pin_secret}")

    socket_connecte = connexion_serveur(pin_secret)
    
    if socket_connecte:
        # 1. Identité de l'adversaire
        data_identite = socket_connecte.recv(1024)
        print(f"{C.VERT}[SERVEUR]{C.RESET} {data_identite.decode('utf-8', errors='replace').strip()}")

        # 2. Envoi de notre flotte
        envoyer_flotte(socket_connecte)

        # --- INITIALISATION DE L'IA ---
        cerveau = CerveauIA()
        mon_tour = False

        print(f"\n{C.CYAN}[BOT]{C.RESET} En attente de la confirmation de l'Hôte...")

        try:
            while True:
                data = socket_connecte.recv(1024)
                if not data:
                    print(f"\n{C.ROUGE}[BOT]{C.RESET} Connexion fermée par le serveur.")
                    break
                
                messages = data.decode('utf-8', errors='replace').strip().split('\n')
                
                for msg in messages:
                    if not msg: continue

                    # --- GESTION DU PROTOCOLE DE JEU ---
                    if "FLOTTE_OK" in msg:
                        print(f"{C.VERT}[BOT]{C.RESET} L'Hôte a validé notre flotte. En attente du tirage au sort...")
                    
                    elif "TOUR:OUI" in msg:
                        print(f"\n{C.JAUNE}[BOT]{C.RESET} C'EST MON TOUR !")
                        mon_tour = True
                        tir = cerveau.calculer_prochain_tir() 
                        time.sleep(1)
                        print(f"{C.CYAN}[BOT]{C.RESET} Tir sur {tir} !")
                        socket_connecte.sendall(f"TIR:{tir}\n".encode('utf-8'))
                    
                    elif "TOUR:NON" in msg:
                        print(f"\n{C.CYAN}[BOT]{C.RESET} C'est le tour de l'adversaire, on encaisse...")
                        mon_tour = False

                    elif msg.startswith("TIR:"):
                        case_visee = msg.split(":")[1]
                        print(f"{C.JAUNE}[BOT]{C.RESET} Alerte : L'adversaire a tiré sur {case_visee} !")

                    elif msg.startswith("REP:"):
                        resultat = msg.split(':')[1]
                        print(f"{C.CYAN}[BOT]{C.RESET} Rapport de l'arbitre : {resultat}")
                        
                        if resultat == "FIN":
                            print(f"\n{C.VERT}[BOT]{C.RESET} Fin de la partie ! Quel combat !")
                            print(f"{C.CYAN}[BOT]{C.RESET} Le Bot est prêt pour une revanche !")
                            socket_connecte.sendall("REV:OUI\n".encode('utf-8'))
                            continue 

                        # --- LE PING-PONG DES TOURS ---
                        if mon_tour:
                            cerveau.enregistrer_resultat(resultat)
                            print(f"{C.CYAN}[BOT]{C.RESET} État du cerveau : Mode {cerveau.mode}")
                            print(f"{C.CYAN}[BOT]{C.RESET} Fin de notre tour, attente de l'adversaire...")
                            mon_tour = False
                        else:
                            print(f"\n{C.JAUNE}[BOT]{C.RESET} À NOUS DE JOUER !")
                            mon_tour = True
                            
                            tir = cerveau.calculer_prochain_tir()
                            time.sleep(1.0) 
                            print(f"{C.CYAN}[BOT]{C.RESET} Tir calculé sur {tir} !")
                            socket_connecte.sendall(f"TIR:{tir}\n".encode('utf-8'))

                    elif msg.startswith("REV:"):
                        choix_adversaire = msg.split(":")[1]
                        if choix_adversaire == "OUI":
                            print(f"\n{C.VERT}[BOT]{C.RESET} REVANCHE ACCEPTÉE ! On remet ça !")
                            cerveau = CerveauIA()
                            mon_tour = False
                            time.sleep(1)
                            envoyer_flotte(socket_connecte)
                        else:
                            print(f"\n{C.ROUGE}[BOT]{C.RESET} L'adversaire a refusé la revanche. Fin des transmissions.")
                            break

                    elif "CHAT:" in msg:
                         message_recu = msg.split(":", 1)[1]
                         print(f"\n{C.MAGENTA}[Adversaire] : {message_recu}{C.RESET}")
                         time.sleep(0.5)
                         reponse_chat = "CHAT:Mes probabilités de victoire s'élèvent à 99%.\n"
                         socket_connecte.sendall(reponse_chat.encode('utf-8'))
                         
        except KeyboardInterrupt:
            print(f"\n{C.JAUNE}[BOT] Extinction manuelle du Bot.{C.RESET}")
        finally:
            socket_connecte.close()