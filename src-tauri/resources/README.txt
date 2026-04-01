================================================================
  FJKM AMBALAVAO ISOTRY — Guide de configuration
================================================================

Ce logiciel peut fonctionner sur un seul PC ou sur deux PC
relies par cable reseau (RJ45 / switch).

----------------------------------------------------------------
  CONFIGURATION AU PREMIER LANCEMENT
----------------------------------------------------------------

Au premier demarrage, une fenetre de configuration s'affiche
et vous demande de choisir le role de ce PC :

  [ Serveur ]  ou  [ Client ]

Choisissez selon les instructions ci-dessous.


================================================================
  CAS 1 — UN SEUL PC (utilisation simple)
================================================================

Choisissez : SERVEUR

-> La base de donnees est creee sur ce PC.
-> Le logiciel fonctionne entierement en local.
-> Aucune configuration supplementaire n'est necessaire.

Cliquez "Valider et demarrer".


================================================================
  CAS 2 — DEUX PC RELIES PAR CABLE RJ45
================================================================

IMPORTANT : Le PC Serveur doit toujours etre allume
            pour que le PC Client puisse fonctionner.


--- PC SERVEUR (PC principal, celui qui stocke les donnees) ---

1. Lancez le logiciel sur ce PC.
2. Choisissez : SERVEUR
3. Notez le port affiche (par defaut : 7654)
4. Cliquez "Valider et demarrer".

Pour connaitre l'adresse IP de ce PC :
   -> Ouvrez l'invite de commandes (Win + R, tapez "cmd")
   -> Tapez : ipconfig
   -> Notez la valeur "Adresse IPv4" (ex: 192.168.1.10)

Communiquez cette adresse IP au PC Client.


--- PC CLIENT (deuxieme PC) ---

1. Reliez ce PC au PC Serveur par cable RJ45
   (directement ou via un switch/routeur).
2. Lancez le logiciel sur ce PC.
3. Choisissez : CLIENT
4. Entrez l'adresse IP du PC Serveur (ex: 192.168.1.10)
5. Entrez le port (par defaut : 7654)
6. Cliquez "Tester la connexion"
   -> Si le test affiche OK, continuez.
   -> Si le test echoue, verifiez le cable et l'adresse IP.
7. Cliquez "Valider et demarrer".


================================================================
  RECONFIGURATION
================================================================

La configuration est sauvegardee automatiquement.
Elle n'est demandee qu'une seule fois (au premier lancement).

Pour changer de configuration (ex : changer l'IP du serveur,
passer de Client a Serveur, etc.) :

  METHODE 1 — Via le bouton dans l'application (recommande)
  ----------------------------------------------------------
  1. Lancez le logiciel normalement.
  2. Dans la barre de navigation (en haut), cliquez sur
     l'icone engrenage ⚙ (a droite du bouton de theme).
  3. Une confirmation s'affiche : "Reconfigurer ? [Oui] [Non]"
  4. Cliquez "Oui".
  5. Le wizard de configuration s'affiche a nouveau.
  6. Choisissez le nouveau mode et validez.

  METHODE 2 — Manuellement (si l'app ne demarre pas)
  ---------------------------------------------------
  Supprimez le fichier config.json situe dans :

    Windows :
    C:\Users\{votre_nom}\AppData\Roaming\mg.fjkm.ambalavao.isotry\

  Ensuite relancez le logiciel pour reconfigurer.


================================================================
  PROBLEMES COURANTS
================================================================

Probleme : Le PC Client ne peut pas se connecter.
Solutions :
  - Verifiez que le cable RJ45 est bien branche.
  - Verifiez que le PC Serveur est allume et que le
    logiciel est lance dessus.
  - Verifiez que l'adresse IP est correcte (tapez
    "ipconfig" sur le PC Serveur pour la confirmer).
  - Verifiez que le pare-feu Windows du PC Serveur
    autorise le port 7654.
    -> Panneau de configuration -> Pare-feu Windows
    -> Parametres avances -> Nouvelle regle entrant
    -> Port TCP 7654 -> Autoriser la connexion

Probleme : Le logiciel ne demarre pas.
Solution :
  - Assurez-vous que Microsoft WebView2 est installe.
    (Il est normalement inclus dans l'installeur.)


================================================================
  POUR AUTORISER LE PORT 7654 DANS LE PARE-FEU (PC Serveur)
================================================================

Ouvrez une invite de commandes en tant qu'administrateur
(clic droit sur le menu Demarrer -> "Terminal (admin)")
et tapez la commande suivante :

  netsh advfirewall firewall add rule name="FJKM API"
    dir=in action=allow protocol=TCP localport=7654

Ou passez par l'interface graphique :
  Panneau de configuration
  -> Systeme et securite
  -> Pare-feu Windows Defender
  -> Parametres avances
  -> Regles de trafic entrant -> Nouvelle regle
  -> Port -> TCP -> Port local specifique : 7654
  -> Autoriser la connexion -> Donner un nom : "FJKM API"


================================================================
  CONTACT / SUPPORT
================================================================

En cas de probleme, contactez l'administrateur du systeme
ou le responsable informatique de l'eglise.

================================================================
