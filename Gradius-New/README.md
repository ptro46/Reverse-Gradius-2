# ASTRA FRACTURE

**Un nouveau shoot’em up spatial, natif Rust, version 0.1.2.**

Projet indépendant créé dans `/Users/ptro/Projets/Gradius-2/Gradius-New/`.
L'esprit du défilement horizontal, de l'esquive et des armes évolutives vient
de Gradius 2 ; ce jeu possède sa propre identité, ses niveaux, ses ennemis,
ses boss, ses graphismes, sa musique et sa simulation. Il ne charge aucune
ROM et ne dépend pas des ressources du projet Gradius-2 voisin.

## Jouer

Ouvrir `dist/Astra Fracture.app`. Le paquet livré est compilé pour
**macOS Apple Silicon / ARM64**. Les sources séparent la simulation du rendu
et sont portables ; Mac Intel, Windows et Linux ne sont pas vérifiés.

Au titre : Entrée ou clic sur « Engager l'expédition ». Les flèches gauche
et droite permettent de commencer dans un secteur avancé.

| Commande | Action |
| --- | --- |
| Flèches, ZQSD ou WASD | Déplacement avec accélération douce |
| Espace ou clic gauche | Tir maintenu |
| Majuscule | Vol de précision et visualisation de la petite hitbox |
| C ou clic droit | Dash invulnérable, coûte 22 points d'énergie |
| V | Surcharge pendant 6 secondes si l'énergie atteint 100 |
| X ou Entrée en vol | Activer l'amélioration sélectionnée |
| 1 / 2 / 3 | Impulsion / Éventail / Lance phase, si débloqués |
| P | Pause |
| H | Afficher les commandes ; suspend le vol |
| M | Couper ou réactiver le son |
| F | Plein écran |
| F3 | Activer ou désactiver les halos et réduire les particules |
| F4 | Qualité Standard / Haute / Ultra / 4K |
| F2 | Activer ou désactiver la Logitech Dual Action |
| F1 | Entraînement invulnérable, signalé à l'écran |
| Échap | Fermer l'aide, revenir au menu, puis quitter |

## Logitech Dual Action en option

La manette USB Logitech Dual Action (`046d:c216`) est détectée automatiquement,
même si elle est branchée après le lancement. **F2** active ou désactive sa
prise en compte ; le statut apparaît au titre et dans l’aide (**H**).
Le clavier et la souris restent disponibles. `--no-gamepad` démarre sans
initialiser la manette ; F2 peut l’activer ensuite.

| Commande manette | Action |
| --- | --- |
| Stick gauche, repéré B11 sur le schéma | Déplacement analogique ; pas besoin de cliquer |
| B8 maintenu | Tir |
| B6 | Surcharge de 6 secondes si énergie à 100 |
| B1 | Impulsion |
| B4 | Éventail, si débloqué |
| B3 | Lance phase, si débloquée |
| B2 | Activer/désactiver les halos et réduire les particules |
| B10 Start | Lancer/recommencer ; pause/reprise pendant la partie |

B11 est techniquement le clic du stick gauche : ses axes X/Y assurent le
mouvement. Une zone morte radiale évite la dérive ; les diagonales conservent
la même vitesse maximale. Au menu, le stick choisit le secteur. Les boutons
non listés restent sans affectation ; les améliorations, le dash et le vol de
précision restent accessibles au clavier. Une déconnexion en partie suspend
le jeu et efface les entrées de la manette ; P ou B10 reprend la partie.
Le branchement ou la réactivation avec B10/B2/B6 déjà maintenu ne produit
pas d’action accidentelle.

Pour un diagnostic sans fenêtre :

```bash
target/release/astra-fracture --gamepad-test 10
```

Le vaisseau possède trois points de coque et trois vies. Les boucliers
absorbent les dégâts avant la coque. Une protection temporaire, indiquée
par une teinte cyan stable, suit un impact ou une réapparition.

## Une expédition en trois actes

| Secteur | Environnement | Boss |
| --- | --- | --- |
| Lune brisée | Fragments lunaires et débris orbitaux | Le Veilleur |
| Couronne de cendres | Forge solaire et portes à plasma | Solstice |
| La Nef de verre | Architecture cristalline et couloirs resserrés | Le Chœur |

Les secteurs durent respectivement 84, 90 et 96 secondes avant l'apparition
du boss. Chaque boss possède trois phases, ses motifs de tir et une période
de protection de son réacteur. La défaite d'un boss ouvre le secteur suivant,
restaure la coque et attribue une prime. Une mort reprend le secteur au
dernier jalon de vingt secondes ; un combat de boss conserve sa progression.

