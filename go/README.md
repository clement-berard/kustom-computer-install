# Ma CLI Simple

Une CLI simple en Go avec Cobra.

## Installation
```bash
go mod download
go build -o mycli
```

## Utilisation

### Afficher l'aide
```bash
./mycli help
```

### Lister les fichiers (ls -la)
```bash
# Répertoire courant
./mycli list

# Répertoire spécifique
./mycli list /tmp
```

## Commandes disponibles

- `list [path]` - Liste les fichiers d'un répertoire (défaut: répertoire courant)
- `help` - Affiche l'aide (généré automatiquement par Cobra)

## Structure

Le projet utilise **Cobra**, l'outil standard pour créer des CLIs en Go.

Cobra génère automatiquement:
- La commande `help`
- Les flags `--help` et `-h`
- La documentation pour chaque commande
