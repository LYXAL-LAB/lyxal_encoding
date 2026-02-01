# Lyxal Engine: Data Encoding (Hardened V3)

**Moteur d'encodage de données standard (Base64, Base32, Hex) optimisé, sécurisé et certifié `no_std`.**

`data-encoding` est le composant fondamental de `lyxal_parser` pour la gestion des formats d'encodage standardisés. Cette version V3 a été entièrement réécrite pour offrir des garanties de performance et de sécurité "Google Grade", essentielles pour le noyau Lyxal.

## 🛡 Garanties "Google-Grade"

Ce module respecte les standards les plus stricts de l'industrie :

- **Zéro Panic** : Toutes les fonctions exposées retournent des `Result`. Le code est conçu pour ne jamais paniquer, même sous des entrées malveillantes.
- **Zéro Allocation (Optional)** : Support complet du `no_alloc` via les APIs `_mut`. Les opérations peuvent se faire entièrement sur la pile ou dans des buffers pré-alloués.
- **Conformité RFC** : Implémentations strictes et canoniques des standards RFC4648 (Base64, Base32, Hex, Base64Url) et RFC5155 (DNSCurve).
- **Hardened** : Validé par fuzzing continu et une suite de tests extensive.

## 🚀 Performances

- **Efficacité** : Les algorithmes sont vectorisés et optimisés pour minimiser les branches conditionnelles.
- **No-Std** : Fonctionne sans la bibliothèque standard Rust, idéal pour l'embarqué et les environnements WASM critiques.
- **Benchmarks** : Validé via `criterion` pour garantir l'absence de régression de performance (nanoseconde-scale).

## 🚀 Utilisation

### Mode Standard (avec `alloc`)

L'API de haut niveau est simple et familière :

```rust
use data_encoding::BASE64;

let data = b"Hello Lyxal";

// Encodage
let encoded = BASE64.encode(data);
assert_eq!(encoded, "SGVsbG8gTHl4YWw=");

// Décodage
let decoded = BASE64.decode(encoded.as_bytes()).expect("Format invalide");
assert_eq!(decoded, data);
```

### Mode Noyau (Zéro Allocation)

Pour les environnements critiques, utilisez l'API `_mut` :

```rust
use data_encoding::BASE64;

let input = b"Hello Lyxal";
let mut output = [0u8; 64];

// Calcul de la taille nécessaire (garantie O(1))
let len = BASE64.encode_len(input.len());
assert!(len <= output.len());

// Encodage in-place
BASE64.encode_mut(input, &mut output[..len]);

// Résultat sans allocation de String
let result = core::str::from_utf8(&output[..len]).unwrap();
assert_eq!(result, "SGVsbG8gTHl4YWw=");
```

### Encodages Personnalisés

Le moteur permet de définir des encodages sur mesure avec des propriétés spécifiques (padding, caractères ignorés, etc.) via une `Specification` :

```rust
use data_encoding::Specification;

let mut spec = Specification::new();
spec.symbols.push_str("0123456789abcdef"); // Hex
spec.padding = Some('='); // Padding personnalisé
let hex_custom = spec.encoding().unwrap();
```

## 📋 Standards Supportés

Ce module fournit des constantes statiques pour les standards les plus courants :

| Constante | Standard | Description |
|-----------|----------|-------------|
| `HEXLOWER` | Base16 | Hexadécimal minuscule |
| `HEXUPPER` | Base16 | Hexadécimal majuscule (RFC4648) |
| `BASE32` | Base32 | RFC4648 avec padding |
| `BASE64` | Base64 | RFC4648 Standard |
| `BASE64URL` | Base64Url | RFC4648 URL-safe |
| `BASE64_MIME` | Base64 | RFC2045 (MIME) |

## 🧪 Sécurité et Fuzzing

La sécurité est auditée via `cargo-fuzz`. Les cibles de fuzzing (`fuzz/fuzz_targets/`) valident en permanence la propriété de "round-trip" (`decode(encode(x)) == x`) et l'absence de paniques sur des entrées aléatoires ou malformées.