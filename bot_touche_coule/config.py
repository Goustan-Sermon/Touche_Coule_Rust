# config.py

HOST = '127.0.0.1'
PORT_JEU = 3333
PORTS_KNOCKING = [7777, 8888, 9999]
NOM_BOT = "Amiral_Python"

# --- PALETTE DE COULEURS ANSI ---
class C:
    CYAN = '\x1b[1;36m'     # Standard [BOT]
    VERT = '\x1b[1;32m'     # Succes / [SERVEUR]
    ROUGE = '\x1b[1;31m'    # [ERREUR]
    JAUNE = '\x1b[1;33m'    # Alerte / Action
    MAGENTA = '\x1b[1;35m'  # [GARDIEN] / Chat
    RESET = '\x1b[0m'       # Reinitialiser la couleur