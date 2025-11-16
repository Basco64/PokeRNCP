### Introduction

Bonjour, je m’appelle Xabi Martinez. Je suis chauffeur routier en reconversion et j’ai toujours utilisé le développement pour me simplifier la vie ou celle des gens autour de moi. Longtemps, j’ai pensé que ce métier était inaccessible sans de longues études, d'ou ma reconversion tardive.
Aujourd'hui, je vous présente PokeRNCP, un backend de Pokédex développé en Rust, nommé PokeRNCP. L’objectif est de montrer une mise en œuvre moderne, sécurisée et déployable, alignée avec le référentiel RNCP pour la partie back‑end.

### Contexte et objectifs

Cette API a été réalisé dans le cadre du Titre Professionnel “Développeur Web & Web Mobile”. J’ai choisi de concentrer mon effort sur un périmètre back‑end complet: une API REST, une base relationnelle modélisée proprement, une sécurité crédible, des tests et une conteneurisation prête à l’emploi. Mon double objectif était de monter en compétence sur Rust, et de présenter une architecture qu’on peut expliquer et maintenir en conditions réelles.

### Contraintes et posture projet

Étant en reconversion et encore actif professionnellement, la contrainte majeure a été la gestion du temps. J’ai adopté une approche pragmatique : d’abord un MVP fonctionnel, puis des itérations ciblées.
Choisir Rust signifiait accepter une montée en compétence légérement complexe. Mais c’est aussi un excellent choix pour démontrer sérieux, sécurité et robustesse.
Pour garder une cohérence globale, j’ai documenté chaque étape.
Cette posture m’a permis de rester efficace tout en construisant une architecture propre et argumentée.

### Architecture technique

Voici l’architecture générale :
Le cœur du projet est une API REST développée avec Axum, un framework Rust orienté performance et middlewares.
Elle communique avec une base PostgreSQL via SQLx, qui compile les requêtes SQL pour garantir leur validité dès la compilation.
La sécurité repose sur un duo de JWT access/refresh, avec des cookies HttpOnly pour réduire les risques d’attaque.
Enfin, toute la pile est conteneurisée via Docker multi-stage pour un binaire Rust léger et docker-compose pour orchestrer API et base de données.

### Méthode Merise

Pour assurer une base cohérente et évolutive, j’ai appliqué Merise :
D’abord le MCD, avec trois entités : User, Pokémon, et la relation User_Pokemon pour gérer le N↔N — indispensable pour représenter la capture.
Dans le MLD, j’ai conservé un modèle normalisé, en évitant les duplications : l’attribut caught n’est pas stocké mais dérivé via une clause EXISTS, ce qui garantit l’intégrité.
Les contraintes SQL portent principalement sur les identifiants, les clés étrangères avec ON DELETE CASCADE, et un UNIQUE (user_id, pokemon_id) pour l’idempotence.

### Sécurité et authentification

La sécurité a été un point majeur du projet.
J’utilise un système dual JWT : un access token très court pour limiter l’exposition, et un refresh token long, protégé dans un cookie HttpOnly.
Les cookies utilisent SameSite Lax ou Strict pour limiter le CSRF. Pour renforcer encore, les appels sensibles exigent un header explicite.
Les mots de passe sont hachés avec Argon2, standard moderne recommandé.
L’ensemble des contrôles — validation, extraction des Claims — est géré via des middlewares Axum dédiés.

### Flux applicatifs

Je vous présente deux flux centraux :
Le login : l’utilisateur envoie email + mot de passe, le backend compare avec le hachage Argon2, génère un JWT access et un refresh dans un cookie HttpOnly.
La capture : l’utilisateur authentifié demande à capturer un Pokémon. L’opération est idempotente : si la ligne existe déjà, ON CONFLICT DO NOTHING.
Le calcul caught est dynamique, ce qui évite les incohérences.
Ces flux démontrent la maîtrise des permissions, de la validation et des opérations atomiques côté base.

### Extraits de code

La fonction list_all compose la liste des Pokémon et, pour chacun, demande à PostgreSQL s’il existe déjà une capture pour l’utilisateur courant. Cette vérification utilise EXISTS, qui répond dès la première occurrence. Les paramètres sont liés pour éviter toute injection, et SQLx moule chaque ligne dans la structure attendue.

La fonction catch recherche d’abord l’identifiant du Pokémon à partir de son nom. Si rien ne correspond, la réponse est claire: “introuvable”. Sinon, la capture est enregistrée dans la table associative. Grâce à la contrainte d’unicité, l’opération reste propre même si elle est appelée deux fois.

### Performance et optimisation

La performance a été pensée dès la conception.
Rust + Tokio fournissent un runtime asynchrone performant et très stable.
SQLx compile les requêtes au build, ce qui évite toute erreur SQL en production.
Le service gère la disponibilité de la BD au démarrage via un retry automatique.
Enfin, les limites (LIMIT) et l’idempotence évitent la surconsommation de ressources, tandis que le calcul caught reste cohérent sans coût d’écriture supplémentaire.

### Tests et qualité

J’ai écrit plusieurs tests d’intégration utilisant reqwest pour reproduire des appels clients réels.
L’outil nextest permet une exécution parallélisée et bien plus rapide.
Rust encourage déjà la qualité, mais clippy permet d’aller plus loin avec des recommandations idiomatiques.
Une documentation Google Doc complète détaille modélisation, migrations, API, sécurité et tests.

### Conteneurisation et déploiement

Le déploiement utilise un Docker multi-stage : d’abord compilation en image Rust, puis extraction du binaire dans une image minimale, ce qui réduit sensiblement la taille.
docker-compose coordonne le backend et PostgreSQL, avec gestion automatique du réseau.
Les variables d’environnement sécurisent la configuration.
Enfin, un tunnel HTTPS cloudflared est prêt pour exposer l’API de manière sécurisée si nécessaire.

### Compétences RNCP couvertes

Cette slide fait le lien direct avec le référentiel du Titre Professionnel.
On retrouve la modélisation relationnelle rigoureuse, l’exposition d’une API REST, une sécurité sérieuse, un traitement des données fiable, des tests, et un déploiement conteneurisé.
Ce projet démontre que je suis capable de produire un backend complet, stable et prêt à être intégré dans un environnement professionnel.

### Conclusion

Pour conclure : PokeRNCP est un backend complet, sécurisé, testé et déployable.
Il m’a permis de consolider mes compétences en Rust, d’appliquer réellement tout le référentiel back-end RNCP, et de travailler avec une exigence proche d’un environnement professionnel.
Merci pour votre écoute.
