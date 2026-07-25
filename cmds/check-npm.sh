#!/usr/bin/env bash

GREEN='\033[0;32m'; RED='\033[0;31m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
FAIL=0

NODE_V=$(node -v 2>/dev/null)
if [ -n "$NODE_V" ]; then
  echo -e "${BLUE}ℹ${NC} Node $NODE_V"
else
  echo -e "${YELLOW}⚠${NC} Node introuvable"
fi

PNPM_V=$(pnpm -v 2>/dev/null)
if [ -n "$PNPM_V" ]; then
  echo -e "${BLUE}ℹ${NC} pnpm $PNPM_V"
fi

NPM_MAJOR=$(npm -v 2>/dev/null | cut -d. -f1)
if [ "${NPM_MAJOR:-0}" -lt 11 ]; then
  echo -e "${RED}✘${NC} npm trop ancien (< 11)"
  echo "  → npm install -g npm@latest"
  FAIL=1
else
  echo -e "${GREEN}✔${NC} npm v$(npm -v)"
fi

WHOAMI=$(npm whoami --registry https://registry.npmjs.org 2>&1)
if [ $? -ne 0 ]; then
  echo -e "${RED}✘${NC} Pas connecté / session expirée"
  echo "  → npm login"
  FAIL=1
else
  echo -e "${GREEN}✔${NC} Connecté: $WHOAMI"
fi

PING=$(npm ping --registry https://registry.npmjs.org 2>&1)
if ! echo "$PING" | grep -qi "pong\|took"; then
  echo -e "${RED}✘${NC} Registry inaccessible"
  echo "  → npm ping --registry https://registry.npmjs.org"
  FAIL=1
else
  echo -e "${GREEN}✔${NC} Registry joignable"
fi

TOKENS=$(npm token list 2>&1)
if [ $? -ne 0 ]; then
  echo -e "${RED}✘${NC} Tokens illisibles (session invalide)"
  echo "  → npm login"
  FAIL=1
elif echo "$TOKENS" | grep -qi "bypass"; then
  echo -e "${YELLOW}⚠${NC} Token bypass 2FA actif (restrictions 2027)"
else
  echo -e "${GREEN}✔${NC} Tokens OK"
fi

REGISTRY=$(npm config get registry)
if [ "$REGISTRY" != "https://registry.npmjs.org/" ]; then
  echo -e "${YELLOW}⚠${NC} Registry non standard: $REGISTRY"
  echo "  → npm config set registry https://registry.npmjs.org/"
else
  echo -e "${GREEN}✔${NC} Registry standard"
fi

exit $FAIL
