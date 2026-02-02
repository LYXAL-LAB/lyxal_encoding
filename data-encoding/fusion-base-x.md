# Plan de Fusion : Intégration de Base-X dans Data-Encoding (TERMINÉ)

## 🎯 Objectif
Intégrer le moteur arithmétique de `base-x` dans `data-encoding` pour supporter les bases non-puissances de 2 (Base58, Base62, Nix-Base32) tout en préservant les garanties de performance et de sécurité existantes.

## 📋 Plan de Fusion Détaillé

### Phase 1 : Intégration du Moteur Arithmétique

- [x] **Ajout du module BigInt**
  - [x] Intégrer `bigint.rs` dans `data-encoding/src/` avec les fonctionnalités `BigUintView`
  - [x] Adapter pour le support `no_std` complet
  - [x] Conserver les optimisations de buffer fixe (128 chunks u32 = 512 octets)

- [x] **Ajout des modules d'encodage/décodage arithmétique**
  - [x] Intégrer les fonctionnalités d'encodage `encode_to_buffer` et `encode`
  - [x] Intégrer les fonctionnalités de décodage avec validation d'alphabet
  - [x] Adapter pour l'API `Result` avec gestion d'erreurs appropriée

### Phase 2 : Extension de l'API

- [x] **Extension de `Specification`**
  - [x] Ajouter un champ `use_arithmetic` pour forcer l'arithmétique si nécessaire
  - [x] Ajouter la détection automatique des bases non-puissances de 2
  - [x] Valider les alphabets (doivent être ASCII)

- [x] **Modification de `Encoding`**
  - [x] Étendre `encode_mut` et `decode_mut` pour déléguer au moteur arithmétique
  - [x] Préserver le stockage statique (objet `Encoding` de 531 octets)
  - [x] Utiliser l'index 384 pour stocker dynamiquement la taille de la base

### Phase 3 : Intégration et Tests

- [x] **Ajout des constantes standard**
  - [x] Base58 (alphabet Bitcoin)
  - [x] Base62 (0-9A-Za-z)

- [x] **Tests et validation**
  - [x] Tests unitaires de round-trip (Base58/Base62) incluant les leaders (zéros de tête)
  - [x] Validation des bornes `encode_len` et `decode_len`
  - [x] Correction des erreurs de récupération d'alphabet (`get_symbols`)

### Phase 4 : Documentation et Benchmarks

- [x] **Documentation**
  - [x] Mettre à jour le README.md avec les nouveaux standards supportés
  - [x] Documenter l'API `encode_mut` mise à jour (retourne `Result<usize, ...>`)

- [x] **Benchmarks et Robustesse**
  - [x] Validation via les benchmarks existants (pas de régression SIMD)
  - [x] Mise à jour de la cible de fuzzing pour couvrir les chemins arithmétiques

## 🛠 Implémentation Technique

### Structure Finale

```
data-encoding/src/
├── lib.rs          # API unifiée, constantes, dispatching
├── bigint.rs       # Moteur BigInt sans allocation
└── arithmetic.rs   # Logique d'encodage arithmétique (Base-X)
```

## 🚀 Résultat de la Fusion

1. **API Unifiée** : `BASE58.encode()` s'utilise exactement comme `BASE64.encode()`.
2. **Zéro Allocation** : Toujours garanti pour les données < 512 octets.
3. **Zéro Panic** : Gestion complète des erreurs via `Result`.
4. **Indépendance** : La bibliothèque `base-x` est maintenant totalement remplacée par cette implémentation durcie.