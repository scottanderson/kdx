# Files: catalog, search, launch, delete, rename, volumes

Status: ready-for-human
Scenario: `linux-catalog-volumes`
Targets: 404, 406, 420, 422, 423, 450, 451, 453

Setup (agent): server detached; `Bases/Default/lab-files/` with `a.txt`,
`b.txt`, and `hello.sh` (prints a line).

1. Connect as admin; Files button; open `lab-files`.
2. Commands → 127.0.0.1 → Generate File Catalog (450/451).
3. Commands → 127.0.0.1 → Search Files..., search `a.txt` (452/453).
4. Right-click `a.txt`, Get Info; change the name to `a2.txt`, Save (411 or
   406: which one the client uses is the question).
5. Right-click `b.txt`, Delete (405 or 404: same question).
6. Right-click `hello.sh`, Launch Program (420; an error reply is fine).
7. Click Back to the top level. Right-click the top level, Get Info, and
   look for volume or base fields (422/423 [I]). If editable, change one and
   Save.
