# Vérification — ASTRA FRACTURE 0.1.2

Vérifications réalisées le 31 octobre 2026 sur ce Mac Apple Silicon.

## Résultats

- Compilation Rust release ARM64 et application macOS autonome avec ressources embarquées.
- `cargo test --offline` : **13 tests réussis** (collision balayée, améliorations, dash/surcharge, mort/réapparition, transitions de campagne, déterminisme et bornes, synthèse audio, commandes manette, seuils, relâchement et zone morte).
- `cargo clippy --offline --all-targets -- -D warnings` : réussi.
- `--self-test` : trois parcours automatisés atteignent `Victory`, en partant respectivement des secteurs 1, 2 et 3. Scores constatés : 172230, 126780 et 76170. Ce pilote emploie l'invulnérabilité et des armes améliorées ; il valide la progression et la possibilité de détruire les boss, pas la difficulté humaine.
- `--smoke-test` : huit captures (titre, trois vols, trois boss et victoire), sortie normale. Examen visuel de l'interface, des sprites et de l'alignement des réacteurs.
- `--smoke-test --4k` : mêmes huit scènes avec cible interne 3840 × 2160, sortie normale. Les captures de fenêtre restent à 1440 × 810 sur cet écran ; elles ne constituent pas une mesure de fréquence d'images.
- Application `.app` ouverte normalement : Entrée, tir maintenu, déplacement, dash, entraînement, pause, aide, son et changement de qualité exercés via les entrées clavier natives. Pause, aide et retour au titre observés. Pas d'exception audio au chargement ; aucune écoute humaine de mixage n'est revendiquée.
- `tools/validate_assets.py` : 12 images originales présentes ; les 9 sprites ont un canal alpha avec fond transparent et des coins d'alpha 0 ou 1/255. Les noirs des matériaux sont conservés. Aucun rectangle opaque observé dans les captures.
- 18 WAV vérifiés : stéréo PCM 16 bits, 44100 Hz, signal non nul, crêtes individuelles sous la saturation. SHA-256 et données audio dans `assets/manifest.json`.
- Signature locale ad hoc vérifiée avec `codesign --verify --deep --strict` ; archives ZIP contrôlées après fabrication.

## Nouvelle affectation — 0.1.2

B8 maintenu tire ; B6 déclenche la surcharge de six secondes uniquement à 100 d’énergie. B7 n’est plus affecté. L’aide et le guide sont à jour. Les tests vérifient les deux commandes séparément, le maintien du tir, l’appui unique de surcharge et l’absence d’action de B7. Aucun nouvel essai physique des boutons n’est revendiqué pour cette réaffectation.

## Ajout Logitech Dual Action — 0.1.1

- Détection réelle réussie hors sandbox : `Logitech Dual Action`, USB `046d:c216`.
- Le diagnostic a reçu des déplacements analogiques non nuls puis un retour au neutre, B1 (Impulsion) et B7 (tir). Les autres boutons disposent de tests automatisés ; ne pas confondre ces tests avec un essai physique complet de tous les boutons.
- macOS : les boutons sont lus par numéro HID physique, X/Y par usages `0x30` / `0x31`, indépendamment de l'ordre des axes de la table SDL. B11 est un clic et n'est pas nécessaire pour déplacer le vaisseau.
- Zone morte radiale, vitesse diagonale limitée, répétition temporisée au menu, actions uniques sur appui et tir maintenu. Le stick droit reste sans affectation.
- B8 exige maintenant exactement 100 d'énergie (l'ancien seuil du moteur était 99) et conserve la surcharge de six secondes. Les armes B4/B3 restent verrouillées tant qu'elles ne sont pas obtenues.
- F2 et `--no-gamepad` permettent de jouer sans manette. Clavier et souris sont conservés. B10 lance/recommence et suspend/reprend la partie.
- Le mode de captures ne démarre pas le pilote HID. Le moteur maintient les actions ponctuelles jusqu'au prochain tick de simulation, y compris si le rendu est plus rapide que 120 Hz.
- La déconnexion suspend le jeu et remet les entrées de la manette à zéro ; le débranchement/rebranchement physique n'a pas été exercé dans cette session.

## Provenance

Trois décors, un intercepteur, cinq ennemis et trois boss générés spécifiquement pour ce jeu avec imagegen. Musiques et bruitages composés et synthétisés par le code Rust de `src/audio.rs`. Effets visuels, simulation, niveaux et comportements nouveaux. Aucun fichier ROM, musique, sprite ou code du jeu Gradius 2 repris. Les deux polices sont tierces, sous OFL, licences incluses.

## Limites de cette première version

- Tests macOS Apple Silicon uniquement. Aucune validation Intel, Windows ou Linux.
- Support optionnel Logitech Dual Action USB uniquement ; autres modèles non vérifiés. Pas de réseau ou sauvegarde de campagne ; le record est sauvegardé.
- Équilibrage à affiner par de vrais essais de difficulté. Campagne courte de trois secteurs et trois boss.
- Les volumes de collision sont simplifiés ; les grandes ailes décoratives n'ont pas de masque pixel par pixel.
- Rendu GPU OpenGL via Macroquad ; pas de backend Metal validé. Le mode 4K est une résolution interne, pas une garantie de 4K native pour chaque dessin ou de 120 images/seconde.
- Application signée localement, sans notarisation Apple ni signature Developer ID pour distribution publique.

## Reproduire

Voir les commandes de compilation, de synthèse et de captures dans `README.md`.
Le validateur de ressources utilise Python 3 et Pillow (`PIL`).
