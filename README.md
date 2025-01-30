### Move commands

- Pour les commandes qui passent de commands-legacy à commands en version courante.

1. Déplacer à la main la commande de commands-legacy à commands en EN
2. mettre l'exe à côté du docusaurus.config.js et faire 

`move_command.exe -f abs.md -d ./docs/`

abs.md c'est le fichier à déplacer

autre ex:
`move_command.exe -f wp-truc.md -d ../../docs/`

3. Supprimer à la main le fichier déplacé de commands-legacy dans les i18n  
