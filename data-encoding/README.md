# Lyxal Engine: Data Encoding

**Moteur d'encodage de données standard (Base64, Base32, Hex) ultra-performant, sécurisé et certifié `no_std`.**

`data-encoding` est le composant cœur de la suite `lyxal_encoding`. Ce moteur représente l'état de l'art en matière d'encodage de données pour Rust, combinant des optimisations matérielles SIMD avec des garanties de sécurité strictes pour les systèmes critiques.

## 🛡 Garanties "Production-Grade"

Ce module est conçu pour être intégré dans des noyaux de base de données (Lyxal/SurrealDB) et des systèmes distribués :

- **Zéro Panic (Guaranteed)** : Toutes les API (`_mut`, `_len`, `decode`) utilisent des retours de type `Result`. Aucune assertion n'est présente dans le chemin d'exécution critique.
- **Zéro Allocation (Static Storage)** : L'objet `Encoding` est désormais `Copy` et n'utilise aucune allocation dynamique. Les spécifications personnalisées sont stockées dans un buffer fixe de 531 octets.
- **Arithmétique Sécurisée** : Protection native contre les débordements (overflows) sur les calculs de longueur d'entrée/sortie, validée sur architectures 32-bit et 64-bit.
- **Mémoire Prévisible** : Les séparateurs (`wrap`) sont inlinés et limités à 15 octets pour garantir une empreinte mémoire constante.

## 🚀 Performances : SIMD Accelerated

Le moteur détecte automatiquement les capacités de votre processeur pour activer des chemins d'exécution optimisés :

- **Hexadécimal (SSSE3)** : Encodage et décodage vectorisés traitant 16 octets par cycle. Validation ultra-rapide des symboles sans branchement.
- **Base64 (SSSE3)** : Algorithme de "bit-shuffling" pour les variantes Standard et URL-safe. Gain de performance massif par rapport aux implémentations scalaires classiques.
- **Branchement Minimal** : Utilisation de traits de types (`BitWidth`, `BitOrderTrait`) pour permettre au compilateur d'éliminer les conditions mortes au runtime.

## 🛠 Utilisation de l'API

### Mode Standard (Haute Lisibilité)

```rust
use data_encoding::BASE64;

let data = b"Lyxal Core";

// Encodage (nécessite la feature "alloc")
let encoded = BASE64.encode(data);
assert_eq!(encoded, "THl4YWwgQ29yZQ==");

// Décodage sécurisé
let decoded = BASE64.decode(encoded.as_bytes()).expect("Format invalide");
```

### Mode Noyau (Zéro Allocation & Zéro Panic)

Indispensable pour le `no_std` ou les chemins de code haute performance.

```rust
use data_encoding::{BASE64, PaddingMode};

let input = b"Performance matters";
let mut output = [0u8; 128];

// 1. Calcul sécurisé de la longueur (Result<usize, EncodeError>)
let len = BASE64.encode_len(input.len()).unwrap(); 

// 2. Encodage in-place (sans panique)
BASE64.encode_mut(input, &mut output[..len]).expect("Buffer trop petit");

// 3. Décodage partiel pour la récupération d'erreur
let mut decoded_buf = [0u8; 128];
let result = BASE64.decode_mut(&output[..len], &mut decoded_buf);
match result {
    Ok(written) => println!("Succès: {} octets", written),
    Err(partial) => eprintln!("Erreur à la position {}", partial.error.position),
}
```

## ⚙️ Configuration Avancée

La structure `Specification` permet de créer des encodages sur mesure sans compromis sur la vitesse :

```rust
use data_encoding::{Specification, PaddingMode, BitOrder};

let mut spec = Specification::new();
spec.symbols.push_str("0123456789ABCDEF");
spec.padding = Some('=');
spec.padding_mode = PaddingMode::PadFinal;
spec.bit_order = BitOrder::MostSignificantFirst;

let my_hex = spec.encoding().expect("Spécification invalide");
// my_hex est Copy et n'alloue rien sur le tas.
```

## 📋 Standards Supportés

| Constante | Standard | Optimisation |
|-----------|----------|--------------|
| `HEXLOWER` | Base16 | SIMD SSSE3 |
| `BASE32` | Base32 | Scalaire Vectorisé |
| `BASE64` | Base64 | SIMD SSSE3 |
| `BASE64URL`| Base64Url| SIMD SSSE3 |
| `BASE64_MIME`| Base64 | Scalaire Vectorisé |

## 🧪 Tests et Robustesse

- **Proptest** : 10 000 tests de propriété générés pour valider l'invariance `decode(encode(x)) == x`.
- **Cargo-Fuzz** : Fuzzing continu sur les cibles `encode` et `decode` pour détecter les cas limites.
- **Kani Rust Verifier** : Preuves formelles sur les calculs arithmétiques critiques.

---
*Version 0.0.1 - Composant cœur de la suite **Lyxal Solution**.*