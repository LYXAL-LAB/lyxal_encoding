# Lyxal Engine: Base45 (Hardened)

**Moteur d'encodage Base45 haute performance, sécurisé et certifié `no_std`.**

`base45` est une composante essentielle de `lyxal_encoding`, spécialisée dans l'encodage compact de données binaires, notamment utilisé dans les standards de santé (Health Certificates) et les protocoles QR code. Cette implémentation a été durcie pour répondre aux exigences "Production-Grade" du noyau Lyxal.

## 🛡 Garanties "Production-Grade"

Ce module respecte les standards de qualité industrielle les plus élevés :

- **Conformité IETF** : Implémentation stricte du [draft-faltstrom-base45](https://datatracker.ietf.org/doc/draft-faltstrom-base45/).
- **Zéro Panic** : Conception robuste garantissant l'absence de crashs (`panic!`) sur des entrées malveillantes.
- **Sécurité Mémoire** : Utilisation minimisée et auditée des blocs `unsafe`.
- **Validé par Fuzzing** : Infrastructure de fuzzing (`cargo-fuzz`) validant en continu la propriété de round-trip (`decode(encode(x)) == x`).
- **No-std** : Support natif pour les environnements embarqués via `alloc`.

### 🚀 Performances

Benchmarks réalisés sur une architecture standard :

| Opération | Débit Moyen |
|-----------|-------------|
| **Encode** | ~340 MiB/s |
| **Decode** | ~170 MiB/s |

L'implémentation est optimisée pour minimiser les allocations et maximiser le débit sur les architectures modernes.

## 🚀 Utilisation

### Mode Standard

```rust
use base45;

let data = "Hello!!";

// Encodage
let encoded = base45::encode(data);
assert_eq!(encoded, "%69 VD92EX0");

// Décodage
let decoded = base45::decode(&encoded).expect("Données invalides");
assert_eq!(String::from_utf8(decoded).unwrap(), data);
```

### Gestion des Erreurs

La fonction `decode` retourne un `Result`, permettant une gestion fine des erreurs (caractères invalides, longueur incorrecte, etc.) sans risque d'arrêt du programme.

## 🧪 Tests et Validation

Le module est validé par trois niveaux de tests :
1.  **Tests Unitaires** : Couverture des vecteurs de test officiels.
2.  **Property Testing** : `proptest` vérifie la cohérence de l'encodage/décodage sur des millions de séquences aléatoires.
3.  **Fuzzing** : Tests de robustesse intensifs via `libFuzzer`.