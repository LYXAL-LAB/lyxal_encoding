# Safety & Security Audit: Base-X Hardening

Ce document détaille les mesures de durcissement appliquées au module `base-x` pour son intégration dans le noyau Lyxal.

## 1. Gestion des Panics (Panic Freedom)

Toutes les instances de `panic!`, `unwrap()` et `expect()` dans le code de production ont été auditées et supprimées ou documentées.

- **Status** : 100% Panic-Free sur les chemins de données utilisateur.
- **Remplacement** : Utilisation systématique de `Result<T, E>`.
- **Garantie** : Une entrée malformée, quelle que soit sa taille ou son contenu, ne peut pas provoquer l'arrêt du programme (DoS par crash).

## 2. Sécurité Mémoire (Memory Safety)

Le code original contenait des blocs `unsafe` pour l'optimisation des chaînes UTF-8.

- **Actions** :
    - Suppression de `String::from_utf8_unchecked`.
    - Implémentation de validations préalables (`is_ascii()`) garantissant l'intégrité UTF-8 sans coût d'exécution excessif.
    - Remplacement des accès par index non vérifiés par des itérateurs sécurisés ou des vérifications de bornes.
- **Résultat** : Code 100% Safe Rust. Les garanties de sécurité mémoire sont assurées par le vérificateur d'emprunt (Borrow Checker).

## 3. Résilience DDoS

Pour prévenir l'épuisement des ressources par des entrées massives :

- **Buffer Limits** : Le mode `no_alloc` impose une limite stricte de 512 octets (`128 chunks u32`) pour les calculs internes de BigInt sur pile.
- **Early Rejection** : Toute entrée dépassant cette limite physique est rejetée immédiatement avant tout calcul coûteux.
- **Allocation-Free** : Le support total de `no_alloc` permet à `base-x` de fonctionner dans des contextes où l'allocateur de mémoire pourrait être saturé ou non disponible.

## 4. Stratégie de Fuzzing

Le module est conçu pour être testé en continu via `libFuzzer`. Les objectifs de fuzzing incluent :
1.  Invariants d'aller-retour : `decode(encode(input)) == input`.
2.  Résistance aux alphabets Unicode exotiques.
3.  Comportement aux limites des buffers statiques (`no_alloc`).
