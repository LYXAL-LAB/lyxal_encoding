# TODO: Data-Encoding 

Ce document répertorie les travaux restants pour atteindre l'état final de la version 3, en se concentrant sur la sécurité critique et les extensions de fonctionnalités.

## 🔴 Priorité Critique : Sécurité & Robustesse
- [x] **Élimination des Paniques (#126)** : Parcourir `src/lib.rs` pour remplacer tous les `assert!` et `unwrap()` par des retours d'erreurs `Result` dans les fonctions `_mut` (sans allocation). Une erreur de taille de buffer doit retourner `Error` et non crasher.
- [x] **Gestion des Débordements Arithmétiques (#145)** : Modifier `encode_len()` et `decode_len()` pour qu'ils retournent un `Result<usize, OverflowError>` afin de sécuriser les calculs sur les entrées massives (particulièrement sur architectures 32-bit).
- [x] **Standardisation `as_chunks` (#74)** : Remplacer le code de découpage manuel par la fonction standard stable `slice::as_chunks`.

## 🟠 Priorité Haute : Extension & Unification (Projet Lyxal)
- [ ] **Fusion base-x** : Intégrer un moteur arithmétique pour supporter les bases non-puissance de 2 (**Base58**, **Base62**, **Nix-Base32**) directement dans `data-encoding`.
- [x] **Optimisation SIMD (#95)** : Implémenter le support SIMD (SSSE3) pour les encodages les plus fréquents (Base64, Hex).
- [x] **Transition `const fn` (#72)** : Rendre les fonctions de calcul et de spécification `const fn` pour permettre des définitions d'encodages statiques.

## 🟡 Priorité Moyenne : Ergonomie & Raffinement
- [x] **Abstractions de Types** : Remplacer l'usage des types génériques `True` et `False` par des traits explicites (ex: `BitOrder`, `PaddingMode`) pour améliorer la lisibilité de l'API et éviter les erreurs de paramètres.
- [x] **Gestion de Padding Avancée** : Ajouter les modes de remplissage `PadConcat` et `PadFinal`.
- [x] **Réduction de l'Empreinte Mémoire** : Limiter la taille maximale du séparateur (`wrap`) à 15 octets pour optimiser les structures internes. *(Note: Implémenté via InternalEncoding::Owned fixe)*.

## 🟢 Documentation
- [x] **Mise à jour Technique** : Réviser l'intégralité de la documentation de `src/lib.rs` pour refléter les garanties de "Zéro Panic" et les nouveaux types génériques.

---
*Note: Les tâches concernant le support `no_std`, l'usage de `MaybeUninit`, le passage en édition 2024 et le Property-Testing intensif ont déjà été complétées.*
