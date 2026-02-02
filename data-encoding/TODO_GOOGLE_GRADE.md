# 🚀 Roadmap: Data-Encoding "Google Grade"

Ce document liste les tâches nécessaires pour élever la crate `data-encoding` au niveau de qualité industrielle ("Google Grade" / "Production Ready").

## 🔴 Phase 1 : Corrections Critiques (Stabilité)

- [x] **Régénération Complète de `src/data.rs`**
    - [x] Utiliser le script de génération pour **toutes** les bases standards :
        - `HEXLOWER`, `HEXUPPER` (et variantes permissives)
        - `BASE32`, `BASE32_NOPAD`, `BASE32HEX`, etc.
        - `BASE64`, `BASE64URL`, `BASE64_MIME`
    - [x] Vérifier que les tables contiennent les valeurs de lookup correctes (et non des zéros).
    - [x] Valider que `BASE58` et `BASE62` sont préservés.
- [x] **Validation des Tests Unitaires**
    - [x] S'assurer que `cargo test` passe sans échec (réparation des "Roundtrip" Hex et Base64).

## 🟠 Phase 2 : Qualité du Code (Linting & Cleanliness)

- [ ] **Résolution des Warnings (`cargo check`)**
    - [ ] Supprimer ou corriger les imports inutilisés (`unused imports`) dans `arithmetic.rs` et `lib.rs`.
    - [ ] Gérer les variables assignées mais jamais lues (prefixer par `_`).
    - [ ] Supprimer le code mort (`dead_code`) ou l'exposer si nécessaire.
- [ ] **Vérification des Features**
    - [ ] Vérifier pourquoi `decode_hex_simd` et `decode_base64_simd` sont marqués comme inutilisés.
    - [ ] S'assurer que la feature SIMD est correctement détectée et activée par défaut sur x86_64.
- [ ] **Formatage**
    - [ ] Appliquer `cargo fmt` pour un style de code uniforme.
    - [ ] Appliquer `cargo clippy` et corriger les suggestions pertinentes.

## 🟡 Phase 3 : Performance & Métriques

- [ ] **Mise en place de Benchmarks (`criterion`)**
    - [ ] Créer un dossier `benches/`.
    - [ ] Benchmark comparatif : `lyxal_encoding::BASE58` vs crate `bs58`.
    - [ ] Benchmark comparatif : `lyxal_encoding::BASE64` vs crate `base64` (Standard vs SIMD).
    - [ ] Mesurer l'overhead de la structure `Specification`.
- [ ] **Optimisation (si nécessaire)**
    - [ ] Analyser les résultats des benchmarks.
    - [ ] Optimiser la boucle chaude de l'encodage arithmétique (`arithmetic.rs`) si elle est le goulot d'étranglement.

## 🟢 Phase 4 : Robustesse & Sécurité

- [ ] **Tests de Propriétés Avancés (`proptest`)**
    - [ ] Ajouter des tests spécifiques pour `BASE58` (gestion des *leaders* / zéros en tête).
    - [ ] Ajouter des tests de rejet : vérifier que le décodeur renvoie une erreur (et ne panique pas) sur des inputs invalides (caractères hors alphabet, longueur incorrecte).
- [ ] **Fuzzing (`cargo-fuzz`)**
    - [ ] Créer une cible de fuzzing pour `arithmetic::encode` et `arithmetic::decode`.
    - [ ] Lancer une session de fuzzing (1h minimum) pour détecter les overflows ou paniques cachées.
- [ ] **Audit `unsafe`**
    - [ ] Identifier tous les blocs `unsafe`.
    - [ ] Ajouter un commentaire `// SAFETY: ...` explicite justifiant la sûreté de chaque opération (ex: conversion `String::from_utf8_unchecked`).

## 🔵 Phase 5 : Documentation & Finition

- [ ] **Documentation API (`rustdoc`)**
    - [ ] Vérifier que toutes les fonctions publiques sont documentées.
    - [ ] Ajouter des exemples exécutables (`doctests`) pour les cas d'utilisation courants.
- [ ] **Validation README**
    - [ ] S'assurer que les exemples du `README.md` fonctionnent réellement (via doctests).
- [ ] **Release**
    - [ ] Bumper la version dans `Cargo.toml`.
    - [ ] Générer le changelog.