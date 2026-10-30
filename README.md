# Gradius 2 / Nemesis 2 — version native Rust pour macOS

Version **0.2.0 expérimentale et jouable**, créée à partir de la ROM fournie.
Le rendu et la simulation tournent en Rust. Le programme ne lance ni émulateur
MSX, ni BIOS, ni interpréteur Z80. Toutes les ressources nécessaires sont
intégrées dans l'exécutable ; la ROM n'est pas requise pour jouer.

L'application se trouve dans `dist/Gradius 2.app`. La compilation livrée est
destinée à **macOS Apple Silicon (ARM64)**, avec un rendu OpenGL. Les autres
plateformes et les Mac Intel ne sont pas vérifiés.

## Mode moderne — 0.2.0

Le jeu démarre avec le mode moderne actif :

- Agrandissement **Scale4x** en deux passes GPU Scale2x, de 256 × 224 à
  1 024 × 896, puis adaptation à la fenêtre en conservant les proportions.
- Contours adoucis selon les motifs des pixels, au lieu d'un simple flou.
- Halos, lumière des tirs, traînée du moteur et particules d'explosion.
- Fond spatial en parallaxe et nébulosité discrète.
- Interface et compteurs redessinés à la résolution de la fenêtre.

**F2** bascule immédiatement entre Moderne et Rétro. **F3** active ou coupe
les effets lumineux et le fond enrichi, tout en conservant l'upscale et
l'interface moderne. `--retro` lance directement le mode rétro ;
`--no-effects` lance le mode moderne sans effets.

L'agrandissement et les effets sont calculés à l'affichage. Les ressources
extraites, la simulation, les collisions et l'audio sont inchangés. Il ne
s'agit pas de nouveaux détails de dessin inventés par une IA : le pixel art
original reste la base. Les règles de Scale2x/Scale4x sont documentées sur
[le site officiel de Scale2x](https://www.scale2x.it/algorithm).

## Correction 0.1.1

Les douze atlas d'ennemis et de capsules ont désormais un fond transparent.
Le vaisseau reste visible pendant l'invulnérabilité, avec une teinte bleutée
fixe qui remplace les images où il disparaissait.

## Commandes

| Touche | Action |
| --- | --- |
| Entrée | Démarrer, revenir au menu après une partie |
| Flèches | Déplacement ; au titre, choisir un secteur |
| Espace ou Z | Tir maintenu |
| Majuscule ou X | Activer la case du compteur de bonus |
| P | Pause |
| M | Couper ou réactiver l'audio |
| F | Plein écran |
| F2 | Basculer Moderne / Rétro |
| F3 | Activer / couper les effets modernes |
| F1 | Entraînement invulnérable, signalé à l'écran |
| Échap | Revenir au titre ; depuis le titre, quitter |

Le secteur 1 lance la campagne : 1→2→3→4→5→6→7→6→5→4→3→2→1→0.
Les autres secteurs sont accessibles en entraînement individuel. Le secteur 0
est la forteresse ; le 8, l'intérieur des boss ; 9–11, les cartes secrètes.
La jauge de bonus propose vitesse, missile, double tir, laser, deux options
et bouclier. Après avoir vaincu un boss, on peut entrer dans son ouverture
pendant cinq secondes pour rejoindre son intérieur.

## Ce qui provient réellement de votre ROM

- Les douze cartes, leurs métablocs, leurs caractères et leurs couleurs.
- Les motifs de sprites, le vaisseau, la fonte, le titre et les cinq boss.
- Les formes initiales des objets du décor et leurs positions.
- Les tables de collision par caractère, les positions des ennemis, les
  générateurs, les tourelles, les checkpoints et les limites de niveaux.
- **67 flux audio** extraits en exécutant hors ligne le moteur sonore original,
  avec ses écritures PSG et SCC. Les flux sont dans `assets/audio`, les WAV
  d'écoute dans `extracted/wav`. Certains identifiants sont muets ou partagent
  des sons ; il ne s'agit donc pas de 67 morceaux distincts.

Source : `Gradius 2. Nemesis 2 (1987)(Konami)[RC-751].rom`, 131 072 octets.
SHA-256 : `1a0f09df467036758c3cc8c9382e8fb79319278575e7a86b9461ed835f7d7608`.
La ROM source reste intacte dans le dossier du projet.

Les décodeurs de référence et l'interpréteur utilisé **uniquement pour
l'extraction hors ligne** sont issus du projet
[antxiko/Nemesis2-disassembly](https://github.com/antxiko/Nemesis2-disassembly).
Leur licence MIT est conservée dans `tools/reference/LICENSE`. Les graphismes,
sons et données du jeu conservent leur provenance Konami. L'archive de référence
et les écarts de variante sont identifiés dans `extracted/reference-audit.json`.

## Fidélité : limites explicites

Cette version est une réécriture jouable, **pas encore une reproduction complète
et fidèle du gameplay original**. Les déplacements des familles d'ennemis,
les cadences, les trajectoires et les attaques des boss sont reconstruits et
simplifiés. Les objets du décor ont leur apparence initiale, sans toutes leurs
animations et interactions. La phase particulière de fin des secteurs 4–6,
les sept armes capturables, les capsules spéciales et les accès originaux aux
zones secrètes ne sont pas entièrement portés. L'intérieur donne actuellement
une récompense simplifiée. Le noyau final et les explosions utilisent aussi
des éléments dessinés par le nouveau moteur. Le jeu à deux joueurs, la
manette et la sauvegarde persistante du score ne sont pas implémentés.

Le son est une **synthèse reconstruite** à partir des commandes originales,
pas un enregistrement de la puce. La restitution analogique et le minutage
à l'intérieur d'une image ne sont pas reproduits. Les pistes dont aucune
boucle n'a été détectée sont plafonnées à 75 secondes ; leurs durées sont
indiquées dans `assets/audio/tracks.json`.

## Sources et reconstruction

Le dossier demandé est `/Users/ptro/Projets/Gradius-2/`.

```bash
cd /Users/ptro/Projets/Gradius-2
cargo test --offline
cargo clippy --offline -- -D warnings
cargo run --release --offline
bash tools/package_macos.sh
```

Les dépendances sont verrouillées dans `Cargo.lock`. Sur une nouvelle machine,
une première compilation peut nécessiter de retirer `--offline` pour télécharger
les dépendances. La compilation Rust ne dépend pas de Python.

Pour refaire l'extraction : Python 3 avec Pillow ; l'exécution du moteur sonore
est incluse dans les outils de référence et n'a pas de dépendance NumPy.

```bash
python3 tools/extract.py 'Gradius 2. Nemesis 2 (1987)(Konami)[RC-751].rom'
cargo run --release --offline -- --export-audio extracted/wav
cargo run --release --offline -- --self-test
```

`src/core.rs` contient la simulation indépendante du rendu et du système.
`src/main.rs` contient l'affichage et les commandes. `src/audio.rs` synthétise
les commandes audio. `assets/manifest.json` consigne les empreintes extraites.

Les vérifications effectuées et leurs limites figurent dans `VALIDATION.md`.
