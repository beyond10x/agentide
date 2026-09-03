#!/bin/sh
set -eu

repository="https://github.com/beyond10x/agentide"
target="x86_64-unknown-linux-gnu"
install_dir="${AGENTIDE_INSTALL_DIR:-${HOME}/.local/bin}"

fail() {
  printf 'agentide installer: %s\n' "$*" >&2
  exit 1
}

for command_name in curl sha256sum tar install mktemp uname grep awk mkdir mv rm; do
  command -v "$command_name" >/dev/null 2>&1 \
    || fail "required command not found: ${command_name}"
done

[ "$(uname -s)" = "Linux" ] || fail "only Linux is currently supported"
[ "$(uname -m)" = "x86_64" ] || fail "only Linux x86_64 is currently supported"

version="${AGENTIDE_VERSION:-}"
if [ -z "$version" ]; then
  release_url="$(
    curl --proto '=https' --tlsv1.2 -fLsS \
      -o /dev/null -w '%{url_effective}' "${repository}/releases/latest"
  )"
  version="${release_url##*/}"
fi

printf '%s\n' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' \
  || fail "resolved release version is not a bare semantic version: ${version}"

name="agentide-${version}-${target}"
archive="${name}.tar.gz"
base="${repository}/releases/download/${version}"
temporary="$(mktemp -d)"
trap 'rm -rf "$temporary"' EXIT HUP INT TERM

printf 'Installing AgentIDE %s for %s\n' "$version" "$target"
curl --proto '=https' --tlsv1.2 -fLsS "${base}/${archive}" -o "${temporary}/${archive}"
curl --proto '=https' --tlsv1.2 -fLsS "${base}/SHA256SUMS" -o "${temporary}/SHA256SUMS"

checksum_line="$(
  awk -v archive="$archive" '
    $2 == archive { count += 1; line = $0 }
    END { if (count != 1) exit 1; print line }
  ' "${temporary}/SHA256SUMS"
)" || fail "SHA256SUMS must contain exactly one checksum for ${archive}"

(
  cd "$temporary"
  printf '%s\n' "$checksum_line" | sha256sum --check -
) || fail "checksum verification failed for ${archive}"

tar -xzf "${temporary}/${archive}" -C "$temporary" "${name}/agentide"
[ -f "${temporary}/${name}/agentide" ] || fail "release archive did not contain ${name}/agentide"

mkdir -p "$install_dir"
install_tmp="$(mktemp "${install_dir}/.agentide.XXXXXX")"
trap 'rm -rf "$temporary"; rm -f "$install_tmp"' EXIT HUP INT TERM
install -m 0755 "${temporary}/${name}/agentide" "$install_tmp"
mv -f "$install_tmp" "${install_dir}/agentide"

installed_version="$("${install_dir}/agentide" --version)"
case "$installed_version" in
  "agentide ${version}"|"agentide-cli ${version}") ;;
  *) fail "installed binary reported '${installed_version}', expected AgentIDE ${version}" ;;
esac

printf 'Installed %s at %s/agentide\n' "$installed_version" "$install_dir"
case ":${PATH}:" in
  *":${install_dir}:"*) ;;
  *) printf 'Add %s to PATH, then run: agentide run\n' "$install_dir" ;;
esac