Les cristaux cyan font avancer la sélection de six améliorations :
propulsion, éventail, missiles à tête chercheuse, lance phase, deux drones
compagnons et bouclier. Le joueur choisit quand dépenser cette sélection.
Les cristaux verts restaurent un point de coque.

L'énergie se gagne avec le temps, les destructions, les cristaux et les
esquives très proches. Elle sert au dash ou à une surcharge offensive.
Enchaîner les destructions entretient le multiplicateur de score. Les
projectiles adverses ont un cœur clair et des couleurs chaudes ou violettes ;
les tirs du joueur sont principalement cyan.

## Présentation et son créés pour ce jeu

- Trois décors peints, un intercepteur, cinq familles ennemies et trois
  boss : images originales générées pour ce projet avec **imagegen**,
  conservées dans `assets/` avec leur transparence.
- Obstacles, réacteurs, drones, étoiles, propulsion, explosions, particules,
  interface et post-traitement : code graphique écrit pour ce jeu.
- Cinq thèmes musicaux originaux, stéréo 44,1 kHz : *Veille orbitale*,
  *Lune brisée*, *Les Forges héliques*, *La Nef de verre* et *Fracture terminale*.
- Treize bruitages synthétisés pour les armes, les impacts, les explosions,
  les cristaux, le dash et les transitions.

Les notes, accords, motifs et timbres sont composés dans `src/audio.rs`.
Les fichiers WAV sont générés hors ligne par le synthétiseur Rust : aucune
mélodie, donnée sonore ou banque d'échantillons de Gradius n'est réutilisée.
Les polices Rajdhani et Share Tech Mono sont des ressources tierces libres ;
leurs licences OFL sont incluses à côté des fichiers TTF.

La simulation est cadencée à 120 Hz. Le rendu GPU utilise un cadre 16:9,
un post-traitement lumineux, des images haute résolution et des géométries
procédurales. La qualité Haute utilise un rendu interne 1 920 × 1 080,
Ultra 2 560 × 1 440, et 4K 3 840 × 2 160. F3/F4 permettent d'ajuster le coût
graphique. Aucune fréquence d'images universelle n'est garantie : elle dépend
de la machine, de la résolution et du nombre d'effets à l'écran.

## Sources et outils

```bash
cd /Users/ptro/Projets/Gradius-2/Gradius-New
cargo test --offline
cargo clippy --offline -- -D warnings
cargo run --release --offline
bash tools/package_macos.sh
```

Sur une machine sans dépendances en cache, retirer `--offline` pour la
première compilation. Le jeu compilé est autonome : toutes ses ressources
sont embarquées dans l'exécutable.

```bash
# Recomposer les cinq musiques et les treize bruitages
cargo run --release --offline -- --compose assets/audio
# Reconstruire après la recomposition pour embarquer les WAV à jour
cargo build --release --offline
# Campagne automatisée avec invulnérabilité et armes améliorées
target/release/astra-fracture --self-test
# Captures de vérification, sans modifier le record personnel
ASTRA_CAPTURE_DIR=verification target/release/astra-fracture --smoke-test
# Même vérification avec rendu interne 4K
ASTRA_CAPTURE_DIR=verification-4k target/release/astra-fracture --smoke-test --4k
```

- `src/core.rs` : règles, progression, collisions balayées, ennemis et boss.
- `src/render.rs` : images, géométries, interface, éclairage et shaders.
- `src/audio.rs` : partition, synthèse, bruitages et lecture avec transitions.
- `src/controller.rs` : détection Logitech, axes physiques, boutons et branchement à chaud.
- `src/main.rs` : fenêtre, entrées, boucle fixe, record et modes de vérification.
- `assets/manifest.json` : provenance et empreintes des ressources.
- `VALIDATION.md` : vérifications effectivement réalisées et limites.

Le record normal est conservé dans
`~/Library/Application Support/Astra Fracture/record.txt` sur macOS.
Le mode de captures ne lit et n'écrit pas ce fichier.

## État de cette première version

La campagne et son écran de victoire sont implémentés. Les tests automatisés
ne remplacent pas des essais de difficulté avec plusieurs joueurs :
l'équilibrage reste à affiner. Cette version utilise clavier, souris et Logitech Dual Action en option,
sans sauvegarde de campagne en cours.
Les collisions de vaisseaux sont des volumes de jeu simplifiés, volontairement
plus petits que les ailes décoratives ; le réacteur du boss est sa cible.
