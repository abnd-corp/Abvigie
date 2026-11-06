# Cibles communes de l'organisation (charte R21) : build, test, lint, check. Un projet peut en ajouter, pas en retirer.
.PHONY: help build test lint check run pages fixtures audit js size clean

help: ## Liste les commandes
	@grep -E '^[a-z-]+:.*##' $(MAKEFILE_LIST) | sed 's/:.*##/ :/'

build: ## Compile la version de livraison (un seul binaire)
	cargo build --release --locked

test: ## Lance tous les tests
	cargo test --locked

lint: js ## Format, clippy sans avertissement, types du JavaScript
	cargo fmt --all --check
	cargo clippy --all-targets --locked -- -D warnings

check: lint test ## Ce qui doit passer avant d'ouvrir une PR (R4)

run: ## Lance le site en local sur http://127.0.0.1:8080, avec les données de démonstration
	cargo run --locked -- --fixtures server/fixtures

pages: ## Écrit le site statique (démonstration, GitHub Pages) dans public/
	cargo run --release --locked -- --fixtures server/fixtures --export public

fixtures: ## Recapture les données publiques de l'organisation (utilise gh)
	scripts/capturer-fixtures.sh

audit: ## Avis de sécurité et licences des dépendances (cargo-deny, cargo-audit)
	cargo deny check
	cargo audit

js: ## Vérifie les types de app.js (JSDoc) avec tsc, s'il est installé
	@if command -v tsc >/dev/null 2>&1; then tsc -p tsconfig.json; else echo "tsc absent : vérification des types JavaScript ignorée"; fi

size: ## Poids du binaire de livraison et du dossier target/
	@ls -l target/release/abvigie 2>/dev/null | awk '{printf "binaire : %.2f Mo\n", $$5/1048576}' || true
	@du -sh target 2>/dev/null | awk '{print "target/ : "$$1}' || true

clean: ## Supprime target/ (plusieurs centaines de Mo)
	cargo clean
