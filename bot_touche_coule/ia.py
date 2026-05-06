# ia.py

import random

class CerveauIA:
    def __init__(self):
        self.lettres = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J']
        self.cases_restantes = set(f"{l}{c}" for l in self.lettres for c in range(1, 11))
        
        self.cases_chasse = []
        for i, l in enumerate(self.lettres):
            for c in range(1, 11):
                if (i + c) % 2 == 0:
                    self.cases_chasse.append(f"{l}{c}")
        random.shuffle(self.cases_chasse)
        
        self.mode = "CHASSE"
        self.dernier_tir = None
        self.cibles_prioritaires = []

    def obtenir_adjacentes(self, case):
        l_idx = self.lettres.index(case[0])
        c_val = int(case[1:])
        adj = []
        if l_idx > 0: adj.append(f"{self.lettres[l_idx-1]}{c_val}")
        if l_idx < 9: adj.append(f"{self.lettres[l_idx+1]}{c_val}")
        if c_val > 1: adj.append(f"{case[0]}{c_val-1}")
        if c_val < 10: adj.append(f"{case[0]}{c_val+1}")
        return adj

    def calculer_prochain_tir(self):
        while self.cibles_prioritaires:
            tir_potentiel = self.cibles_prioritaires.pop()
            if tir_potentiel in self.cases_restantes:
                self.dernier_tir = tir_potentiel
                self.cases_restantes.remove(tir_potentiel)
                return tir_potentiel
        
        self.mode = "CHASSE"

        while self.cases_chasse:
            tir_potentiel = self.cases_chasse.pop()
            if tir_potentiel in self.cases_restantes:
                self.dernier_tir = tir_potentiel
                self.cases_restantes.remove(tir_potentiel)
                return tir_potentiel

        tir_potentiel = random.choice(list(self.cases_restantes))
        self.dernier_tir = tir_potentiel
        self.cases_restantes.remove(tir_potentiel)
        return tir_potentiel

    def enregistrer_resultat(self, resultat):
        if resultat == "TOUCHE":
            self.mode = "CIBLE"
            nouvelles_cibles = self.obtenir_adjacentes(self.dernier_tir)
            self.cibles_prioritaires.extend(nouvelles_cibles)
            
        elif resultat.startswith("COULE"):
            self.mode = "CHASSE"
            self.cibles_prioritaires = []