# Lyxal Engine: Base-X (Hardened)

**Moteur de conversion de base haute performance, sécurisé et certifié `no_std`.**

`base-x` est le composant noyau de `lyxal_encoding` responsable de la conversion bidirectionnelle entre données binaires et représentations textuelles dans n'importe quelle base (Base58, Base62, etc.). Cette version a été lourdement durcie pour répondre aux standards de sécurité critiques du noyau Lyxal.

## 🛡 Garanties "Production-Grade"

Ce module a été refabriqué pour offrir des garanties de robustesse maximales :

- **Zéro Panic** : L'API a été migrée vers des retours `Result`. Aucune entrée malveillante ne peut faire planter le processus.
- **Zéro Unsafe** : Suppression totale des blocs `unsafe`. Les garanties mémoire reposent exclusivement sur le compilateur Rust.
- **DDoS Resilient** : Support natif du parsing sans allocation (`no_alloc`). Les limites de buffers sont vérifiées à la compilation et à l'exécution.
- **no_std & no_alloc** : Compatible avec les environnements les plus contraints (firmware, micro-noyaux).

### 🚀 Performances
- **Vitesse** : Le mode `no_alloc` est environ **2x plus rapide** que le mode `alloc` grâce à la suppression des cycles de gestion du tas (heap).
- **Déterminisme** : Utilise un buffer BigInt fixe de 512 octets (128 chunks u32), couvrant 100% des besoins standards (PeerIDs, Clés Crypto).

## 🚀 Utilisation

### Mode Standard (avec `alloc`)

```rust
use base_x;

let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
let data = b"hello world";

// Encodage sécurisé
let encoded = base_x::encode(alphabet, data).expect("Alphabet invalide");
assert_eq!(encoded, "StV1DL6CwTry7suV");

// Décodage sécurisé
let decoded = base_x::decode(alphabet, &encoded).expect("Données corrompues");
assert_eq!(decoded, data);
```

### Mode Noyau Lyxal (sans allocation)

```rust
use base_x::encode_to_buffer;

let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
let input = [0xDE, 0xAD, 0xBE, 0xEF];
let mut buffer = [0u8; 64];

// Encodage sur pile (zero-allocation)
let len = encode_to_buffer(alphabet.as_bytes(), &input, &mut buffer).unwrap();
let result = core::str::from_utf8(&buffer[..len]).unwrap();
```

## 📋 Architecture des Erreurs

- `EncodeError` : Retourné si le buffer est trop petit, l'entrée trop large pour le buffer statique, ou si l'alphabet est invalide.
- `DecodeError` : Retourné si le format d'entrée ne correspond pas à l'alphabet ou si les données sont malformées.

## 🧪 Sécurité et Fuzzing

Ce module est audité via `cargo-fuzz`. Les cibles de fuzzing se trouvent dans le répertoire `fuzz/` et couvrent les chemins de code `alloc` et `no_alloc` pour garantir l'absence de régressions ou de dépassements de capacité.
