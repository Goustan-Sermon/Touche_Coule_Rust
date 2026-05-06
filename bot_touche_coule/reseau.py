# reseau.py

import socket
import ssl
import time
from config import HOST, PORT_JEU, PORTS_KNOCKING, NOM_BOT, C

def frapper_aux_ports(ip, ports):
    print(f"{C.MAGENTA}[BOT]{C.RESET} Début de la séquence de Port Knocking sur {ip}...")
    for port in ports:
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(0.2)
            sock.connect((ip, port))
            sock.close()
        except (socket.timeout, ConnectionRefusedError):
            pass
        print(f"{C.MAGENTA}      ->{C.RESET} Frappe sur le port {port}...")
        time.sleep(0.5)
    print(f"{C.MAGENTA}[BOT]{C.RESET} Séquence terminée. Le port principal devrait être ouvert.")

def connexion_serveur(pin_secret):
    frapper_aux_ports(HOST, PORTS_KNOCKING)
    time.sleep(1)

    context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
    context.check_hostname = False
    context.verify_mode = ssl.CERT_NONE

    print(f"\n{C.CYAN}[BOT]{C.RESET} Tentative de connexion TLS sur le port {PORT_JEU}...")
    
    try:
        sock_brut = socket.create_connection((HOST, PORT_JEU), timeout=5)
        sock_tls = context.wrap_socket(sock_brut, server_hostname=HOST)
        print(f"{C.VERT}[BOT]{C.RESET} Tunnel TLS établi avec succès !")

        message_hello = f"HELLO:{NOM_BOT}:{pin_secret}\n"
        print(f"{C.CYAN}[BOT]{C.RESET} Envoi de la trame d'authentification : {message_hello.strip()}")
        sock_tls.sendall(message_hello.encode('utf-8'))

        reponse_auth = sock_tls.recv(1024).decode('utf-8', errors='replace')
        print(f"{C.VERT}[SERVEUR]{C.RESET} {reponse_auth.strip()}")

        if "AUTH_OK" in reponse_auth:
             print(f"{C.VERT}[BOT]{C.RESET} Authentification validée par le serveur !")
             sock_tls.settimeout(None)
             return sock_tls
        else:
             print(f"{C.ROUGE}[ERREUR]{C.RESET} Authentification rejetée (Mauvais PIN ?)")
             return None

    except ConnectionRefusedError:
        print(f"\n{C.ROUGE}[ERREUR]{C.RESET} Connexion refusée. Le Port Knocking a-t-il échoué ou le serveur est-il éteint ?")
        return None
    except socket.timeout:
         print(f"\n{C.ROUGE}[ERREUR]{C.RESET} Délai d'attente dépassé (Timeout). Le serveur ne répond pas comme prévu.")
         return None
    except Exception as e:
        print(f"\n{C.ROUGE}[ERREUR]{C.RESET} Une erreur est survenue : {e}")
        return None