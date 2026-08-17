#!bin/bash

INPUT="./top500Domains.csv"
rm "./input_links.md" 2>/dev/null
OLDIFS="$IFS"
IFS=','
[ ! -f $INPUT ] && {
  echo "$INPUT file not found"
  exit 99
}
while read Rank RootDomains LinkingRootDomains DomainAuthority; do
  regex_pattern="^\"(.*)\""
  if [[ "$RootDomains" =~ $regex_pattern ]]; then
    rd="${BASH_REMATCH[1]}"
    printf "[MDLink](https://$rd)\n" >>input_links.md
  else
    echo "Failed for: $RootDomains"
  fi
done <"$INPUT"
IFS="$OLDIFS"