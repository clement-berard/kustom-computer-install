npm-check() {
    local GREEN='\033[0;32m' RED='\033[0;31m' YELLOW='\033[1;33m' BLUE='\033[0;34m' NC='\033[0m'
    local FAIL=0

    local NODE_V
    NODE_V=$(node -v 2>/dev/null)
    if [ -n "$NODE_V" ]; then
        echo -e "${BLUE}ℹ${NC} Node $NODE_V"
    else
        echo -e "${YELLOW}⚠${NC} Node not found"
    fi

    local PNPM_V
    PNPM_V=$(pnpm -v 2>/dev/null)
    if [ -n "$PNPM_V" ]; then
        echo -e "${BLUE}ℹ${NC} pnpm $PNPM_V"
    fi

    local NPM_MAJOR
    NPM_MAJOR=$(npm -v 2>/dev/null | cut -d. -f1)
    if [ "${NPM_MAJOR:-0}" -lt 11 ]; then
        echo -e "${RED}✘${NC} npm too old (< 11)"
        echo "  → npm install -g npm@latest"
        FAIL=1
    else
        echo -e "${GREEN}✔${NC} npm v$(npm -v)"
    fi

    local WHOAMI
    WHOAMI=$(npm whoami --registry https://registry.npmjs.org 2>&1)
    if [ $? -ne 0 ]; then
        echo -e "${RED}✘${NC} Not logged in / session expired"
        echo "  → npm login"
        FAIL=1
    else
        echo -e "${GREEN}✔${NC} Logged in: $WHOAMI"
    fi

    local PING
    PING=$(npm ping --registry https://registry.npmjs.org 2>&1)
    if ! echo "$PING" | grep -qi "pong\|took"; then
        echo -e "${RED}✘${NC} Registry unreachable"
        echo "  → npm ping --registry https://registry.npmjs.org"
        FAIL=1
    else
        echo -e "${GREEN}✔${NC} Registry reachable"
    fi

    local TOKENS
    TOKENS=$(npm token list 2>&1)
    if [ $? -ne 0 ]; then
        echo -e "${RED}✘${NC} Tokens unreadable (invalid session)"
        echo "  → npm login"
        FAIL=1
    elif echo "$TOKENS" | grep -qi "bypass"; then
        echo -e "${YELLOW}⚠${NC} 2FA bypass token active (2027 restrictions)"
    else
        echo -e "${GREEN}✔${NC} Tokens OK"
    fi

    local REGISTRY
    REGISTRY=$(npm config get registry)
    if [ "$REGISTRY" != "https://registry.npmjs.org/" ]; then
        echo -e "${YELLOW}⚠${NC} Non-standard registry: $REGISTRY"
        echo "  → npm config set registry https://registry.npmjs.org/"
    else
        echo -e "${GREEN}✔${NC} Standard registry"
    fi

    return $FAIL
}
