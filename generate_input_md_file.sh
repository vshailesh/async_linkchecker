#!bin/bash

INPUT="./top500Domains.csv"
rm "./input_links.md" 2>/dev/null
OLDIFS="$IFS"
IFS=','
[ ! -f $INPUT ] && {
  echo "$INPUT file not found"
  exit 99
}
(( count=0 ))
while read Rank RootDomains LinkingRootDomains DomainAuthority; do
  regex_pattern="^\"(.*)\""
  if [[ "$RootDomains" =~ $regex_pattern ]]; then
    rd="${BASH_REMATCH[1]}"
    if [[ $count -eq 0 ]]; then
      count=$(( count + 1 ))
      continue
    fi
    count=$(( count + 1 ))
    printf "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since 1966, when designers at Letraset and James Mosley, the librarian at St Bride Printing Library in London, took a 1914 Cicero translation and scrambled it to make dummy text for Letraset's Body Type sheets. [MDLink](https://$rd). It has survived not only many decades, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised thanks to these sheets and more recently with desktop publishing software like Aldus PageMaker and Microsoft Word including versions of Lorem Ipsum.\n" >>input_links.md
  else
    echo "Failed for: $RootDomains"
  fi
done <"$INPUT"
IFS="$OLDIFS"