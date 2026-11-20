alias y='yarn'
alias yd='yarn dev'
alias ys='yarn start'
alias yb='yarn build'
alias ya='yarn add'
alias yad='yarn add -D'

## yarn helpers
yup() {
  local yarn_version=$(yarn --version 2>/dev/null)

  if [[ "$yarn_version" == 1.* ]]; then
    echo "📦 Yarn v$yarn_version (Classic) détecté..."
    yarn upgrade-interactive --latest
  else
    echo "📦 Yarn v$yarn_version (Berry) détecté..."
    yarn upgrade-interactive
  fi
}

ywhy() {
  # Default: full dependency tree grouped by version. -s: compact summary.
  if [[ "$1" == "-s" ]]; then
    shift
  else
    _ywhy_tree "$@"
    return
  fi
  echo "🔍 Analyse de $1..."

  # Yarn Berry (>= 2) : pas de --no-color et sortie différente -> on passe par --json
  if [[ "$(yarn --version 2>/dev/null)" != 1.* ]]; then
    yarn why "$1" --json 2>/dev/null | jq -rs '
      [ .[] | select(.value | contains("@virtual:") | not)
        | .value as $parent
        | .children | to_entries[]
        | select(.key | contains("@virtual:") | not)
        | { v: (.key | sub(".*@npm:"; "")),
            line: ("    ↳ " + ($parent | sub("::__archiveUrl.*"; "")) + "  (via " + (.value.descriptor | sub(".*@npm:"; "")) + ")") } ]
      | if length == 0 then "❌ Aucune dependance trouvee."
        else group_by(.v)[] |
          "\n=======================================\n🎯 VERSION : " + .[0].v
          + "\n=======================================\n"
          + ([.[].line] | unique | join("\n"))
        end'
    return
  fi

  yarn why "$1" --no-color 2>/dev/null | awk '
    /=> Found/ {
      # On isole ce qui est entre les guillemets
      str = $0
      sub(/.*=> Found "/, "", str)
      sub(/".*/, "", str)

      # On extrait la version
      version = str
      sub(/.*@/, "", version)
      current_v = version

      if (!seen[current_v]++) {
        versions[++v_count] = current_v
      }

      data[current_v] = data[current_v] "\n  📦 " str
      next
    }

    /info This module exists because/ {
      sub(/.*info This module exists because /, "    ↳ ")
      data[current_v] = data[current_v] "\n" $0
      next
    }

    # Capture les lignes de liste
    /^[[:space:]]*-/ {
      sub(/^[[:space:]]*-[[:space:]]*(Hoisted from )?/, "    ↳ ")
      # Nettoyage des guillemets
      gsub(/"/, "", $0)
      data[current_v] = data[current_v] "\n" $0
      next
    }

    END {
      if (v_count == 0) {
        print "❌ Aucune dependance trouvee."
        exit
      }
      for (i = 1; i <= v_count; i++) {
        v = versions[i]
        print "\n======================================="
        print "🎯 VERSION : " v
        print "======================================="
        print data[v]
      }
      print "\n"
    }
  '
}

_ywhy_tree() {
  gum style --bold --foreground 212 "🌳 Arbre complet de $1 (par version)"
  # npm ls reads node_modules, so it works for Yarn 1 and Berry (node-modules linker)
  npm ls "$1" --all --json 2>/dev/null | jq -r --arg p "$1" '
    def hl: "\u001b[1;30;43m " + . + " \u001b[0m";
    def chains($chain):
      (.dependencies // {}) | to_entries[]
      | .key as $k | .value as $v
      | ($chain + [$k + "@" + ($v.version // "?")]) as $c
      | if $k == $p then {v: ($v.version // "?"), chain: $c} else empty end,
        ($v | chains($c));
    def render($pre):
      to_entries as $es
      | range(0; $es | length) as $i
      | $es[$i] as $e
      | ($i == ($es | length) - 1) as $last
      | ($pre + (if $last then "└── " else "├── " end)
         + (if ($e.key | startswith($p + "@")) then ($e.key | hl) else $e.key end)),
        ($e.value | render($pre + (if $last then "    " else "│   " end)));
    [ chains([]) ]
    | if length == 0 then "##NONE"
      else group_by(.v)[] |
        "##" + .[0].v,
        ( reduce (.[].chain) as $c ({}; setpath($c; getpath($c) // {})) | render("") )
      end' | while IFS= read -r line; do
    if [[ "$line" == "##NONE" ]]; then
      gum style --foreground 196 "❌ Aucune dependance trouvee."
    elif [[ "$line" == "##"* ]]; then
      echo
      gum style --bold --foreground 212 --border rounded --border-foreground 99 --padding "0 2" "🎯 VERSION ${line#\#\#}"
    else
      echo "$line"
    fi
  done
}
