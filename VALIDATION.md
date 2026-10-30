# Vérifications — 30 octobre 2026

Résultats sur ce Mac ARM64, pour la version expérimentale 0.2.0.

| Contrôle | Résultat |
| --- | --- |
| ROM source | 131 072 octets ; SHA-256 enregistré ; fichier source conservé intact |
| Données de référence | 89 830 octets de données comparés ; 9 927 différences aux mêmes offsets, documentées ; aucune équivalence globale des deux ROM revendiquée |
| Extraction | Douze cartes et VRAM ; graphismes, cinq boss et tables du jeu décodés depuis la ROM fournie |
| Intégrité des ressources | 137 empreintes SHA-256 vérifiées, dimensions des douze masques et taille des VRAM vérifiées |
| Audio | 67 flux PSG/SCC parsés et 67 WAV mono PCM 16 bits à 22 050 Hz vérifiés ; 65 non silencieux et 2 muets |
| Tests Rust | Six tests réussis : collision/dimensions, armes, tirs et respawn, structure et transitions de campagne, rendu PCM non silencieux |
| Simulation | 1 800 pas par secteur, soit 21 600 pas sur les douze secteurs, sans erreur |
| Campagne | Les transitions des quatorze étapes atteignent la victoire dans le test ; les combats y sont abrégés artificiellement |
| Analyse statique | `cargo clippy --offline -- -D warnings` réussi |
| Compilation | `cargo build --release --offline` réussi ; exécutable Mach-O ARM64 |
| Interface macOS | Titre, décors des secteurs 1 et 3 et boss affichés ; captures relues visuellement |
| Commandes en fenêtre | Entrée, déplacement à droite, tir maintenu, pause et retour au titre vérifiés par interaction UI |
| Paquet | Application autonome, signature ad hoc vérifiée ; archive ZIP sans erreur |

Les captures automatiques sont enregistrées dans `extracted/verification/`.
Leur mode de test active l'invulnérabilité et positionne le vaisseau à plusieurs
endroits : elles ne constituent pas une partie complète gagnée au clavier.

La tentative de backend Metal n'a pas été retenue : l'API de capture interne
Macroquad y échoue, et le contrôle de la fenêtre n'a pas donné un rendu exploitable.
Le paquet livré utilise **OpenGL**, dont le rendu et les commandes ont été vérifiés.
La tentative graphique dans le bac à sable a également échoué à créer un contexte ;
les vérifications GUI réussies ont eu lieu dans la session macOS normale.

Non vérifiés : fidélité image par image ou cycle par cycle face à un MSX,
équivalence auditive avec les puces physiques, campagne entière jouée manuellement,
Mac Intel, Linux et Windows. Les comportements simplifiés et fonctionnalités
absentes sont détaillés dans `README.md`.

## Régression sprites — 0.1.1

- Les douze atlas d'ennemis ont été régénérés ; leurs pixels colorés sont
  identiques à la version précédente. Les fonds noirs sont RGBA (0,0,0,0).
- Les autres ressources, y compris cartes, vaisseau et audio, ont les mêmes
  empreintes que la version précédente.
- Six captures GPU avec invulnérabilité active couvrent les six phases de
  l'ancien clignotement. Le vaisseau est visible et son image est identique
  dans les six captures.
- Sept cellules d'ennemis/capsules ont été comparées au pixel près à leur
  composition attendue sur un damier coloré, dans chacune des six captures.
- 27 648 pixels de rendu vérifiés ; six tests Rust, Clippy strict et le
  contrôle d'intégrité des ressources passent.

Les captures et le rapport sont dans `extracted/verification/sprite-fix/`.
Pour reproduire le contrôle :

```bash
GRADIUS_SMOKE_DIR=extracted/verification/sprite-fix \
  target/release/gradius2-native --sprite-check
python3 tools/verify_sprite_render.py
python3 tools/validate_assets.py
```

## Mode moderne — 0.2.0

- Le shader Scale4x a été comparé à un calcul indépendant sur CPU : égalité
  RGBA exacte des **917 504 pixels** de sortie (1 024 × 896).
- Captures comparables du titre, du secteur 1 et du boss enregistrées dans
  `extracted/verification/modern`, en Moderne et en Rétro ; images relues.
- Le contrôle de transparence et des six phases d'invulnérabilité reste
  valide : **27 648 pixels GPU** comparés à leur composition attendue.
- `src/core.rs`, `src/audio.rs` et les 138 fichiers du dossier `assets`
  sont identiques à la version 0.1.1. Les six tests Rust et Clippy strict
  passent. Aucune modification des collisions ou du rythme de simulation.
- Application ARM64 reconstruite et signée localement ; archive ZIP vérifiée.

Les halos, le décor spatial procédural, les particules et la nouvelle
interface sont des éléments de présentation écrits pour cette version.
L'upscale ne constitue pas une extraction de détails supplémentaires de la ROM.

Reproduction du contrôle GPU :

```bash
GRADIUS_SMOKE_DIR=extracted/verification/modern \
  target/release/gradius2-native --visual-check
GRADIUS_SMOKE_DIR=extracted/verification/modern \
  target/release/gradius2-native --visual-check --retro
python3 tools/verify_upscale.py
```
